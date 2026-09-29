//! Integration tests for 10,000-sweep parallel Rayon Magnon BEC benchmark.

use phonon_solver::magnon_bec::{MagnonBecBenchmarkConfig, MagnonBecBenchmarkRunner};

#[test]
fn test_magnon_bec_parallel_benchmark_execution() {
    let runner = MagnonBecBenchmarkRunner::new();
    let config = MagnonBecBenchmarkConfig {
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
        report.bec_formation_fraction > 0.50,
        "BEC formation fraction should be > 50%, got {:.2}",
        report.bec_formation_fraction
    );
    assert!(
        report.mean_chemical_potential_ratio > 0.80,
        "Mean chemical potential ratio should be > 0.8, got {:.2}",
        report.mean_chemical_potential_ratio
    );
    assert!(
        report.sub_critical_velocity_fraction > 0.95,
        "Subcritical fraction should be > 95%, got {:.2}",
        report.sub_critical_velocity_fraction
    );
    assert!(
        report.mean_transmission_advantage > 100.0,
        "Mean transmission advantage should exceed 100x, got {:.1}x",
        report.mean_transmission_advantage
    );
    assert!(
        report.min_transmission_advantage > 5.0,
        "Min transmission advantage should exceed 5x, got {:.1}x",
        report.min_transmission_advantage
    );
    assert!(
        report.mean_non_local_voltage_uv > 0.0,
        "Mean ISHE voltage should be positive"
    );
}
