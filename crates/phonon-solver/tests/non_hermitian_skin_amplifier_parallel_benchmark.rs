#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for non-Hermitian skin-topological phonon diodes and unidirectional quantum
//! acoustic amplifiers across multi-threaded Rayon workers.

use phonon_solver::non_hermitian_skin_amplifier::SkinAmplifierBenchmarkRunner;

#[test]
fn test_10k_non_hermitian_skin_amplifier_parallel_sweep() {
    let cycles = 10_000;
    let result = SkinAmplifierBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 131 Non-Hermitian Skin-Topological Phonon Diodes & Unidirectional Quantum Acoustic Amplifiers Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Forward Gain (dB): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_forward_gain_db,
        result.min_forward_gain_db,
        result.max_forward_gain_db
    );
    println!(
        "Reverse Isolation (dB): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_reverse_isolation_db,
        result.min_reverse_isolation_db,
        result.max_reverse_isolation_db
    );
    println!(
        "Added Noise Quanta: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_added_noise_quanta,
        result.min_added_noise_quanta,
        result.max_added_noise_quanta
    );
    println!(
        "Power Saturation Threshold (dBm): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_power_saturation_threshold_dbm,
        result.min_power_saturation_threshold_dbm,
        result.max_power_saturation_threshold_dbm
    );
    println!(
        "Skin Mode Localization Ratio: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_skin_mode_localization_ratio,
        result.min_skin_mode_localization_ratio,
        result.max_skin_mode_localization_ratio
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

    // Assert forward gain >= 28.0 dB
    assert!(
        result.mean_forward_gain_db >= 28.0,
        "Mean forward gain must be >= 28.0 dB, got {:.5}",
        result.mean_forward_gain_db
    );
    assert!(
        result.min_forward_gain_db >= 28.0,
        "Min forward gain must be >= 28.0 dB, got {:.5}",
        result.min_forward_gain_db
    );

    // Assert reverse isolation >= 42.0 dB
    assert!(
        result.mean_reverse_isolation_db >= 42.0,
        "Mean reverse isolation must be >= 42.0 dB, got {:.5}",
        result.mean_reverse_isolation_db
    );
    assert!(
        result.min_reverse_isolation_db >= 42.0,
        "Min reverse isolation must be >= 42.0 dB, got {:.5}",
        result.min_reverse_isolation_db
    );

    // Assert added noise <= 0.250 quanta
    assert!(
        result.mean_added_noise_quanta <= 0.250,
        "Mean added noise must be <= 0.250 quanta, got {:.5}",
        result.mean_added_noise_quanta
    );
    assert!(
        result.max_added_noise_quanta <= 0.250,
        "Max added noise must be <= 0.250 quanta, got {:.5}",
        result.max_added_noise_quanta
    );

    // Assert power saturation threshold >= -15.0 dBm
    assert!(
        result.mean_power_saturation_threshold_dbm >= -15.0,
        "Mean power saturation threshold must be >= -15.0 dBm, got {:.5}",
        result.mean_power_saturation_threshold_dbm
    );
    assert!(
        result.min_power_saturation_threshold_dbm >= -15.0,
        "Min power saturation threshold must be >= -15.0 dBm, got {:.5}",
        result.min_power_saturation_threshold_dbm
    );

    // Assert skin mode localization ratio >= 0.900
    assert!(
        result.mean_skin_mode_localization_ratio >= 0.900,
        "Mean skin mode localization ratio must be >= 0.900, got {:.5}",
        result.mean_skin_mode_localization_ratio
    );
    assert!(
        result.min_skin_mode_localization_ratio >= 0.900,
        "Min skin mode localization ratio must be >= 0.900, got {:.5}",
        result.min_skin_mode_localization_ratio
    );
}
