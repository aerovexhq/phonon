use phonon_core::CircuitGraph;
use phonon_models::diode::DiodeModel;
use phonon_solver::mna::{ModelContext, NewtonOptions};
use phonon_solver::SolverError;
use phonon_thermal::cauer::CauerNetwork;
use phonon_thermal::monolithic::{solve_electrothermal_dc, ElectroThermalBinding};
use phonon_thermal::runaway::assess_diode_thermal_stability;

#[test]
fn test_diode_stability_criterion() {
    let diode_model = DiodeModel {
        is: 1e-9, // 1 nA leakage
        bv: 200.0,
        ..Default::default()
    };

    // Small thermal resistance (R_th = 5 K/W) under reverse bias (-50 V) -> Stable
    let assessment_stable = assess_diode_thermal_stability(-50.0, 5.0, &diode_model, 300.0);
    assert!(
        assessment_stable.is_stable,
        "Low thermal resistance must be thermally stable, got loop_gain={}",
        assessment_stable.thermal_loop_gain
    );
    assert!(assessment_stable.thermal_loop_gain < 1.0);

    // Extreme thermal resistance (R_th = 500_000 K/W) under reverse bias (-150 V) -> Unstable
    let assessment_unstable =
        assess_diode_thermal_stability(-150.0, 500_000.0, &diode_model, 380.0);
    assert!(
        !assessment_unstable.is_stable,
        "High thermal resistance with high reverse voltage must predict runaway"
    );
    assert!(assessment_unstable.thermal_loop_gain > 1.0);
}

#[test]
fn test_diode_thermal_runaway_detection() {
    // Construct a circuit designed to trigger thermal runaway:
    // High reverse voltage (120 V), huge thermal resistance (R_th = 2000 K/W),
    // and elevated initial ambient temperature (350 K).
    let mut graph = CircuitGraph::new();
    // Diode reverse-biased: anode at 0, cathode at "n_cath"
    graph
        .add_voltage_source("VREV", "n_in", "0", 120.0)
        .unwrap();
    graph.add_resistor("R1", "n_in", "n_cath", 100.0).unwrap();
    graph.add_diode("D1", "0", "n_cath").unwrap();

    let diode_model = DiodeModel {
        is: 1e-8, // 10 nA leakage
        bv: 150.0,
        ..Default::default()
    };

    let ambient_k = 360.0;
    let mut ctx = ModelContext::new();
    ctx.temperature_kelvin = ambient_k;
    ctx.set_diode_model("D1", diode_model);

    let mut cauer = CauerNetwork::new();
    cauer.add_stage("Die_Ambient", 5_000.0, 1e-3); // 5000 K/W -> extreme runaway

    let binding = ElectroThermalBinding {
        component_name: "D1".to_string(),
        cauer,
        ambient_k,
    };

    let newton_opts = NewtonOptions::default();
    let res = solve_electrothermal_dc(&graph, &[binding], &ctx, &newton_opts, 30, 1e-3);

    // The solver must detect thermal runaway and return a clean structured error
    assert!(res.is_err(), "Thermal runaway must be detected");
    match res {
        Err(SolverError::NumericalAnomaly { detail }) => {
            assert!(
                detail.contains("Thermal runaway detected"),
                "Expected runaway diagnostic, got: {detail}"
            );
        }
        other => panic!("Expected NumericalAnomaly with runaway diagnostic, got: {other:?}"),
    }
}
