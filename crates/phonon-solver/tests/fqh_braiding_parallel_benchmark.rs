//! Integration tests for parallel Rayon benchmark of FQH interferometry and 10,000 anyon braid sequences.

use phonon_solver::fqh::FqhBenchmarkRunner;

#[test]
fn test_parallel_10k_braid_sequences_and_interferometry() {
    let runner = FqhBenchmarkRunner::standard();
    let report = runner.run();

    assert_eq!(report.total_sequences, 10_000);
    assert!(
        report.mean_fidelity >= 0.999,
        "Mean braid fidelity must exceed 99.9%, got {}",
        report.mean_fidelity
    );
    assert!(
        report.min_fidelity >= 0.990,
        "Minimum braid fidelity must exceed 99.0%, got {}",
        report.min_fidelity
    );

    // Non-Abelian parity visibility: even >= 0.80, odd < 0.05
    assert!(
        report.even_anyon_visibility >= 0.80,
        "Even anyon visibility must be >= 0.80, got {}",
        report.even_anyon_visibility
    );
    assert!(
        report.odd_anyon_visibility < 0.05,
        "Odd anyon visibility must collapse, got {}",
        report.odd_anyon_visibility
    );
    assert!(
        report.visibility_extinction_ratio >= 50.0,
        "Extinction ratio must be >= 50, got {}",
        report.visibility_extinction_ratio
    );

    // Throughput
    assert!(
        report.throughput_sequences_per_sec > 20_000.0,
        "Throughput should exceed 20,000 seq/s, got {}",
        report.throughput_sequences_per_sec
    );
    assert!(
        report.passed_criteria,
        "All FQH benchmark criteria should pass"
    );
}
