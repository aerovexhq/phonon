//! Parallel benchmark integration test for Floquet topological engineering,
//! verifying 10,000 parameter sweeps across Rayon threads.

use phonon_solver::floquet_topological::FloquetTopologicalBenchmarkRunner;

#[test]
fn test_floquet_topological_parallel_benchmark_10000_sweeps() {
    let runner = FloquetTopologicalBenchmarkRunner::new(10_000);
    let report = runner.run_parallel_benchmark();

    println!("\n=== Floquet Topological Benchmark Report ===");
    println!("Total sweeps: {}", report.total_cycles);
    println!("Elapsed time: {:.4} s", report.elapsed_seconds);
    println!(
        "Throughput: {:.2} sweeps/sec",
        report.throughput_cycles_per_sec
    );
    println!(
        "Mean bandgap opening: {:.2} meV",
        report.mean_gap_opening_mev
    );
    println!("Min bandgap opening: {:.2} meV", report.min_gap_opening_mev);
    println!(
        "Mean switching contrast: {:.2} dB",
        report.mean_switch_contrast_db
    );
    println!(
        "Min switching contrast: {:.2} dB",
        report.min_switch_contrast_db
    );
    println!("Max unitarity error: {:.2e}", report.max_unitarity_error);
    println!(
        "Topological compliance fraction: {:.4}",
        report.compliance_fraction
    );

    assert_eq!(report.total_cycles, 10_000);
    assert!(
        report.throughput_cycles_per_sec > 10_000.0,
        "Throughput must exceed 10,000 sweeps/sec, achieved {:.2}",
        report.throughput_cycles_per_sec
    );
    assert!(
        report.mean_gap_opening_mev >= 50.0,
        "Mean dynamic gap must be >= 50 meV, got {:.2} meV",
        report.mean_gap_opening_mev
    );
    assert!(
        report.min_gap_opening_mev >= 50.0,
        "Min dynamic gap must be >= 50 meV, got {:.2} meV",
        report.min_gap_opening_mev
    );
    assert!(
        report.mean_switch_contrast_db >= 30.0,
        "Mean switching contrast must be >= 30 dB, got {:.2} dB",
        report.mean_switch_contrast_db
    );
    assert!(
        report.min_switch_contrast_db >= 30.0,
        "Min switching contrast must be >= 30 dB, got {:.2} dB",
        report.min_switch_contrast_db
    );
    assert!(
        report.max_unitarity_error < 1e-10,
        "Max unitarity error must be < 1e-10, got {:.2e}",
        report.max_unitarity_error
    );
    assert_eq!(
        report.compliance_fraction, 1.0,
        "All sweeps must be 100% compliant with topological gap and contrast criteria"
    );
}
