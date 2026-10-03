#![deny(unsafe_code)]

//! Verification test suite for Dynamic Floorplan Mesh Generator,
//! ADI Transient Heat Diffusion, and Monolithic Electro-Thermal Co-Simulator.

use phonon_core::CircuitGraph;
use phonon_models::diode::DiodeModel;
use phonon_solver::mna::ModelContext;
use phonon_solver::SolverError;
use phonon_thermal::floorplan::{DieProperties, DynamicFloorplanMesh, FloorplanComponent};
use phonon_thermal::transient_co_sim::ElectroThermalCoSimulator;
use std::collections::HashMap;

#[test]
fn test_floorplan_mesh_power_distribution_across_cells() {
    let die = DieProperties {
        width: 0.005,      // 5 mm
        height: 0.005,     // 5 mm
        thickness: 0.0003, // 300 um
        ..Default::default()
    };

    let comp_r1 = FloorplanComponent::new("R1", 0.001, 0.001, 0.001, 0.001); // 1mm x 1mm at (1, 1) mm
    let comp_d1 = FloorplanComponent::new("D1", 0.003, 0.003, 0.001, 0.001); // 1mm x 1mm at (3, 3) mm

    let mut mesh = DynamicFloorplanMesh::new(die.clone(), 10, 10, vec![comp_r1, comp_d1]);

    let mut powers = HashMap::new();
    powers.insert("R1".to_string(), 2.0); // 2 Watts
    powers.insert("D1".to_string(), 1.5); // 1.5 Watts

    mesh.map_power_distribution(&powers);

    let cell_vol = mesh.dx() * mesh.dy() * die.thickness;
    let total_power: f64 = mesh.heat_sources.iter().sum::<f64>() * cell_vol;

    // Total dissipated power must be exactly conserved across the finite difference grid
    assert!(
        (total_power - 3.5).abs() < 1e-6,
        "Total mapped power must equal 3.5 W, got {:.6} W",
        total_power
    );

    // Cell (0, 0) is outside component footprints and must have zero heat source
    assert_eq!(mesh.heat_sources[mesh.index(0, 0)], 0.0);

    // Cells inside R1 footprint (around i=2, j=2) must have positive heat sources
    let r1_cell_idx = mesh.index(2, 2);
    assert!(mesh.heat_sources[r1_cell_idx] > 0.0);

    // Cells inside D1 footprint (around i=6, j=6) must have positive heat sources
    let d1_cell_idx = mesh.index(6, 6);
    assert!(mesh.heat_sources[d1_cell_idx] > 0.0);
}

#[test]
fn test_transient_diffusion_stability_and_thermal_spreading() {
    let die = DieProperties {
        width: 0.004,
        height: 0.004,
        thickness: 0.0002,
        ambient_temperature: 300.0,
        h_conv: 15.0,
        ..Default::default()
    };

    // Component R_heat centered on the die
    let comp = FloorplanComponent::new("R_heat", 0.0015, 0.0015, 0.001, 0.001);
    let mut mesh = DynamicFloorplanMesh::new(die, 8, 8, vec![comp]);

    let mut powers = HashMap::new();
    powers.insert("R_heat".to_string(), 4.0); // 4 Watts

    // Initial temperature must be ambient 300 K everywhere
    assert_eq!(mesh.min_temperature(), 300.0);
    assert_eq!(mesh.max_temperature(), 300.0);

    // Step transient diffusion for 20 ms with dt = 1 ms
    for _ in 0..20 {
        mesh.step_transient(0.001, &powers)
            .expect("ADI step must succeed");
    }

    let (peak_t, peak_x, peak_y) = mesh.peak_temperature();

    // Temperature must increase above ambient
    assert!(peak_t > 300.0, "Hotspot temperature must rise above 300 K");

    // Hotspot location must be centered near the heat source
    assert!((peak_x - 0.002).abs() < 0.001);
    assert!((peak_y - 0.002).abs() < 0.001);

    // Corner cell temperature must be cooler than the hotspot due to thermal spreading
    let corner_t = mesh.get_temperature(0, 0);
    assert!(
        peak_t > corner_t,
        "Center peak ({:.1} K) must exceed corner ({:.1} K)",
        peak_t,
        corner_t
    );

    // Verify unconditional numerical stability with very large time step dt = 0.1 s
    mesh.step_transient(0.1, &powers)
        .expect("Large dt ADI step must remain stable");
    let (large_dt_peak, _, _) = mesh.peak_temperature();
    assert!(large_dt_peak.is_finite());
}

