#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for programmable chiral phonon networks across multi-threaded Rayon workers.

use phonon_solver::programmable_chiral_graph::GraphBenchmarkRunner;

#[test]
fn test_10k_programmable_chiral_graph_parallel_sweep() {
    let cycles = 10_000;
    let result = GraphBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 120 Programmable Chiral Graph Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!("Physical Compliance Fraction: {:.2}%", result.physical_compliance_fraction * 100.0);
    println!("Graph Entanglement Fidelity: mean = {:.4}, min = {:.4}", result.mean_graph_entanglement_fidelity, result.min_graph_entanglement_fidelity);
    println!("Topological Edge Purity: mean = {:.4}, min = {:.4}", result.mean_topological_edge_purity, result.min_topological_edge_purity);
    println!("Network Nodes Count: mean = {:.1}, min = {}", result.mean_network_nodes_count, result.min_network_nodes_count);
    println!("Switching Time (ns): mean = {:.2}, max = {:.2}", result.mean_switching_time_ns, result.max_switching_time_ns);
    println!("Nullifier Variance (dB): mean = {:.2}, max = {:.2}", result.mean_nullifier_variance_db, result.max_nullifier_variance_db);
    println!("Stabilizer Generator Fidelity: mean = {:.4}, min = {:.4}", result.mean_stabilizer_generator_fidelity, result.min_stabilizer_generator_fidelity);

    assert_eq!(
        result.total_cycles, cycles,
        "Total benchmark cycles must equal requested 10,000"
    );

    // Verify 100% physical compliance across all parameter sweep variations
    assert!(
        (result.physical_compliance_fraction - 1.0).abs() < 1.0e-6,
        "Physical compliance fraction must be 100.0%, got {:.2}%",
        result.physical_compliance_fraction * 100.0
    );

    // Assert multi-partite graph entanglement fidelity >= 94.0%
    assert!(
        result.mean_graph_entanglement_fidelity >= 0.940,
        "Mean graph entanglement fidelity must be >= 0.940, got {:.4}",
        result.mean_graph_entanglement_fidelity
    );
    assert!(
        result.min_graph_entanglement_fidelity >= 0.940,
        "Worst-case graph entanglement fidelity must be >= 0.940, got {:.4}",
        result.min_graph_entanglement_fidelity
    );

    // Assert topological edge purity >= 96.0%
    assert!(
        result.mean_topological_edge_purity >= 0.960,
        "Mean topological edge purity must be >= 0.960, got {:.4}",
        result.mean_topological_edge_purity
    );
    assert!(
        result.min_topological_edge_purity >= 0.960,
        "Worst-case topological edge purity must be >= 0.960, got {:.4}",
        result.min_topological_edge_purity
    );

    // Assert network nodes count >= 64
    assert!(
        result.mean_network_nodes_count >= 64.0,
        "Mean network nodes count must be >= 64, got {:.1}",
        result.mean_network_nodes_count
    );
    assert!(
        result.min_network_nodes_count >= 64,
        "Worst-case network nodes count must be >= 64, got {}",
        result.min_network_nodes_count
    );

    // Assert switching time <= 20.0 ns
    assert!(
        result.mean_switching_time_ns <= 20.0,
        "Mean switching time must be <= 20.0 ns, got {:.2} ns",
        result.mean_switching_time_ns
    );
    assert!(
        result.max_switching_time_ns <= 20.0,
        "Worst-case switching time must be <= 20.0 ns, got {:.2} ns",
        result.max_switching_time_ns
    );

    // Assert nullifier variance <= -4.5 dB
    assert!(
        result.mean_nullifier_variance_db <= -4.5,
        "Mean nullifier variance must be <= -4.5 dB, got {:.2} dB",
        result.mean_nullifier_variance_db
    );
    assert!(
        result.max_nullifier_variance_db <= -4.5,
        "Worst-case nullifier variance must be <= -4.5 dB, got {:.2} dB",
        result.max_nullifier_variance_db
    );

    // Assert stabilizer generator fidelity >= 95.0%
    assert!(
        result.mean_stabilizer_generator_fidelity >= 0.950,
        "Mean stabilizer generator fidelity must be >= 0.950, got {:.4}",
        result.mean_stabilizer_generator_fidelity
    );
    assert!(
        result.min_stabilizer_generator_fidelity >= 0.950,
        "Worst-case stabilizer generator fidelity must be >= 0.950, got {:.4}",
        result.min_stabilizer_generator_fidelity
    );

    assert!(
        result.throughput_sweeps_per_sec > 0.0,
        "Throughput must be positive, got {:.2} sweeps/sec",
        result.throughput_sweeps_per_sec
    );
}
