//! Integration Test: Kron's Diakoptics / Node Tearing Circuit Decomposition.
//!
//! Validates:
//! - Multi-threaded decomposition of an interconnected circuit into independent subcircuits
//! - Exact algebraic equivalence between Diakoptics Schur complement condensation and monolithic MNA
//! - Verified to machine precision (< 1e-11 V error) across all internal and boundary nodes.

use phonon_core::CircuitGraph;
use phonon_solver::mna::{solve_dc_linear, SolverOptions};
use phonon_solver::parallel::{DiakopticsSolver, PartitionedCircuit};

#[test]
fn test_diakoptics_exact_equivalence_to_monolithic_mna() {
    let mut g = CircuitGraph::new();
    let _n_in1 = g.get_or_create_node("in1");
    let _n_b1 = g.get_or_create_node("b1");
    let n_t1 = g.get_or_create_node("t1");
    let _n_b2 = g.get_or_create_node("b2");
    let _n_out2 = g.get_or_create_node("out2");

    g.add_voltage_source("V1", "in1", "0", 12.0).unwrap();
    g.add_resistor("R1a", "in1", "b1", 100.0).unwrap();
    g.add_resistor("R1b", "b1", "0", 200.0).unwrap();
    g.add_resistor("R_tie1", "b1", "t1", 25.0).unwrap();
    g.add_resistor("R_tie2", "b2", "t1", 35.0).unwrap();
    g.add_resistor("R_shunt", "t1", "0", 400.0).unwrap();
    g.add_resistor("R2a", "b2", "out2", 50.0).unwrap();
    g.add_resistor("R2b", "out2", "0", 150.0).unwrap();

    // 1. Automatic partitioning using torn node "t1"
    let partitioned = PartitionedCircuit::from_graph_and_torn_nodes(g.clone(), vec![n_t1]);
    assert_eq!(
        partitioned.subcircuit_count(),
        2,
        "Expected 2 disconnected subcircuits when t1 is torn"
    );

    // 2. Multi-threaded Diakoptics solve
    let diak_sol = DiakopticsSolver::solve_linear(&partitioned, &SolverOptions::default())
        .expect("Diakoptics solve failed");

    // 3. Monolithic MNA solve
    let mono_sol =
        solve_dc_linear(&g, &SolverOptions::default()).expect("Monolithic MNA solve failed");

    // 4. Assert Exact Equivalence Across All Named Nodes
    let check_nodes = ["in1", "b1", "t1", "b2", "out2"];
    for &name in &check_nodes {
        let node = g.get_node(name).unwrap();
        let mono_v = mono_sol.node_voltage(node);
        let diak_v = diak_sol.node_voltage(node);

        println!(
            "Node {}: Monolithic = {:.8} V, Diakoptics = {:.8} V",
            name, mono_v, diak_v
        );
        let diff = (diak_v - mono_v).abs();
        assert!(
            diff < 1e-11,
            "Mismatch at node '{}': Diakoptics={:.8}V, Monolithic={:.8}V, diff={:.3e}",
            name,
            diak_v,
            mono_v,
            diff
        );
    }

    // Also assert exact branch current equivalence
    let br_v1 = g
        .components()
        .iter()
        .find_map(|c| match c {
            phonon_core::ComponentRecord::VoltageSource { name, branch, .. } if name == "V1" => {
                Some(*branch)
            }
            _ => None,
        })
        .unwrap();

    let mono_i = mono_sol.branch_current(br_v1);
    let diak_i = diak_sol.branch_current(br_v1);
    println!(
        "Branch I(V1): Monolithic = {:.8} A, Diakoptics = {:.8} A",
        mono_i, diak_i
    );
    assert!(
        (diak_i - mono_i).abs() < 1e-11,
        "Mismatch at branch I(V1): Diakoptics={:.8}A, Monolithic={:.8}A",
        diak_i,
        mono_i
    );
}
