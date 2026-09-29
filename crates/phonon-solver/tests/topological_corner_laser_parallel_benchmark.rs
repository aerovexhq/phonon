#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for topological acoustic higher-order corner mode lasers and non-Hermitian
//! phonon cavities across multi-threaded Rayon workers.

use phonon_solver::topological_corner_laser::CornerLaserBenchmarkRunner;

#[test]
fn test_10k_topological_corner_laser_parallel_sweep() {
    let cycles = 10_000;
    let result = CornerLaserBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 130 Topological Acoustic Higher-Order Corner Mode Lasers Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Corner Lasing Efficiency: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_corner_lasing_efficiency,
        result.min_corner_lasing_efficiency,
        result.max_corner_lasing_efficiency
    );
    println!(
        "Threshold Power (uW): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_threshold_power_uw,
        result.min_threshold_power_uw,
        result.max_threshold_power_uw
    );
    println!(
        "Corner Mode Localization: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_corner_mode_localization,
        result.min_corner_mode_localization,
        result.max_corner_mode_localization
    );
    println!(
        "Mode Discrimination (dB): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_mode_discrimination_db,
        result.min_mode_discrimination_db,
        result.max_mode_discrimination_db
    );
    println!(
        "Emission Linewidth (kHz): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_emission_linewidth_khz,
        result.min_emission_linewidth_khz,
        result.max_emission_linewidth_khz
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

    // Assert corner lasing efficiency >= 0.750
    assert!(
        result.mean_corner_lasing_efficiency >= 0.750,
        "Mean corner lasing efficiency must be >= 0.750, got {:.5}",
        result.mean_corner_lasing_efficiency
    );
    assert!(
        result.min_corner_lasing_efficiency >= 0.750,
        "Min corner lasing efficiency must be >= 0.750, got {:.5}",
        result.min_corner_lasing_efficiency
    );

    // Assert threshold power <= 10.0 uW
    assert!(
        result.mean_threshold_power_uw <= 10.0,
        "Mean threshold power must be <= 10.0 uW, got {:.5}",
        result.mean_threshold_power_uw
    );
    assert!(
        result.max_threshold_power_uw <= 10.0,
        "Max threshold power must be <= 10.0 uW, got {:.5}",
        result.max_threshold_power_uw
    );

    // Assert corner mode localization >= 0.920
    assert!(
        result.mean_corner_mode_localization >= 0.920,
        "Mean corner mode localization must be >= 0.920, got {:.5}",
        result.mean_corner_mode_localization
    );
    assert!(
        result.min_corner_mode_localization >= 0.920,
        "Min corner mode localization must be >= 0.920, got {:.5}",
        result.min_corner_mode_localization
    );

    // Assert mode discrimination >= 25.0 dB
    assert!(
        result.mean_mode_discrimination_db >= 25.0,
        "Mean mode discrimination must be >= 25.0 dB, got {:.5}",
        result.mean_mode_discrimination_db
    );
    assert!(
        result.min_mode_discrimination_db >= 25.0,
        "Min mode discrimination must be >= 25.0 dB, got {:.5}",
        result.min_mode_discrimination_db
    );

    // Assert emission linewidth <= 5.0 kHz
    assert!(
        result.mean_emission_linewidth_khz <= 5.0,
        "Mean emission linewidth must be <= 5.0 kHz, got {:.5}",
        result.mean_emission_linewidth_khz
    );
    assert!(
        result.max_emission_linewidth_khz <= 5.0,
        "Max emission linewidth must be <= 5.0 kHz, got {:.5}",
        result.max_emission_linewidth_khz
    );
}
