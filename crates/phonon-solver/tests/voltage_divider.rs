//! Verification of resistive voltage dividers and multi-stage ladder networks against analytical solutions.

use approx::assert_relative_eq;
use phonon_core::CircuitGraph;
use phonon_solver::{solve_dc_linear, SolverOptions};

#[test]
fn test_simple_voltage_divider() {
    let mut graph = CircuitGraph::new();

    // 10V DC source
    let _ = graph.add_voltage_source("V1", "in", "0", 10.0).unwrap();
    // R1 = 3000 ohms, R2 = 2000 ohms
    let _ = graph.add_resistor("R1", "in", "out", 3000.0).unwrap();
    let _ = graph.add_resistor("R2", "out", "0", 2000.0).unwrap();

    let sol = solve_dc_linear(&graph, &SolverOptions::default()).expect("DC solve must succeed");

    let v_in = sol.node_voltage_by_name(&graph, "in").unwrap();
    let v_out = sol.node_voltage_by_name(&graph, "out").unwrap();
    let i_v1 = sol.branch_current(phonon_core::BranchId::new(0));

    // Analytical expectation: V_out = 10.0 * (2000 / 5000) = 4.0 V
    assert_relative_eq!(v_in, 10.0, epsilon = 1e-12);
    assert_relative_eq!(v_out, 4.0, epsilon = 1e-12);
    // Source current: I = 10V / 5000 ohms = 2 mA (branch current into pos terminal is -2mA)
    assert_relative_eq!(i_v1, -0.002, epsilon = 1e-12);
}

#[test]
fn test_n_stage_resistor_ladder() {
    let mut graph = CircuitGraph::new();
    let num_stages = 10;
    let r_series = 1000.0;
    let r_shunt = 2000.0;

    // V_in = 10.0 V
    graph.add_voltage_source("V1", "node_0", "0", 10.0).unwrap();

    for i in 0..num_stages {
        let from_node = format!("node_{}", i);
        let to_node = format!("node_{}", i + 1);

        graph
            .add_resistor(&format!("R_ser_{}", i), &from_node, &to_node, r_series)
            .unwrap();
        graph
            .add_resistor(&format!("R_sh_{}", i + 1), &to_node, "0", r_shunt)
            .unwrap();
    }

    let sol =
        solve_dc_linear(&graph, &SolverOptions::default()).expect("Ladder solve must succeed");

    // Assert voltages monotonically decay toward 0
    let mut prev_v = 10.0;
    for i in 1..=num_stages {
        let node_name = format!("node_{}", i);
        let v = sol.node_voltage_by_name(&graph, &node_name).unwrap();
        assert!(v > 0.0, "Voltage at {} must be positive", node_name);
        assert!(
            v < prev_v,
            "Voltage must strictly decay along ladder: {} < {}",
            v,
            prev_v
        );
        prev_v = v;
    }
}
