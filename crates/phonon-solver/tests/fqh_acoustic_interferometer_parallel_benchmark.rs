#![deny(unsafe_code)]

use phonon_solver::fqh_acoustic_interferometer::FqhInterferometerBenchmarkRunner;

#[test]
fn test_fqh_acoustic_interferometer_parallel_benchmark() {
    let runner = FqhInterferometerBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!("\n=== Phase 102: FQH Acoustic Interferometer & Noise Probe Benchmark ===");
    println!("Total cycles: {}", report.total_cycles);
    println!("Elapsed time: {:.4} s", report.elapsed_seconds);
    println!(
        "Throughput: {:.2} sweeps/s",
        report.throughput_cycles_per_sec
    );
    println!(
        "Mean charge precision error: {:.3e} (target <= 1.0e-4)",
        report.mean_precision_error
    );
    println!(
        "Max charge precision error: {:.3e}",
        report.max_precision_error
    );
    println!(
        "Mean Fano factor: {:.6} (target 0.25 +/- 1e-4)",
        report.mean_fano_factor
    );
    println!(
        "Mean fringe visibility: {:.2}% (target >= 90.0%)",
        report.mean_visibility_pct
    );
    println!("Min fringe visibility: {:.2}%", report.min_visibility_pct);
    println!(
        "Mean coherence length: {:.2} um (target >= 25.0 um)",
        report.mean_coherence_length_um
    );
    println!(
        "Min coherence length: {:.2} um",
        report.min_coherence_length_um
    );
    println!(
        "Mean cross-correlation: {:.2} dB (target <= -20.0 dB)",
        report.mean_cross_correlation_db
    );
    println!(
        "Max cross-correlation: {:.2} dB",
        report.max_cross_correlation_db
    );
    println!(
        "Mean readout SNR: {:.2} dB (target >= 25.0 dB)",
        report.mean_readout_snr_db
    );
    println!("Min readout SNR: {:.2} dB", report.min_readout_snr_db);
    println!(
        "Physical compliance fraction: {:.4}",
        report.compliance_fraction
    );

    assert_eq!(report.total_cycles, 10_000);
    assert!(report.throughput_cycles_per_sec > 100_000.0);
    assert!(report.mean_precision_error <= 1.0e-4);
    assert!(report.max_precision_error <= 1.0e-4);
    assert!((report.mean_fano_factor - 0.25).abs() <= 1.0e-4);
    assert!(report.mean_visibility_pct >= 90.0);
    assert!(report.min_visibility_pct >= 90.0);
    assert!(report.mean_coherence_length_um >= 25.0);
    assert!(report.min_coherence_length_um >= 25.0);
    assert!(report.mean_cross_correlation_db <= -20.0);
    assert!(report.max_cross_correlation_db <= -20.0);
    assert!(report.mean_readout_snr_db >= 25.0);
    assert!(report.min_readout_snr_db >= 25.0);
    assert_eq!(report.compliance_fraction, 1.0);
}
