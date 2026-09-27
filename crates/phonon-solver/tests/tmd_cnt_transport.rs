//! Integration tests for 2D TMD monolayers (MoS2, WS2) and Carbon Nanotubes (CNTs)
//! transport, MNA companion solver simulation, and KCL conservation.

use phonon_core::{AtomisticChannelType, CircuitGraph};
use phonon_models::{CarbonNanotube, CntCharacter, TmdMonolayer, QUANTUM_CONDUCTANCE_SI};
use phonon_solver::mna::{solve_dc_non_linear, ModelContext, NewtonOptions};
use phonon_solver::verification::verify_kcl_dynamic;
use std::collections::HashMap;

#[test]
fn test_tmd_monolayer_physics_and_spin_orbit() {
    let mos2 = TmdMonolayer::mos2();
    let ws2 = TmdMonolayer::ws2();

    // Bandgaps are direct at K valley
    assert!((mos2.bandgap_ev - 1.82).abs() < 1e-4);
    assert!((ws2.bandgap_ev - 1.98).abs() < 1e-4);

    // Heavier W atoms give much stronger spin-orbit splitting in WS2
    assert!(ws2.spin_orbit_splitting_ev > 0.40);
    assert!(mos2.spin_orbit_splitting_ev < 0.20);

    // Energy dispersions at K point
    let e_v_up = ws2.valence_dispersion_ev(0.0, true);
    let e_v_down = ws2.valence_dispersion_ev(0.0, false);
    assert!((e_v_up - e_v_down - ws2.spin_orbit_splitting_ev).abs() < 1e-5);
}

#[test]
fn test_cnt_chirality_and_quantum_conductance() {
    // (10, 10) armchair is metallic
    let cnt_metallic = CarbonNanotube::armchair(10);
    assert_eq!(cnt_metallic.character, CntCharacter::Metallic);
    assert_eq!(cnt_metallic.bandgap_ev, 0.0);

    // (10, 0) zigzag is semiconducting with ~1.1 eV bandgap
    let cnt_semi = CarbonNanotube::zigzag(10);
    assert_eq!(cnt_semi.character, CntCharacter::Semiconducting);
    assert!(cnt_semi.bandgap_ev > 0.8 && cnt_semi.bandgap_ev < 1.4);

    // Metallic tube ballistic low-field conductance: G = 2 * G0 = 4 e^2 / h
    let v_ds = 0.01; // 10 mV
    let i_metal = cnt_metallic.evaluate_fet_current(1.0, v_ds, 0.0, 50e-9, 300.0);
    let g_sim = i_metal / v_ds;
    let g_theory = 2.0 * QUANTUM_CONDUCTANCE_SI;
    // With quasi-ballistic transmission factor ~ 0.95:
    assert!(
        g_sim > 0.85 * g_theory && g_sim <= g_theory,
        "Simulated conductance {} should be close to 2 * G0 = {}",
        g_sim,
        g_theory
    );
}

#[test]
fn test_tmd_fet_mna_circuit_simulation_and_kcl() {
    // Build circuit:
    // Node 0: GND
    // Node 1: VDD (+1.0 V)
    // Node 2: Gate bias (+1.2 V)
    // Node 3: TMD FET Drain, connected to VDD through 10 kOhm load resistor
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("V1", "1", "0", 1.0).unwrap();
    graph.add_voltage_source("Vgate", "2", "0", 1.2).unwrap();
    graph.add_resistor("Rload", "1", "3", 10_000.0).unwrap();

    // TMD Monolayer channel (Drain: 3, Gate: 2, Source: 0)
    let channel_type = AtomisticChannelType::TmdMonolayer {
        species: "MoS2".to_string(),
        length_m: 30e-9,
        width_m: 1e-6,
    };
    graph
        .add_atomistic_channel("X_TMD", "3", "2", "0", channel_type)
        .unwrap();

    let mut context = ModelContext::new();
    context.set_atomistic_tmd("X_TMD", TmdMonolayer::mos2());

    let newton_opts = NewtonOptions {
        reltol: 1e-4,
        vntol: 1e-6,
        abstol: 1e-12,
        max_iters: 100,
        enable_gmin_stepping: true,
        enable_source_stepping: true,
    };

    let solution = solve_dc_non_linear(&graph, &context, &newton_opts)
        .expect("TMD FET DC operating point must converge");

    let v_drain = solution.node_voltages[3];
    assert!(
        v_drain > 0.0 && v_drain < 1.0,
        "Drain node must settle within (0.0, 1.0) V: got {}",
        v_drain
    );

    // Verify rigorous Kirchhoff's Current Law at all nodes
    let empty_caps = HashMap::new();
    let kcl = verify_kcl_dynamic(
        &graph,
        &solution.node_voltages,
        &solution.branch_currents,
        &empty_caps,
        Some(&context),
        1e-3,
        1e-9,
    );

    assert!(
        kcl.is_valid,
        "KCL must be satisfied with TMD FET: max_residual={}",
        kcl.max_residual
    );
}

#[test]
fn test_cnt_fet_mna_circuit_simulation_and_kcl() {
    // Build CNT FET inverter / pull-down stage:
    // Node 0: GND
    // Node 1: VDD (+0.8 V)
    // Node 2: Gate bias (+0.8 V ON)
    // Node 3: CNT Drain, loaded by 50 kOhm resistor
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("Vdd", "1", "0", 0.8).unwrap();
    graph.add_voltage_source("Vin", "2", "0", 0.8).unwrap();
    graph.add_resistor("Rpullup", "1", "3", 50_000.0).unwrap();

    let channel_type = AtomisticChannelType::CarbonNanotube {
        n: 10,
        m: 0,
        length_m: 50e-9,
    };
    graph
        .add_atomistic_channel("X_CNT", "3", "2", "0", channel_type)
        .unwrap();

    let mut context = ModelContext::new();
    context.set_atomistic_cnt("X_CNT", CarbonNanotube::zigzag(10));

    let newton_opts = NewtonOptions::default();

    let solution = solve_dc_non_linear(&graph, &context, &newton_opts)
        .expect("CNT FET DC operating point must converge");

    let v_out = solution.node_voltages[3];
    assert!(
        v_out < 0.8,
        "CNT FET pulled output below VDD: got {}",
        v_out
    );

    let empty_caps = HashMap::new();
    let kcl = verify_kcl_dynamic(
        &graph,
        &solution.node_voltages,
        &solution.branch_currents,
        &empty_caps,
        Some(&context),
        1e-3,
        1e-9,
    );

    assert!(
        kcl.is_valid,
        "KCL must be strictly satisfied in CNT circuit: max_residual={}",
        kcl.max_residual
    );
}