#[test]
fn test_lockstep_electrothermal_cosim_with_self_heating_diode_resistor() {
    // Construct circuit: 5V DC source driving 50 Ohm resistor and diode D1 to ground
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("V1", "n_in", "0", 5.0).unwrap();
    graph.add_resistor("R1", "n_in", "n_d", 50.0).unwrap();
    graph.add_diode("D1", "n_d", "0").unwrap();

    let die = DieProperties {
        width: 0.005,
        height: 0.005,
        thickness: 0.0003,
        ambient_temperature: 300.0,
        h_conv: 20.0,
        ..Default::default()
    };

    let r1_comp = FloorplanComponent::new("R1", 0.001, 0.002, 0.001, 0.001);
    let d1_comp = FloorplanComponent::new("D1", 0.003, 0.002, 0.001, 0.001);

    let mesh = DynamicFloorplanMesh::new(die, 10, 10, vec![r1_comp, d1_comp]);

    let mut ctx = ModelContext::new();
    ctx.temperature_kelvin = 300.0;
    ctx.set_diode_model("D1", DiodeModel::default());

    let mut co_sim = ElectroThermalCoSimulator::new(graph, mesh);

    let traj = co_sim
        .simulate(0.005, 0.0005, &ctx)
        .expect("Electro-thermal co-simulation must converge");

    assert!(!traj.is_empty(), "Trajectory must contain recorded steps");
    assert!(!traj.runaway_detected, "Circuit should not trigger runaway");

    let last_rec = traj.last_record().expect("Must have last record");
    assert!(last_rec.time_s >= 0.005);

    // Verify self-heating: component temperatures must rise above initial 300 K
    let t_r1 = last_rec.component_temperatures.get("R1").copied().unwrap_or(0.0);
    let t_d1 = last_rec.component_temperatures.get("D1").copied().unwrap_or(0.0);

    assert!(t_r1 > 300.0, "Resistor R1 must exhibit Joule self-heating, got {:.2} K", t_r1);
    assert!(t_d1 > 300.0, "Diode D1 must exhibit self-heating, got {:.2} K", t_d1);

    // Verify power dissipation records
    let p_r1 = last_rec.component_powers.get("R1").copied().unwrap_or(0.0);
    let p_d1 = last_rec.component_powers.get("D1").copied().unwrap_or(0.0);
    assert!(p_r1 > 0.0, "Resistor must dissipate positive power");
    assert!(p_d1 > 0.0, "Diode must dissipate positive power");
}

#[test]
fn test_thermal_runaway_detection_under_overload() {
    // Reverse-biased diode under 100V with high leakage current
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("VREV", "n_top", "0", 100.0).unwrap();
    graph.add_resistor("R_LIM", "n_top", "n_cath", 20.0).unwrap();
    graph.add_diode("D1", "0", "n_cath").unwrap();

    let die = DieProperties {
        width: 0.002,
        height: 0.002,
        thickness: 0.0001,
        ambient_temperature: 380.0, // High initial ambient
        h_conv: 0.5,                // Extremely weak convective cooling
        ..Default::default()
    };

    let d1_comp = FloorplanComponent::new("D1", 0.0005, 0.0005, 0.001, 0.001);
    let mesh = DynamicFloorplanMesh::new(die, 6, 6, vec![d1_comp]);

    let mut ctx = ModelContext::new();
    ctx.temperature_kelvin = 380.0;
    ctx.set_diode_model(
        "D1",
        DiodeModel {
            is: 1e-7, // High leakage
            bv: 80.0, // Operating beyond breakdown -> massive Joule heat
            ..Default::default()
        },
    );

    let mut co_sim = ElectroThermalCoSimulator::new(graph, mesh)
        .with_runaway_threshold(420.0); // Safe threshold 420 K

    let result = co_sim.simulate(0.01, 0.001, &ctx);

    // Simulation must halt and return thermal runaway diagnostic error
    assert!(result.is_err(), "Thermal runaway must be detected and halt simulation");
    match result {
        Err(SolverError::NumericalAnomaly { detail }) => {
            assert!(
                detail.contains("Thermal runaway detected"),
                "Expected runaway error message, got: {detail}"
            );
        }
        other => panic!("Expected NumericalAnomaly error, got: {other:?}"),
    }

    assert!(co_sim.trajectory.runaway_detected);
}

#[test]
fn test_isothermal_contour_generation() {
    let die = DieProperties {
        width: 0.004,
        height: 0.004,
        thickness: 0.0002,
        ambient_temperature: 300.0,
        ..Default::default()
    };

    let comp = FloorplanComponent::new("Hotspot", 0.0015, 0.0015, 0.001, 0.001);
    let mut mesh = DynamicFloorplanMesh::new(die, 8, 8, vec![comp]);

    let mut powers = HashMap::new();
    powers.insert("Hotspot".to_string(), 10.0);

    // Heat up the center
    for _ in 0..10 {
        mesh.step_transient(0.001, &powers).unwrap();
    }

    let (peak_t, _, _) = mesh.peak_temperature();
    assert!(peak_t > 315.0);

    let contour_temps = vec![305.0, 310.0, 315.0];
    let contours = mesh.compute_isothermal_contours(&contour_temps);

    assert_eq!(contours.len(), 3);
    assert_eq!(contours[0].target_temperature, 305.0);
    assert_eq!(contours[1].target_temperature, 310.0);
    assert_eq!(contours[2].target_temperature, 315.0);

    // Segments must be generated for reachable temperatures
    assert!(
        !contours[0].segments.is_empty(),
        "305 K isotherm must have line segments"
    );

    // All line segment endpoints must be inside die bounds [0, 4mm]
    for c in &contours {
        for &(p1, p2) in &c.segments {
            assert!(p1.0 >= 0.0 && p1.0 <= 0.004);
            assert!(p1.1 >= 0.0 && p1.1 <= 0.004);
            assert!(p2.0 >= 0.0 && p2.0 <= 0.004);
            assert!(p2.1 >= 0.0 && p2.1 <= 0.004);
        }
    }
}
