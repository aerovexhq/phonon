//! Verification of dependent (VCVS, VCCS) and independent current sources.

use approx::assert_relative_eq;
use phonon_core::CircuitGraph;
use phonon_solver::{solve_dc_linear, SolverOptions};

#[test]
fn test_vcvs_non_inverting_amplifier() {
    let mut graph = CircuitGraph::new();

    // Input signal: 1.5 V
    graph.add_voltage_source("V_in", "in", "0", 1.5).unwrap();

    // Feedback divider: R_f = 9000 ohms, R_g = 1000 ohms -> Gain = 1 + 9000/1000 = 10.0
    graph.add_resistor("R_f", "out", "inv", 9000.0).unwrap();
    graph.add_resistor("R_g", "inv", "0", 1000.0).unwrap();

    // High-gain VCVS modeling operational amplifier: A_ol = 1e5
    // V(out) - V(0) = A_ol * (V(in) - V(inv))
    graph
        .add_vcvs("E_opamp", "out", "0", "in", "inv", 1e5)
        .unwrap();

    let sol =
        solve_dc_linear(&graph, &SolverOptions::default()).expect("OpAmp circuit must converge");

    let v_out = sol.node_voltage_by_name(&graph, "out").unwrap();
    let v_inv = sol.node_voltage_by_name(&graph, "inv").unwrap();

    // With A_ol = 1e5, ideal gain is 10.0, actual gain is 10.0 / (1 + 10/1e5) ~= 9.9990
    assert_relative_eq!(v_out, 15.0, max_relative = 0.001);
    // Virtual short at opamp inputs: V(inv) ~= V(in) = 1.5 V
    assert_relative_eq!(v_inv, 1.5, max_relative = 0.001);
}

#[test]
fn test_vccs_transconductance_amplifier() {
    let mut graph = CircuitGraph::new();

    // V_ctrl = 2.0 V
    graph
        .add_voltage_source("V_ctrl", "gate", "0", 2.0)
        .unwrap();

    // VCCS: gm = 0.025 S (25 mA/V), pumping into node 'drain'
    graph
        .add_vccs("G_fet", "0", "drain", "gate", "0", 0.025)
        .unwrap();

    // Load resistor: R_load = 200 ohms
    graph.add_resistor("R_load", "drain", "0", 200.0).unwrap();

    let sol =
        solve_dc_linear(&graph, &SolverOptions::default()).expect("VCCS circuit must converge");

    let v_drain = sol.node_voltage_by_name(&graph, "drain").unwrap();

    // Expected: I_drain = gm * V_gate = 0.025 * 2.0 = 0.05 A
    // V_drain = I_drain * R_load = 0.05 * 200 = 10.0 V
    assert_relative_eq!(v_drain, 10.0, epsilon = 1e-11);
}

#[test]
fn test_independent_current_source() {
    let mut graph = CircuitGraph::new();

    // Current source: 4 mA into node 'top'
    graph.add_current_source("I1", "0", "top", 0.004).unwrap();
    // Resistor: 2500 ohms
    graph.add_resistor("R1", "top", "0", 2500.0).unwrap();

    let sol = solve_dc_linear(&graph, &SolverOptions::default())
        .expect("Current source circuit must converge");

    let v_top = sol.node_voltage_by_name(&graph, "top").unwrap();

    // Expected: V = I * R = 0.004 * 2500 = 10.0 V
    assert_relative_eq!(v_top, 10.0, epsilon = 1e-12);
}
