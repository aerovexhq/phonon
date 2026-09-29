#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for quantum acoustic tensor network simulators and continuous-variable
//! fault-tolerant magic state distillation across multi-threaded Rayon workers.

use phonon_solver::quantum_acoustic_tensor_distillation::DistillationBenchmarkRunner;

#[test]
fn test_10k_quantum_acoustic_tensor_distillation_parallel_sweep() {
    let cycles = 10_000;
    let result = DistillationBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 123 Quantum Acoustic Tensor Distillation Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Magic State Fidelity: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_magic_state_fidelity, result.min_magic_state_fidelity, result.max_magic_state_fidelity
    );
    println!(
        "Photon Subtraction Probability: mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_photon_subtraction_prob, result.min_photon_subtraction_prob, result.max_photon_subtraction_prob
    );
    println!(
        "Distillation Cycle Latency (us): mean = {:.3}, min = {:.3}, max = {:.3}",
        result.mean_distillation_cycle_latency_us, result.min_distillation_cycle_latency_us, result.max_distillation_cycle_latency_us
    );
    println!(
        "Non-Gaussian Gate Fidelity: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_non_gaussian_gate_fidelity, result.min_non_gaussian_gate_fidelity, result.max_non_gaussian_gate_fidelity
    );
    println!(
        "Acoustic Error Threshold: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_acoustic_error_threshold, result.min_acoustic_error_threshold, result.max_acoustic_error_threshold
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

    // Assert magic state fidelity >= 0.990 (99.0%)
    assert!(
        result.mean_magic_state_fidelity >= 0.990,
        "Mean magic state fidelity must be >= 0.990, got {:.5}",
        result.mean_magic_state_fidelity
    );
    assert!(
        result.min_magic_state_fidelity >= 0.990,
        "Worst-case magic state fidelity must be >= 0.990, got {:.5}",
        result.min_magic_state_fidelity
    );

    // Assert photon subtraction probability >= 0.150 (15.0%)
    assert!(
        result.mean_photon_subtraction_prob >= 0.150,
        "Mean photon subtraction prob must be >= 0.150, got {:.4}",
        result.mean_photon_subtraction_prob
    );
    assert!(
        result.min_photon_subtraction_prob >= 0.150,
        "Worst-case photon subtraction prob must be >= 0.150, got {:.4}",
        result.min_photon_subtraction_prob
    );

    // Assert distillation cycle latency <= 5.0 us
    assert!(
        result.mean_distillation_cycle_latency_us <= 5.0,
        "Mean distillation cycle latency must be <= 5.0 us, got {:.3} us",
        result.mean_distillation_cycle_latency_us
    );
    assert!(
        result.max_distillation_cycle_latency_us <= 5.0,
        "Worst-case distillation cycle latency must be <= 5.0 us, got {:.3} us",
        result.max_distillation_cycle_latency_us
    );

    // Assert non-Gaussian gate fidelity >= 0.985 (98.5%)
    assert!(
        result.mean_non_gaussian_gate_fidelity >= 0.985,
        "Mean non-Gaussian gate fidelity must be >= 0.985, got {:.5}",
        result.mean_non_gaussian_gate_fidelity
    );
    assert!(
        result.min_non_gaussian_gate_fidelity >= 0.985,
        "Worst-case non-Gaussian gate fidelity must be >= 0.985, got {:.5}",
        result.min_non_gaussian_gate_fidelity
    );

    // Assert acoustic physical error threshold >= 0.015 (1.5%)
    assert!(
        result.mean_acoustic_error_threshold >= 0.015,
        "Mean acoustic error threshold must be >= 0.015, got {:.5}",
        result.mean_acoustic_error_threshold
    );
    assert!(
        result.min_acoustic_error_threshold >= 0.015,
        "Worst-case acoustic error threshold must be >= 0.015, got {:.5}",
        result.min_acoustic_error_threshold
    );
}
