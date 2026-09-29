#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for topological quantum acoustic memory across multi-threaded Rayon workers.

use phonon_solver::majorana_surface_memory::MemoryBenchmarkRunner;

#[test]
fn test_10k_majorana_surface_memory_parallel_sweep() {
    let cycles = 10_000;
    let result = MemoryBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 121 Topological Quantum Acoustic Memory Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!("Physical Compliance Fraction: {:.2}%", result.physical_compliance_fraction * 100.0);
    println!("Quantum Coherence T2 (ms): mean = {:.2}, min = {:.2}", result.mean_quantum_coherence_t2_ms, result.min_quantum_coherence_t2_ms);
    println!("Fault-Tolerant Threshold: mean = {:.4}, min = {:.4}", result.mean_fault_tolerant_threshold, result.min_fault_tolerant_threshold);
    println!("Syndrome Decoding Latency (us): mean = {:.3}, max = {:.3}", result.mean_syndrome_decoding_latency_us, result.max_syndrome_decoding_latency_us);
    println!("Logical Error Rate: mean = {:.4e}, max = {:.4e}", result.mean_logical_error_rate, result.max_logical_error_rate);
    println!("Acoustic Qubit Storage Fidelity: mean = {:.5}, min = {:.5}", result.mean_acoustic_qubit_storage_fidelity, result.min_acoustic_qubit_storage_fidelity);

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

    // Assert quantum coherence time T2 >= 10.0 ms
    assert!(
        result.mean_quantum_coherence_t2_ms >= 10.0,
        "Mean quantum coherence T2 must be >= 10.0 ms, got {:.2} ms",
        result.mean_quantum_coherence_t2_ms
    );
    assert!(
        result.min_quantum_coherence_t2_ms >= 10.0,
        "Worst-case quantum coherence T2 must be >= 10.0 ms, got {:.2} ms",
        result.min_quantum_coherence_t2_ms
    );

    // Assert fault-tolerant threshold >= 0.010 (1.0%)
    assert!(
        result.mean_fault_tolerant_threshold >= 0.010,
        "Mean fault-tolerant threshold must be >= 0.010, got {:.4}",
        result.mean_fault_tolerant_threshold
    );
    assert!(
        result.min_fault_tolerant_threshold >= 0.010,
        "Worst-case fault-tolerant threshold must be >= 0.010, got {:.4}",
        result.min_fault_tolerant_threshold
    );

    // Assert syndrome decoding latency <= 2.50 us
    assert!(
        result.mean_syndrome_decoding_latency_us <= 2.50,
        "Mean syndrome decoding latency must be <= 2.50 us, got {:.3} us",
        result.mean_syndrome_decoding_latency_us
    );
    assert!(
        result.max_syndrome_decoding_latency_us <= 2.50,
        "Worst-case syndrome decoding latency must be <= 2.50 us, got {:.3} us",
        result.max_syndrome_decoding_latency_us
    );

    // Assert logical error rate <= 1.0e-5
    assert!(
        result.mean_logical_error_rate <= 1.0e-5,
        "Mean logical error rate must be <= 1.0e-5, got {:.4e}",
        result.mean_logical_error_rate
    );
    assert!(
        result.max_logical_error_rate <= 1.0e-5,
        "Worst-case logical error rate must be <= 1.0e-5, got {:.4e}",
        result.max_logical_error_rate
    );

    // Assert acoustic qubit storage fidelity >= 0.995 (99.5%)
    assert!(
        result.mean_acoustic_qubit_storage_fidelity >= 0.995,
        "Mean acoustic qubit storage fidelity must be >= 0.995, got {:.5}",
        result.mean_acoustic_qubit_storage_fidelity
    );
    assert!(
        result.min_acoustic_qubit_storage_fidelity >= 0.995,
        "Worst-case acoustic qubit storage fidelity must be >= 0.995, got {:.5}",
        result.min_acoustic_qubit_storage_fidelity
    );
}
