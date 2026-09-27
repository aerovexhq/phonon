//! Verification of automated singularity diagnostics, floating node detection, and loop checks.

use phonon_core::{CircuitGraph, CoreError};
use phonon_solver::{solve_dc_linear, SolverError, SolverOptions};

#[test]
fn test_floating_node_detection_and_remediation() {
    let mut graph = CircuitGraph::new();

    // Valid part of circuit
    graph.add_voltage_source("V1", "in", "0", 5.0).unwrap();
    graph.add_resistor("R1", "in", "0", 1000.0).unwrap();

    // Isolated floating node 'float_pin'
    let _ = graph.get_or_create_node("float_pin");

    // Strict mode: must report floating node error
    let result_strict = solve_dc_linear(&graph, &SolverOptions::default());
    assert!(
        matches!(result_strict, Err(SolverError::Core(CoreError::FloatingNode { ref name, .. })) if name == "float_pin"),
        "Expected FloatingNode error for 'float_pin', got: {:?}",
        result_strict
    );
}

#[test]
fn test_loop_of_voltage_sources() {
    let mut graph = CircuitGraph::new();

    // Two voltage sources in parallel between 'v_bus' and ground
    graph.add_voltage_source("V1", "v_bus", "0", 5.0).unwrap();
    graph.add_voltage_source("V2", "v_bus", "0", 3.3).unwrap();

    let result = solve_dc_linear(&graph, &SolverOptions::default());
    assert!(
        matches!(
            result,
            Err(SolverError::Core(CoreError::VoltageSourceLoop { .. }))
        ),
        "Expected VoltageSourceLoop error, got: {:?}",
        result
    );
}

#[test]
fn test_disconnected_subnetwork_singularity_diagnosis() {
    let mut graph = CircuitGraph::new();

    // Ground-connected circuit
    graph.add_voltage_source("V1", "n1", "0", 10.0).unwrap();
    graph.add_resistor("R1", "n1", "0", 100.0).unwrap();

    // Subnetwork connected together, but completely floating from ground:
    // n2 and n3 share R2, but have no path to 0
    graph.add_resistor("R2", "n2", "n3", 500.0).unwrap();
    graph.add_current_source("I1", "n2", "n3", 0.01).unwrap();

    // Solve with strict options
    let result = solve_dc_linear(&graph, &SolverOptions::default());
    match result {
        Err(SolverError::SingularMatrix {
            ref entity_diagnostic,
            ..
        }) => {
            // Must pinpoint either n2 or n3
            assert!(
                entity_diagnostic.contains("n2") || entity_diagnostic.contains("n3"),
                "Diagnostic should identify offending node, got: {}",
                entity_diagnostic
            );
        }
        other => panic!("Expected SingularMatrix with diagnostic, got: {:?}", other),
    }

    // Auto-gmin shunt remediation mode: solves successfully!
    let options_gmin = SolverOptions {
        auto_gmin_shunt: true,
        gmin_value: 1e-12,
    };
    let sol_remediated = solve_dc_linear(&graph, &options_gmin);
    assert!(
        sol_remediated.is_ok(),
        "Gmin remediation should solve floating subnetwork"
    );
}
