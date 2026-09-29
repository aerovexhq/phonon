//! Parallel Rayon benchmark test for quantum topological polariton condensates,
//! optomechanical vortices, and non-equilibrium superfluids across 10,000 parameter sweeps.

use phonon_solver::polariton_condensate::PolaritonCondensateBenchmarkRunner;

#[test]
fn test_polariton_condensate_parallel_benchmark_10000_sweeps() {
    let runner = PolaritonCondensateBenchmarkRunner::new(10_000);
    let report = runner.run_parallel_benchmark();

    assert_eq!(
        report.total_cycles, 10_000,
        "Benchmark must execute exactly 10,000 sweeps"
    );
    assert!(
        report.elapsed_seconds > 0.0,
        "Elapsed time must be non-zero"
    );
    assert!(
        report.throughput_cycles_per_sec > 10_000.0,
        "Throughput must exceed 10,000 sweeps/sec, achieved {:.2} sweeps/sec",
        report.throughput_cycles_per_sec
    );

    // Physical requirements
    assert!(
        report.mean_superfluid_fraction >= 0.80,
        "Mean superfluid fraction must be >= 0.80 (80%), got {:.4}",
        report.mean_superfluid_fraction
    );
    assert!(
        report.min_superfluid_fraction >= 0.80,
        "Minimum superfluid fraction across all sweeps must be >= 0.80, got {:.4}",
        report.min_superfluid_fraction
    );

    assert!(
        report.mean_sound_speed_m_s > 1.0e5,
        "Mean Bogoliubov sound speed must exceed 1e5 m/s, got {:.1} m/s",
        report.mean_sound_speed_m_s
    );

    assert!(
        (report.mean_circulation_ratio - 1.0).abs() < 1e-4,
        "Mean circulation quantum ratio must be 1.0, got {:.4}",
        report.mean_circulation_ratio
    );

    assert!(
        report.mean_switching_contrast_db >= 25.0,
        "Mean switching contrast must be >= 25.0 dB, got {:.2} dB",
        report.mean_switching_contrast_db
    );
    assert!(
        report.min_switching_contrast_db >= 25.0,
        "Minimum switching contrast across all sweeps must be >= 25.0 dB, got {:.2} dB",
        report.min_switching_contrast_db
    );

    assert_eq!(
        report.compliance_fraction,
        1.0,
        "All sweeps must be 100% compliant with physical criteria, got {:.2}%",
        report.compliance_fraction * 100.0
    );

    println!(
        "Phase 77 Polariton Condensate Benchmark Completed:\n\
         - Sweeps: {}\n\
         - Elapsed: {:.4} s\n\
         - Throughput: {:.2} sweeps/sec\n\
         - Mean Superfluid Fraction: {:.2}% (min: {:.2}%)\n\
         - Mean Sound Speed: {:.1} m/s\n\
         - Mean Circulation Ratio: {:.4}\n\
         - Mean Switching Contrast: {:.2} dB (min: {:.2} dB)\n\
         - Compliance: {:.1}%",
        report.total_cycles,
        report.elapsed_seconds,
        report.throughput_cycles_per_sec,
        report.mean_superfluid_fraction * 100.0,
        report.min_superfluid_fraction * 100.0,
        report.mean_sound_speed_m_s,
        report.mean_circulation_ratio,
        report.mean_switching_contrast_db,
        report.min_switching_contrast_db,
        report.compliance_fraction * 100.0
    );
}
