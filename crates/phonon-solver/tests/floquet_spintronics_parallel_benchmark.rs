//! Parallel Multi-Cycle Floquet Benchmark Integration Test.

use phonon_solver::floquet::{FloquetBenchmarkConfig, FloquetBenchmarkRunner};

#[test]
fn test_floquet_10000_cycles_parallel_benchmark() {
    let runner = FloquetBenchmarkRunner::new();
    let config = FloquetBenchmarkConfig {
        total_cycles: 10_000,
        wavelength_m: 3.2e-6,
        base_field_v_per_m: 4.0e8,
    };

    let report = runner.run(&config);

    println!(
        "Floquet Topological Benchmark: {} cycles in {:.2} ms ({:.0} cycles/sec)",
        report.total_cycles, report.elapsed_ms, report.cycles_per_sec
    );
    println!(
        "  Mean Floquet Mass Gap: {:.2} meV",
        report.mean_mass_gap_ev * 1e3
    );
    println!(
        "  Max Floquet Mass Gap:  {:.2} meV",
        report.max_mass_gap_ev * 1e3
    );
    println!(
        "  Mean Harmonic Cutoff:  {:.1}",
        report.mean_harmonic_cutoff
    );
    println!(
        "  Topological Fraction:  {:.1}%",
        report.topological_fraction * 100.0
    );

    assert_eq!(report.total_cycles, 10_000);
    assert!(
        report.mean_mass_gap_ev > 0.05,
        "Mean gap should exceed 50 meV"
    );
    assert!(
        report.max_mass_gap_ev > 0.10,
        "Max gap should exceed 100 meV"
    );
    assert!(
        report.mean_harmonic_cutoff >= 5.0,
        "Mean cutoff should be >= 5"
    );
    assert!(
        report.topological_fraction > 0.70,
        "Over 70% must be topological"
    );
    assert!(
        report.cycles_per_sec > 10_000.0,
        "Throughput should exceed 10,000 cycles/sec"
    );
}
