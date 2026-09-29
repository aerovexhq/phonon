//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for quantum acoustic topological Chern circulators across multi-threaded Rayon workers.

use phonon_solver::topological_chern_circulator::ChernCirculatorBenchmarkRunner;

#[test]
fn test_10k_chern_circulator_parallel_sweep() {
    let cycles = 10_000;
    let result = ChernCirculatorBenchmarkRunner::run_benchmark(cycles);

    assert_eq!(
        result.total_cycles, cycles,
        "Total benchmark cycles must equal requested 10,000"
    );

    // Verify 100% physical compliance across all parameter sweep variations
    assert!(
        (result.physical_compliance_fraction - 1.0).abs() < 1.0e-6,
        "Physical compliance fraction must be 100.0%, got {:.2}%",
        result.physical_compliance_fraction * 100.0
    );

    // Assert forward acoustic transmission >= 95.0%
    assert!(
        result.mean_forward_transmission >= 0.950,
        "Mean forward transmission must be >= 0.950, got {:.4}",
        result.mean_forward_transmission
    );
    assert!(
        result.min_forward_transmission >= 0.950,
        "Worst-case forward transmission must be >= 0.950, got {:.4}",
        result.min_forward_transmission
    );

    // Assert non-reciprocal isolation >= 35.0 dB
    assert!(
        result.mean_isolation_db >= 35.0,
        "Mean non-reciprocal isolation must be >= 35.0 dB, got {:.2} dB",
        result.mean_isolation_db
    );
    assert!(
        result.min_isolation_db >= 35.0,
        "Worst-case non-reciprocal isolation must be >= 35.0 dB, got {:.2} dB",
        result.min_isolation_db
    );

    // Assert topological bandgap ratio >= 0.120
    assert!(
        result.mean_bandgap_ratio >= 0.120,
        "Mean topological bandgap ratio must be >= 0.120, got {:.4}",
        result.mean_bandgap_ratio
    );
    assert!(
        result.min_bandgap_ratio >= 0.120,
        "Worst-case topological bandgap ratio must be >= 0.120, got {:.4}",
        result.min_bandgap_ratio
    );

    // Assert backscattering reflection <= -40.0 dB
    assert!(
        result.mean_backscattering_db <= -40.0,
        "Mean backscattering reflection must be <= -40.0 dB, got {:.2} dB",
        result.mean_backscattering_db
    );
    assert!(
        result.max_backscattering_db <= -40.0,
        "Worst-case backscattering reflection must be <= -40.0 dB, got {:.2} dB",
        result.max_backscattering_db
    );

    // Assert insertion loss <= 0.80 dB
    assert!(
        result.mean_insertion_loss_db <= 0.80,
        "Mean insertion loss must be <= 0.80 dB, got {:.3} dB",
        result.mean_insertion_loss_db
    );
    assert!(
        result.max_insertion_loss_db <= 0.80,
        "Worst-case insertion loss must be <= 0.80 dB, got {:.3} dB",
        result.max_insertion_loss_db
    );

    assert!(
        result.throughput_sweeps_per_sec > 0.0,
        "Throughput must be positive, got {:.2} sweeps/sec",
        result.throughput_sweeps_per_sec
    );
}
