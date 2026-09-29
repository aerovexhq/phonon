//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for topological phononic Floquet Weyl semimetals across multi-threaded Rayon workers.

use phonon_solver::topological_weyl_acoustics::WeylAcousticBenchmarkRunner;

#[test]
fn test_10k_weyl_acoustic_parallel_sweep() {
    let cycles = 10_000;
    let result = WeylAcousticBenchmarkRunner::run_benchmark(cycles);

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

    // Assert normalized Weyl point separation >= 0.350
    assert!(
        result.mean_weyl_separation >= 0.350,
        "Mean Weyl point separation must be >= 0.350, got {:.4}",
        result.mean_weyl_separation
    );
    assert!(
        result.min_weyl_separation >= 0.350,
        "Worst-case Weyl point separation must be >= 0.350, got {:.4}",
        result.min_weyl_separation
    );

    // Assert surface Fermi arc transmission >= 94.0%
    assert!(
        result.mean_fermi_arc_transmission >= 0.940,
        "Mean Fermi arc transmission must be >= 94.0%, got {:.2}%",
        result.mean_fermi_arc_transmission * 100.0
    );
    assert!(
        result.min_fermi_arc_transmission >= 0.940,
        "Worst-case Fermi arc transmission must be >= 94.0%, got {:.2}%",
        result.min_fermi_arc_transmission * 100.0
    );

    // Assert dislocation mode purity >= 96.0%
    assert!(
        result.mean_dislocation_purity >= 0.960,
        "Mean dislocation mode purity must be >= 96.0%, got {:.2}%",
        result.mean_dislocation_purity * 100.0
    );
    assert!(
        result.min_dislocation_purity >= 0.960,
        "Worst-case dislocation mode purity must be >= 96.0%, got {:.2}%",
        result.min_dislocation_purity * 100.0
    );

    // Assert bulk bandgap isolation >= 30.0 dB
    assert!(
        result.mean_bulk_isolation_db >= 30.0,
        "Mean bulk bandgap isolation must be >= 30.0 dB, got {:.2} dB",
        result.mean_bulk_isolation_db
    );
    assert!(
        result.min_bulk_isolation_db >= 30.0,
        "Worst-case bulk bandgap isolation must be >= 30.0 dB, got {:.2} dB",
        result.min_bulk_isolation_db
    );

    assert!(
        result.throughput_sweeps_per_sec > 0.0,
        "Throughput must be positive, got {:.2} sweeps/sec",
        result.throughput_sweeps_per_sec
    );
}
