//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for quantum acoustic waveguide QED across multi-threaded Rayon workers.

use phonon_solver::quantum_acoustic_waveguide::WaveguideQedBenchmarkRunner;

#[test]
fn test_10k_waveguide_qed_parallel_sweep() {
    let cycles = 10_000;
    let result = WaveguideQedBenchmarkRunner::run_benchmark(cycles);

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

    // Assert chiral directionality >= 95.0%
    assert!(
        result.mean_directionality >= 0.950,
        "Mean chiral directionality must be >= 95.0%, got {:.2}%",
        result.mean_directionality * 100.0
    );
    assert!(
        result.min_directionality >= 0.950,
        "Worst-case directionality must be >= 95.0%, got {:.2}%",
        result.min_directionality * 100.0
    );

    // Assert Purcell enhancement factor >= 80.0
    assert!(
        result.mean_purcell_factor >= 80.0,
        "Mean Purcell enhancement must be >= 80.0, got {:.2}",
        result.mean_purcell_factor
    );
    assert!(
        result.min_purcell_factor >= 80.0,
        "Worst-case Purcell enhancement must be >= 80.0, got {:.2}",
        result.min_purcell_factor
    );

    // Assert bound state lifetime extension >= 50.0x
    assert!(
        result.mean_lifetime_extension >= 50.0,
        "Mean bound state lifetime extension must be >= 50.0x, got {:.2}x",
        result.mean_lifetime_extension
    );
    assert!(
        result.min_lifetime_extension >= 50.0,
        "Worst-case lifetime extension must be >= 50.0x, got {:.2}x",
        result.min_lifetime_extension
    );

    // Assert entanglement concurrence >= 0.90
    assert!(
        result.mean_concurrence >= 0.900,
        "Mean entanglement concurrence must be >= 0.900, got {:.3}",
        result.mean_concurrence
    );
    assert!(
        result.min_concurrence >= 0.900,
        "Worst-case entanglement concurrence must be >= 0.900, got {:.3}",
        result.min_concurrence
    );
}
