//! Parallel Rayon benchmark test for 10,000 Majorana braid operations.

use phonon_solver::hexagonal_majorana::run_parallel_majorana_braid_benchmark;

#[test]
fn test_parallel_majorana_braid_10k_benchmark() {
    let report = run_parallel_majorana_braid_benchmark(10_000);

    println!("Majorana 10,000-Braid Benchmark Report:");
    println!("  Total Sweeps:               {}", report.total_sweeps);
    println!(
        "  Average Fidelity:           {:.6}",
        report.average_fidelity
    );
    println!(
        "  Minimum Fidelity:           {:.6}",
        report.minimum_fidelity
    );
    println!(
        "  High-Fidelity Fraction:     {:.2}%",
        report.high_fidelity_fraction * 100.0
    );
    println!(
        "  Average Diabatic Leakage:   {:.6e}",
        report.average_diabatic_leakage
    );
    println!(
        "  Average Commutator Norm:    {:.6}",
        report.average_commutator_norm
    );
    println!(
        "  Max Yang-Baxter Error:      {:.6e}",
        report.max_yang_baxter_error
    );
    println!(
        "  Elapsed Time:               {:.2} ms",
        report.elapsed_millis
    );
    println!(
        "  Throughput:                 {:.1} sweeps/sec",
        report.sweeps_per_second
    );

    assert_eq!(report.total_sweeps, 10_000);
    assert!(
        report.average_fidelity >= 0.99,
        "Average fidelity must be >= 0.99"
    );
    assert!(
        report.high_fidelity_fraction >= 0.95,
        "High-fidelity fraction must be >= 95%"
    );
    assert!(
        report.average_diabatic_leakage < 0.01,
        "Diabatic leakage must be < 1%"
    );
    assert!(
        report.average_commutator_norm > 0.5,
        "Non-Abelian commutator norm must be > 0.5"
    );
    assert!(
        report.max_yang_baxter_error < 1e-10,
        "Yang-Baxter relation error must be < 1e-10"
    );
    assert!(
        report.sweeps_per_second > 50_000.0,
        "Throughput must exceed 50,000 sweeps/sec"
    );
}
