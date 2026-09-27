//! Verification of balanced and unbalanced Wheatstone bridge circuits against exact analytical Thevenin equations.

use approx::assert_relative_eq;
use phonon_core::CircuitGraph;
use phonon_solver::{solve_dc_linear, SolverOptions};

#[test]
fn test_balanced_wheatstone_bridge() {
    let mut graph = CircuitGraph::new();

    // Excitation: 12.0 V
    graph.add_voltage_source("V_exc", "top", "0", 12.0).unwrap();

    // Balanced ratio: R1 / R3 = R2 / R4 = 100 / 200
    graph.add_resistor("R1", "top", "node_a", 100.0).unwrap();
    graph.add_resistor("R3", "node_a", "0", 200.0).unwrap();

    graph.add_resistor("R2", "top", "node_b", 100.0).unwrap();
    graph.add_resistor("R4", "node_b", "0", 200.0).unwrap();

    // Detector bridge resistor
    graph
        .add_resistor("R_det", "node_a", "node_b", 50.0)
        .unwrap();

    let sol =
        solve_dc_linear(&graph, &SolverOptions::default()).expect("Bridge solve must succeed");

    let v_a = sol.node_voltage_by_name(&graph, "node_a").unwrap();
    let v_b = sol.node_voltage_by_name(&graph, "node_b").unwrap();

    // Expected: V_a = V_b = 12 * (200 / 300) = 8.0 V
    assert_relative_eq!(v_a, 8.0, epsilon = 1e-12);
    assert_relative_eq!(v_b, 8.0, epsilon = 1e-12);
    assert_relative_eq!(v_a - v_b, 0.0, epsilon = 1e-12);
}

#[test]
fn test_unbalanced_wheatstone_bridge_thevenin() {
    let mut graph = CircuitGraph::new();

    let v_in = 10.0;
    let r1 = 1000.0;
    let r2 = 1000.0;
    let r3 = 1000.0;
    let r4 = 1200.0; // 20% imbalance
    let r_bridge = 500.0;

    graph.add_voltage_source("V1", "top", "0", v_in).unwrap();
    graph.add_resistor("R1", "top", "node_a", r1).unwrap();
    graph.add_resistor("R3", "node_a", "0", r3).unwrap();

    graph.add_resistor("R2", "top", "node_b", r2).unwrap();
    graph.add_resistor("R4", "node_b", "0", r4).unwrap();

    graph
        .add_resistor("R_det", "node_a", "node_b", r_bridge)
        .unwrap();

    let sol = solve_dc_linear(&graph, &SolverOptions::default())
        .expect("Unbalanced bridge solve must succeed");

    let v_a = sol.node_voltage_by_name(&graph, "node_a").unwrap();
    let v_b = sol.node_voltage_by_name(&graph, "node_b").unwrap();
    let v_det = v_a - v_b;

    // Thevenin equivalent:
    // V_th = V_in * (R3 / (R1 + R3) - R4 / (R2 + R4))
    let v_th = v_in * (r3 / (r1 + r3) - r4 / (r2 + r4));
    // R_th = (R1 || R3) + (R2 || R4)
    let r_th = (r1 * r3) / (r1 + r3) + (r2 * r4) / (r2 + r4);
    // Expected detector voltage: V_det = V_th * (R_bridge / (R_th + R_bridge))
    let expected_v_det = v_th * (r_bridge / (r_th + r_bridge));

    assert_relative_eq!(v_det, expected_v_det, epsilon = 1e-11);
}
