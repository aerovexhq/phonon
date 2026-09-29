use phonon_solver::valleytronics::ValleytronicsBenchmarkRunner;

#[test]
fn test_valleytronics_parallel_benchmark_10000_sweeps() {
    let runner = ValleytronicsBenchmarkRunner::new(10_000);
    let report = runner.run_parallel_benchmark();

    println!("Valleytronics Benchmark Report: {:#?}", report);

    assert_eq!(
        report.total_cycles, 10_000,
        "Must complete all 10,000 parameter sweeps"
    );

    assert_eq!(
        report.high_contrast_fraction, 1.0,
        "100% of sweeps must achieve rectification >= 20 dB and ON/OFF >= 20 dB"
    );

    assert!(
        report.min_rectification_db >= 20.0,
        "Minimum rectification ratio must be >= 20 dB: got {}",
        report.min_rectification_db
    );

    assert!(
        report.mean_on_off_ratio_db >= 20.0,
        "Mean transistor ON/OFF ratio must be >= 20 dB: got {}",
        report.mean_on_off_ratio_db
    );

    assert!(
        report.mean_circular_dichroism >= 0.95,
        "Mean circular dichroism must be >= 95%: got {}",
        report.mean_circular_dichroism
    );

    assert!(
        report.throughput_cycles_per_sec > 10_000.0,
        "Throughput must exceed 10,000 sweeps/sec: got {}",
        report.throughput_cycles_per_sec
    );
}
