#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for hybrid superconducting opto-acoustic quantum repeaters and entanglement
//! distribution networks across multi-threaded Rayon workers.

use phonon_solver::opto_acoustic_quantum_repeater::RepeaterBenchmarkRunner;

#[test]
fn test_10k_opto_acoustic_quantum_repeater_parallel_sweep() {
    let cycles = 10_000;
    let result = RepeaterBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 124 Opto-Acoustic Quantum Repeater Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Bell-State Fidelity: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_bell_state_fidelity, result.min_bell_state_fidelity, result.max_bell_state_fidelity
    );
    println!(
        "Repetition Rate (kHz): mean = {:.3}, min = {:.3}, max = {:.3}",
        result.mean_repetition_rate_khz, result.min_repetition_rate_khz, result.max_repetition_rate_khz
    );
    println!(
        "Distribution Latency (us): mean = {:.3}, min = {:.3}, max = {:.3}",
        result.mean_distribution_latency_us, result.min_distribution_latency_us, result.max_distribution_latency_us
    );
    println!(
        "Memory-Transduction Roundtrip Fidelity: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_memory_transduction_roundtrip_fidelity,
        result.min_memory_transduction_roundtrip_fidelity,
        result.max_memory_transduction_roundtrip_fidelity
    );
    println!(
        "Purification Efficiency: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_purification_efficiency, result.min_purification_efficiency, result.max_purification_efficiency
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

    // Assert Bell-state fidelity >= 0.950 (95.0%)
    assert!(
        result.mean_bell_state_fidelity >= 0.950,
        "Mean Bell-state fidelity must be >= 0.950, got {:.5}",
        result.mean_bell_state_fidelity
    );
    assert!(
        result.min_bell_state_fidelity >= 0.950,
        "Worst-case Bell-state fidelity must be >= 0.950, got {:.5}",
        result.min_bell_state_fidelity
    );

    // Assert repetition rate >= 100.0 kHz
    assert!(
        result.mean_repetition_rate_khz >= 100.0,
        "Mean repetition rate must be >= 100.0 kHz, got {:.3} kHz",
        result.mean_repetition_rate_khz
    );
    assert!(
        result.min_repetition_rate_khz >= 100.0,
        "Worst-case repetition rate must be >= 100.0 kHz, got {:.3} kHz",
        result.min_repetition_rate_khz
    );

    // Assert distribution latency <= 10.0 us
    assert!(
        result.mean_distribution_latency_us <= 10.0,
        "Mean distribution latency must be <= 10.0 us, got {:.3} us",
        result.mean_distribution_latency_us
    );
    assert!(
        result.max_distribution_latency_us <= 10.0,
        "Worst-case distribution latency must be <= 10.0 us, got {:.3} us",
        result.max_distribution_latency_us
    );

    // Assert memory-transduction roundtrip fidelity >= 0.980 (98.0%)
    assert!(
        result.mean_memory_transduction_roundtrip_fidelity >= 0.980,
        "Mean roundtrip fidelity must be >= 0.980, got {:.5}",
        result.mean_memory_transduction_roundtrip_fidelity
    );
    assert!(
        result.min_memory_transduction_roundtrip_fidelity >= 0.980,
        "Worst-case roundtrip fidelity must be >= 0.980, got {:.5}",
        result.min_memory_transduction_roundtrip_fidelity
    );

    // Assert purification efficiency >= 0.850 (85.0%)
    assert!(
        result.mean_purification_efficiency >= 0.850,
        "Mean purification efficiency must be >= 0.850, got {:.5}",
        result.mean_purification_efficiency
    );
    assert!(
        result.min_purification_efficiency >= 0.850,
        "Worst-case purification efficiency must be >= 0.850, got {:.5}",
        result.min_purification_efficiency
    );
}
