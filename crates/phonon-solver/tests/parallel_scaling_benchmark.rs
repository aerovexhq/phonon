//! Integration Benchmark & Test: Parallel Scaling and Kron's Diakoptics.
//!
//! Validates:
//! - Multi-partition Diakoptics on a large ladder network (200 nodes, 4 partitions)
//! - Exact algebraic equivalence to Monolithic MNA across all 200 nodes (< 1e-11 V)
//! - Multi-threaded Rayon execution consistency and scalability across thread pools
//! - Parallel SoA device batch evaluation accuracy against scalar models.

use phonon_core::constants::T_REF;
use phonon_core::CircuitGraph;
use phonon_solver::mna::{solve_dc_linear, ModelContext, SolverOptions};
use phonon_solver::parallel::{CircuitSoA, DiakopticsSolver, PartitionedCircuit};
use std::time::Instant;

#[test]
fn test_diakoptics_scaling_large_ladder_network() {
    let mut g = CircuitGraph::new();
    let num_nodes = 200;

    // Drive node 1 with a 10.0V source
    g.add_voltage_source("V_in", "n_1", "0", 10.0).unwrap();

    // Build resistive ladder: series resistors n_i -> n_{i+1} and shunt resistors n_i -> 0
    for i in 1..num_nodes {
        let n_curr = format!("n_{}", i);
        let n_next = format!("n_{}", i + 1);
        let r_name = format!("R_ser_{}", i);
        g.add_resistor(&r_name, &n_curr, &n_next, 10.0).unwrap();
    }

    for i in 1..=num_nodes {
        let n_curr = format!("n_{}", i);
        let r_name = format!("R_sh_{}", i);
        g.add_resistor(&r_name, &n_curr, "0", 100.0).unwrap();
    }

    // Partition into 4 subcircuits by tearing nodes 51, 101, 151
    let n51 = g.get_node("n_51").unwrap();
    let n101 = g.get_node("n_101").unwrap();
    let n151 = g.get_node("n_151").unwrap();

    let partitioned =
        PartitionedCircuit::from_graph_and_torn_nodes(g.clone(), vec![n51, n101, n151]);

    assert_eq!(
        partitioned.subcircuit_count(),
        4,
        "Expected 4 subcircuits from 3 torn nodes on 200-node ladder"
    );
    assert_eq!(
        partitioned.torn_variable_count(),
        3,
        "Expected 3 torn boundary variables"
    );

    let opts = SolverOptions::default();

    // 1. Monolithic solve
    let start_mono = Instant::now();
    let mono_sol = solve_dc_linear(&g, &opts).expect("Monolithic solve failed");
    let dur_mono = start_mono.elapsed();

    // 2. Multi-threaded Diakoptics solve (Global Rayon threadpool)
    let start_diak = Instant::now();
    let diak_sol =
        DiakopticsSolver::solve_linear(&partitioned, &opts).expect("Diakoptics solve failed");
    let dur_diak = start_diak.elapsed();

    println!(
        "200-Node Ladder Network (4 partitions):\n  Monolithic MNA: {:?}\n  Parallel Diakoptics: {:?}",
        dur_mono, dur_diak
    );

    // 3. Verify Exact Equivalence Across All 200 Nodes
    let mut max_diff = 0.0f64;
    for i in 1..=num_nodes {
        let node = g.get_node(&format!("n_{}", i)).unwrap();
        let v_mono = mono_sol.node_voltage(node);
        let v_diak = diak_sol.node_voltage(node);
        let diff = (v_diak - v_mono).abs();
        if diff > max_diff {
            max_diff = diff;
        }
        assert!(
            diff < 1e-11,
            "Node n_{} mismatch: Diakoptics = {:.10} V, Monolithic = {:.10} V, diff = {:.3e}",
            i,
            v_diak,
            v_mono,
            diff
        );
    }
    println!(
        "Max voltage discrepancy across all 200 nodes: {:.3e} V (PASSED < 1e-11 V)",
        max_diff
    );

    // 4. Thread Scaling Verification (1 thread vs 4 threads)
    let pool_1 = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap();
    let pool_4 = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .unwrap();

    let iters = 20;
    let t1_start = Instant::now();
    let sol_1 = pool_1.install(|| {
        let mut last_sol = None;
        for _ in 0..iters {
            last_sol = Some(DiakopticsSolver::solve_linear(&partitioned, &opts).unwrap());
        }
        last_sol.unwrap()
    });
    let dur_1 = t1_start.elapsed();

    let t4_start = Instant::now();
    let sol_4 = pool_4.install(|| {
        let mut last_sol = None;
        for _ in 0..iters {
            last_sol = Some(DiakopticsSolver::solve_linear(&partitioned, &opts).unwrap());
        }
        last_sol.unwrap()
    });
    let dur_4 = t4_start.elapsed();

    println!(
        "Scaling ({} iters):\n  1 Worker Thread:  {:?}\n  4 Worker Threads: {:?}",
        iters, dur_1, dur_4
    );

    // Verify consistency between 1-thread and 4-thread runs
    for i in 1..=num_nodes {
        let node = g.get_node(&format!("n_{}", i)).unwrap();
        let v1 = sol_1.node_voltage(node);
        let v4 = sol_4.node_voltage(node);
        assert!(
            (v1 - v4).abs() < 1e-14,
            "Thread pool result divergence at n_{}: v1={:.10}, v4={:.10}",
            i,
            v1,
            v4
        );
    }
}

#[test]
fn test_circuit_soa_parallel_evaluation() {
    let mut g = CircuitGraph::new();
    let n_diodes = 2000;

    // Create a network with 2,000 diodes connected between sequential nodes and ground
    for i in 1..=n_diodes {
        let n_anode = format!("d_anode_{}", i);
        let d_name = format!("D_{}", i);
        g.add_diode(&d_name, &n_anode, "0").unwrap();
    }

    let soa = CircuitSoA::from_circuit_graph(&g);
    assert_eq!(soa.diodes.len(), n_diodes);

    let context = ModelContext::default();
    let temp_k = T_REF;

    // Generate a test voltage vector
    let mut x = vec![0.0; g.active_nodes()];
    for (i, val) in x.iter_mut().enumerate() {
        *val = 0.5 + ((i % 50) as f64) * 0.005; // 0.5V to 0.745V forward bias
    }

    // Parallel evaluation via Rayon
    let par_eval = soa.diodes.evaluate_parallel(&x, &context, temp_k);
    assert_eq!(par_eval.len(), n_diodes);

    // Sequential verification
    for (i, (pos, neg, gd_par, ieq_par)) in par_eval.into_iter().enumerate() {
        let name = &soa.diodes.name[i];
        assert_eq!(pos, soa.diodes.pos[i]);
        assert_eq!(neg, soa.diodes.neg[i]);

        let vd = if pos.is_ground() {
            0.0
        } else {
            x[pos.index() - 1]
        };
        let model = context.get_diode_model(name);
        let eval = model.evaluate(vd, temp_k);
        let ieq_expected = eval.g_d * vd - eval.i_d;

        let diff_gd = (gd_par - eval.g_d).abs();
        let diff_ieq = (ieq_par - ieq_expected).abs();

        assert!(
            diff_gd < 1e-12,
            "Diode {} gd mismatch: par={:.8e}, seq={:.8e}",
            name,
            gd_par,
            eval.g_d
        );
        assert!(
            diff_ieq < 1e-12,
            "Diode {} ieq mismatch: par={:.8e}, seq={:.8e}",
            name,
            ieq_par,
            ieq_expected
        );
    }
}
