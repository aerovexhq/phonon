#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for quantum acoustic chiral spin-mechanical frequency-bin entanglement
//! and phononic Bell state analyzers across multi-threaded Rayon workers.

use phonon_solver::chiral_frequency_bin_bell_analyzer::FrequencyBinBellBenchmarkRunner;

#[test]
fn test_10k_chiral_frequency_bin_bell_analyzer_parallel_sweep() {
    let cycles = 10_000;
    let result = FrequencyBinBellBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 147 Quantum Acoustic Frequency-Bin Entanglement & Phononic Bell State Analyzers Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Bell State Measurement Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_bell_state_measurement_fidelity,
        result.min_bell_state_measurement_fidelity,
        result.max_bell_state_measurement_fidelity
    );
    println!(
        "Frequency-Bin Mode Indistinguishability: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_frequency_bin_mode_indistinguishability,
        result.min_frequency_bin_mode_indistinguishability,
        result.max_frequency_bin_mode_indistinguishability
    );
    println!(
        "Cross-Talk Quantum Dephasing Rate (Hz): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_crosstalk_quantum_dephasing_rate_hz,
        result.min_crosstalk_quantum_dephasing_rate_hz,
        result.max_crosstalk_quantum_dephasing_rate_hz
    );
    println!(
        "Dark Count Probability: mean = {:.4e}, min = {:.4e}, max = {:.4e}",
        result.mean_dark_count_probability,
        result.min_dark_count_probability,
        result.max_dark_count_probability
    );
    println!(
        "Two-Phonon Entanglement Concurrence: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_two_phonon_entanglement_concurrence,
        result.min_two_phonon_entanglement_concurrence,
        result.max_two_phonon_entanglement_concurrence
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
        result.min_bell_state_measurement_fidelity >= 0.9950,
        "Minimum Bell state measurement fidelity must be >= 0.9950, got {:.6}",
        result.min_bell_state_measurement_fidelity
    );
    assert!(
        result.min_frequency_bin_mode_indistinguishability >= 0.9980,
        "Minimum frequency-bin mode indistinguishability must be >= 0.9980, got {:.6}",
        result.min_frequency_bin_mode_indistinguishability
    );
    assert!(
        result.max_crosstalk_quantum_dephasing_rate_hz <= 120.0,
        "Maximum cross-talk dephasing rate must be <= 120.0 Hz, got {:.4} Hz",
        result.max_crosstalk_quantum_dephasing_rate_hz
    );
    assert!(
        result.max_dark_count_probability <= 1.0e-5,
        "Maximum dark count probability must be <= 1.0e-5, got {:.4e}",
        result.max_dark_count_probability
    );
    assert!(
        result.min_two_phonon_entanglement_concurrence >= 0.980,
        "Minimum two-phonon entanglement concurrence must be >= 0.980, got {:.6}",
        result.min_two_phonon_entanglement_concurrence
    );
}
