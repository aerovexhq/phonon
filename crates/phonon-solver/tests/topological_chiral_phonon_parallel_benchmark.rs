use phonon_solver::chiral_phonon::ChiralPhononBenchmarkRunner;

#[test]
fn test_chiral_phonon_parallel_benchmark_10000_sweeps() {
    let runner = ChiralPhononBenchmarkRunner::new(10_000);
    let report = runner.run_parallel_benchmark();

    println!("Chiral Phonon Benchmark Report: {:#?}", report);

    assert_eq!(
        report.total_cycles, 10_000,
        "Must complete all 10,000 thermal cycles"
    );

    assert!(
        report.min_rectification_ratio >= 10.0,
        "Minimum rectification ratio R = {} must be >= 10.0x",
        report.min_rectification_ratio
    );

    assert_eq!(
        report.high_contrast_fraction, 1.0,
        "100% of cycles must achieve high-contrast thermal rectification R >= 10x"
    );

    assert!(
        report.mean_corner_transmission >= 0.90,
        "Mean corner transmission T_bend = {} must be >= 90%",
        report.mean_corner_transmission
    );

    assert!(
        report.max_backscattering_prob <= 0.10,
        "Maximum backscattering reflection R_back = {} must be <= 10%",
        report.max_backscattering_prob
    );

    assert!(
        report.throughput_cycles_per_sec > 10_000.0,
        "Throughput must exceed 10,000 cycles/sec: got {}",
        report.throughput_cycles_per_sec
    );
}
