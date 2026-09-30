#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for non-Abelian quantum acoustic fault-tolerant surface codes and chiral
//! Majorana stabilizer simulators across multi-threaded Rayon workers.

use phonon_solver::quantum_acoustic_surface_code::SurfaceCodeBenchmarkRunner;

#[test]
fn test_10k_quantum_acoustic_surface_code_parallel_sweep() {
    let cycles = 10_000;
    let result = SurfaceCodeBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 160 Non-Abelian Quantum Acoustic Fault-Tolerant Surface Codes & Chiral Majorana Stabilizer Simulators Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Logical State Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_logical_state_fidelity,
        result.min_logical_state_fidelity,
        result.max_logical_state_fidelity
    );
    println!(
        "Fault-Tolerant Threshold Error Rate: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_fault_tolerant_threshold_error_rate,
        result.min_fault_tolerant_threshold_error_rate,
        result.max_fault_tolerant_threshold_error_rate
    );
    println!(
        "Syndrome Decoding Latency (ns): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_syndrome_decoding_latency_ns,
        result.min_syndrome_decoding_latency_ns,
        result.max_syndrome_decoding_latency_ns
    );
    println!(
        "Uncorrectable Logical Error Rate: mean = {:.4e}, min = {:.4e}, max = {:.4e}",
        result.mean_uncorrectable_logical_error_rate,
        result.min_uncorrectable_logical_error_rate,
        result.max_uncorrectable_logical_error_rate
    );
    println!(
        "Inter-Stabilizer Crosstalk Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_inter_stabilizer_crosstalk_isolation_db,
        result.min_inter_stabilizer_crosstalk_isolation_db,
        result.max_inter_stabilizer_crosstalk_isolation_db
    );

    assert_eq!(
        result.total_cycles, cycles,
        "Total benchmark cycles must equal requested 10,000"
    );

    // Verify 100% physical compliance across all parameter sweep variations
    assert!(
        (result.physical_compliance_fraction - 1.0).abs() < 1.0e-6,
        "Physical compliance must be 100.0%, got {:.2}%",
        result.physical_compliance_fraction * 100.0
    );

    // Verify roadmap thresholds
    assert!(
        result.min_logical_state_fidelity >= 0.9980,
        "Minimum logical state fidelity must be >= 0.9980, got {:.6}",
        result.min_logical_state_fidelity
    );
    assert!(
        result.max_fault_tolerant_threshold_error_rate <= 0.0075,
        "Maximum fault-tolerant threshold error rate must be <= 0.0075, got {:.6}",
        result.max_fault_tolerant_threshold_error_rate
    );
    assert!(
        result.max_syndrome_decoding_latency_ns <= 120.0,
        "Maximum syndrome decoding latency must be <= 120.0 ns, got {:.4}",
        result.max_syndrome_decoding_latency_ns
    );
    assert!(
        result.max_uncorrectable_logical_error_rate <= 1.0e-5,
        "Maximum uncorrectable logical error rate must be <= 1.0e-5, got {:.4e}",
        result.max_uncorrectable_logical_error_rate
    );
    assert!(
        result.min_inter_stabilizer_crosstalk_isolation_db >= 52.0,
        "Minimum inter-stabilizer crosstalk isolation must be >= 52.0 dB, got {:.4}",
        result.min_inter_stabilizer_crosstalk_isolation_db
    );
}
