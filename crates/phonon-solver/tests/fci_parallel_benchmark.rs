use phonon_solver::fractional_chern::FciBenchmarkRunner;

#[test]
fn test_fci_parallel_benchmark_10000_sweeps() {
    let runner = FciBenchmarkRunner::new(10_000);
    let report = runner.run_parallel_benchmark();

    println!(
        "FCI & Anyonic Teleportation Benchmark Report: {:#?}",
        report
    );

    assert_eq!(
        report.total_cycles, 10_000,
        "Must complete all 10,000 parameter sweeps"
    );

    assert_eq!(
        report.high_fidelity_fraction, 1.0,
        "100% of sweeps must achieve fidelity >= 99% and gap >= 1.0 meV"
    );

    assert!(
        report.min_fidelity >= 0.99,
        "Minimum teleportation fidelity must be >= 99%: got {}",
        report.min_fidelity
    );

    assert!(
        report.mean_fidelity >= 0.995,
        "Mean teleportation fidelity must be >= 99.5%: got {}",
        report.mean_fidelity
    );

    assert!(
        report.mean_spectral_gap_ev >= 0.001,
        "Mean spectral gap must be >= 1.0 meV: got {} meV",
        report.mean_spectral_gap_ev * 1e3
    );

    assert!(
        report.mean_fubini_study_ratio >= 1.0 && report.mean_fubini_study_ratio <= 1.15,
        "Mean Fubini-Study trace ratio must satisfy [1.0, 1.15]: got {}",
        report.mean_fubini_study_ratio
    );

    assert!(
        report.throughput_cycles_per_sec > 10_000.0,
        "Throughput must exceed 10,000 sweeps/sec: got {}",
        report.throughput_cycles_per_sec
    );
}
