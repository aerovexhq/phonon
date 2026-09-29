use phonon_solver::non_hermitian_skin::NhseBenchmarkRunner;

#[test]
fn test_nhse_parallel_benchmark_10000_sweeps() {
    let runner = NhseBenchmarkRunner::new(10_000);
    let report = runner.run_parallel_benchmark();

    println!(
        "NHSE & Directional Amplifier Benchmark Report: {:#?}",
        report
    );

    assert_eq!(
        report.total_cycles, 10_000,
        "Must complete all 10,000 parameter sweeps"
    );

    assert_eq!(
        report.high_contrast_fraction, 1.0,
        "100% of sweeps must achieve skin localization >= 30 dB and contrast >= 30 dB"
    );

    assert!(
        report.min_skin_localization_db >= 30.0,
        "Minimum skin localization factor must be >= 30 dB: got {} dB",
        report.min_skin_localization_db
    );

    assert!(
        report.mean_skin_localization_db >= 40.0,
        "Mean skin localization factor must be >= 40 dB: got {} dB",
        report.mean_skin_localization_db
    );

    assert!(
        report.min_directional_contrast_db >= 30.0,
        "Minimum directional contrast must be >= 30 dB: got {} dB",
        report.min_directional_contrast_db
    );

    assert!(
        report.mean_directional_contrast_db >= 60.0,
        "Mean directional contrast must be >= 60 dB: got {} dB",
        report.mean_directional_contrast_db
    );

    assert!(
        report.throughput_cycles_per_sec > 10_000.0,
        "Throughput must exceed 10,000 sweeps/sec: got {}",
        report.throughput_cycles_per_sec
    );
}
