//! Multi-threaded Rayon benchmark verification for non-Hermitian polariton exceptional points across 10,000 parameter sweeps.

use phonon_solver::polariton_exceptional_point::PolaritonEpBenchmarkRunner;

#[test]
fn test_polariton_ep_parallel_benchmark() {
    let runner = PolaritonEpBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!("\n=== Non-Hermitian Polariton EP 10,000 Parameter Sweep Benchmark ===");
    println!("Total sweeps:                 {}", report.total_cycles);
    println!(
        "Elapsed time:                 {:.4} s",
        report.elapsed_seconds
    );
    println!(
        "Throughput:                   {:.2} sweeps/s",
        report.throughput_cycles_per_sec
    );
    println!(
        "Mean Exceptional Sensitivity: {:.2} dB (min: {:.2} dB, max: {:.2} dB)",
        report.mean_exceptional_sensitivity_db,
        report.min_exceptional_sensitivity_db,
        report.max_exceptional_sensitivity_db
    );
    println!(
        "Mean Chiral Mode Purity:      {:.3}% (min: {:.3}%)",
        report.mean_chiral_mode_purity_pct, report.min_chiral_mode_purity_pct
    );
    println!(
        "Mean Condensation Threshold:  {:.3} mW (max: {:.3} mW)",
        report.mean_condensation_threshold_mw, report.max_condensation_threshold_mw
    );
    println!(
        "Mean Polariton Linewidth:     {:.2} MHz (max: {:.2} MHz)",
        report.mean_polariton_laser_linewidth_mhz, report.max_polariton_laser_linewidth_mhz
    );
    println!(
        "Mean Gyro Enhancement:        {:.2}x (min: {:.2}x)",
        report.mean_gyro_scale_factor_enhancement, report.min_gyro_scale_factor_enhancement
    );
    println!(
        "Physical Compliance Fraction: {:.4} (100% required)",
        report.compliance_fraction
    );

    assert_eq!(report.total_cycles, 10_000);
    assert!(report.mean_exceptional_sensitivity_db >= 30.0);
    assert!(report.min_exceptional_sensitivity_db >= 30.0);
    assert!(report.mean_chiral_mode_purity_pct >= 99.0);
    assert!(report.min_chiral_mode_purity_pct >= 99.0);
    assert!(report.max_condensation_threshold_mw <= 5.0);
    assert!(report.max_polariton_laser_linewidth_mhz <= 50.0);
    assert!(report.mean_gyro_scale_factor_enhancement >= 10.0);
    assert!(report.min_gyro_scale_factor_enhancement >= 10.0);
    assert_eq!(
        report.compliance_fraction, 1.0,
        "All 10,000 parameter sweeps must satisfy physical bounds"
    );
}
