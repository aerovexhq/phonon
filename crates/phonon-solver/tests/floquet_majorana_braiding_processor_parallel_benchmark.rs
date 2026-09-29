#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for topological phononic Floquet-Majorana braiding processors and non-Abelian
//! topological logic across multi-threaded Rayon workers.

use phonon_solver::floquet_majorana_braiding_processor::BraidingProcessorBenchmarkRunner;

#[test]
fn test_10k_floquet_majorana_braiding_processor_parallel_sweep() {
    let cycles = 10_000;
    let result = BraidingProcessorBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 127 Topological Phononic Floquet-Majorana Braiding Processors Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Braiding Gate Fidelity: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_braiding_gate_fidelity,
        result.min_braiding_gate_fidelity,
        result.max_braiding_gate_fidelity
    );
    println!(
        "Topological Protection Gap (MHz): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_topological_protection_gap_mhz,
        result.min_topological_protection_gap_mhz,
        result.max_topological_protection_gap_mhz
    );
    println!(
        "Operation Latency (ns): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_operation_latency_ns,
        result.min_operation_latency_ns,
        result.max_operation_latency_ns
    );
    println!(
        "Edge State Isolation (dB): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_edge_state_isolation_db,
        result.min_edge_state_isolation_db,
        result.max_edge_state_isolation_db
    );
    println!(
        "Non-Abelian State Purity: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_non_abelian_state_purity,
        result.min_non_abelian_state_purity,
        result.max_non_abelian_state_purity
    );

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

    // Assert braiding gate fidelity >= 0.9980
    assert!(
        result.mean_braiding_gate_fidelity >= 0.9980,
        "Mean braiding gate fidelity must be >= 0.9980, got {:.5}",
        result.mean_braiding_gate_fidelity
    );
    assert!(
        result.min_braiding_gate_fidelity >= 0.9980,
        "Min braiding gate fidelity must be >= 0.9980, got {:.5}",
        result.min_braiding_gate_fidelity
    );

    // Assert topological protection gap >= 15.0 MHz
    assert!(
        result.mean_topological_protection_gap_mhz >= 15.0,
        "Mean topological protection gap must be >= 15.0 MHz, got {:.5}",
        result.mean_topological_protection_gap_mhz
    );
    assert!(
        result.min_topological_protection_gap_mhz >= 15.0,
        "Min topological protection gap must be >= 15.0 MHz, got {:.5}",
        result.min_topological_protection_gap_mhz
    );

    // Assert operation latency <= 150.0 ns
    assert!(
        result.mean_operation_latency_ns <= 150.0,
        "Mean operation latency must be <= 150.0 ns, got {:.5}",
        result.mean_operation_latency_ns
    );
    assert!(
        result.max_operation_latency_ns <= 150.0,
        "Max operation latency must be <= 150.0 ns, got {:.5}",
        result.max_operation_latency_ns
    );

    // Assert edge state isolation >= 40.0 dB
    assert!(
        result.mean_edge_state_isolation_db >= 40.0,
        "Mean edge state isolation must be >= 40.0 dB, got {:.5}",
        result.mean_edge_state_isolation_db
    );
    assert!(
        result.min_edge_state_isolation_db >= 40.0,
        "Min edge state isolation must be >= 40.0 dB, got {:.5}",
        result.min_edge_state_isolation_db
    );

    // Assert non-Abelian state purity >= 0.9950
    assert!(
        result.mean_non_abelian_state_purity >= 0.9950,
        "Mean non-Abelian state purity must be >= 0.9950, got {:.5}",
        result.mean_non_abelian_state_purity
    );
    assert!(
        result.min_non_abelian_state_purity >= 0.9950,
        "Min non-Abelian state purity must be >= 0.9950, got {:.5}",
        result.min_non_abelian_state_purity
    );
}
