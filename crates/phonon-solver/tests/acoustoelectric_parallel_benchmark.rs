use phonon_solver::acoustoelectric::AcoustoelectricBenchmarkRunner;

#[test]
fn test_acoustoelectric_parallel_benchmark_10000_cycles() {
    let runner = AcoustoelectricBenchmarkRunner::new(10_000);
    let report = runner.run_parallel_benchmark();

    println!("Acoustoelectric Benchmark Report: {:#?}", report);

    assert_eq!(
        report.total_cycles, 10_000,
        "Must complete all 10,000 acoustic cycle sweeps"
    );

    assert_eq!(
        report.high_precision_fraction, 1.0,
        "100% of sweeps on the plateau must achieve quantization error < 1e-4"
    );

    assert!(
        report.mean_quantization_error < 1.0e-4,
        "Mean quantization error must be < 1e-4: got {}",
        report.mean_quantization_error
    );

    assert!(
        report.mean_bell_fidelity >= 0.95,
        "Mean Bell state fidelity must be >= 95%: got {}",
        report.mean_bell_fidelity
    );

    assert!(
        report.mean_concurrence >= 0.90,
        "Mean entanglement concurrence must be >= 0.90: got {}",
        report.mean_concurrence
    );

    assert!(
        report.throughput_cycles_per_sec > 10_000.0,
        "Throughput must exceed 10,000 cycles/sec: got {}",
        report.throughput_cycles_per_sec
    );
}
