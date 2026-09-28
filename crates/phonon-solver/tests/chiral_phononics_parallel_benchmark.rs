use phonon_solver::phononic_topological::{
    ChiralPhononicsBenchmarkConfig, ChiralPhononicsBenchmarkReport, ChiralPhononicsBenchmarkRunner,
};

#[test]
fn test_chiral_phononics_10k_parallel_benchmark() {
    let runner = ChiralPhononicsBenchmarkRunner::new();
    let config = ChiralPhononicsBenchmarkConfig {
        total_cycles: 10_000,
        ..ChiralPhononicsBenchmarkConfig::default()
    };

    let report: ChiralPhononicsBenchmarkReport = runner.run(&config);

    assert_eq!(report.total_cycles, 10_000);
    assert!(report.elapsed_ms > 0.0);
    assert!(
        report.cycles_per_sec > 20_000.0,
        "Cycles per second was {}, expected > 20,000",
        report.cycles_per_sec
    );

    // Topological valley bandgap opening assertion: > 5%
    assert!(
        report.mean_valley_gap_ratio > 0.05,
        "Mean valley gap ratio was {}, expected > 0.05",
        report.mean_valley_gap_ratio
    );
    assert!(
        report.max_valley_gap_ratio > 0.10,
        "Max valley gap ratio was {}, expected > 0.10",
        report.max_valley_gap_ratio
    );

    // Corner transmission assertion: >= 90%
    assert!(
        report.mean_corner_transmission >= 0.90,
        "Mean corner transmission was {}, expected >= 0.90",
        report.mean_corner_transmission
    );

    // Diode and Circulator non-reciprocal isolation assertions: > 20 dB
    assert!(
        report.mean_diode_isolation_db > 20.0,
        "Mean diode isolation was {} dB, expected > 20 dB",
        report.mean_diode_isolation_db
    );
    assert!(
        report.mean_circulator_isolation_db > 20.0,
        "Mean circulator isolation was {} dB, expected > 20 dB",
        report.mean_circulator_isolation_db
    );

    // Topological yield fraction: >= 95%
    assert!(
        report.topological_fraction >= 0.95,
        "Topological fraction was {}, expected >= 0.95",
        report.topological_fraction
    );
}
