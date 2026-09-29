//! Integration tests for 10,000-sweep parallel Rayon Chiral Polariton benchmark.

use phonon_solver::chiral_polariton::{
    ChiralPolaritonBenchmarkConfig, ChiralPolaritonBenchmarkRunner,
};

#[test]
fn test_chiral_polariton_parallel_benchmark_execution() {
    let runner = ChiralPolaritonBenchmarkRunner::new();
    let config = ChiralPolaritonBenchmarkConfig {
        total_sweeps: 10_000,
        ..Default::default()
    };

    let report = runner.run_benchmark(&config);

    assert_eq!(report.total_sweeps, 10_000);
    assert!(report.elapsed_ms > 0.0);
    assert!(
        report.sweeps_per_sec > 10_000.0,
        "Throughput should be > 10k sweeps/sec, got {:.0}",
        report.sweeps_per_sec
    );

    // Physics assertions
    assert!(
        report.mean_splitting_mhz >= 10.0,
        "Mean splitting should be >= 10 MHz, got {:.2} MHz",
        report.mean_splitting_mhz
    );
    assert!(
        report.min_splitting_mhz >= 8.0,
        "Min splitting should be >= 8 MHz, got {:.2} MHz",
        report.min_splitting_mhz
    );
    assert!(
        report.mean_isolation_db >= 20.0,
        "Mean isolation should be >= 20 dB, got {:.2} dB",
        report.mean_isolation_db
    );
    assert!(
        report.min_isolation_db >= 20.0,
        "Min isolation must be >= 20 dB, got {:.2} dB",
        report.min_isolation_db
    );
    assert_eq!(
        report.isolation_compliance_fraction, 1.0,
        "100% of sweeps must achieve >= 20 dB isolation"
    );
    assert!(
        report.mean_ishe_voltage_uv > 0.05,
        "Mean ISHE voltage should be > 0.05 uV, got {:.3} uV",
        report.mean_ishe_voltage_uv
    );
}
