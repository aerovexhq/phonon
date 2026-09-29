//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for topological Majorana braiding across multi-threaded Rayon workers.

use phonon_solver::topological_majorana_braiding::MajoranaBraidingBenchmarkRunner;

#[test]
fn test_10k_majorana_braiding_parallel_sweep() {
    let cycles = 10_000;
    let result = MajoranaBraidingBenchmarkRunner::run_benchmark(cycles);

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

    // Assert braiding gate fidelity >= 99.90%
    assert!(
        result.mean_braiding_fidelity >= 0.9990,
        "Mean braiding gate fidelity must be >= 99.90%, got {:.4}%",
        result.mean_braiding_fidelity * 100.0
    );
    assert!(
        result.min_braiding_fidelity >= 0.9990,
        "Worst-case braiding gate fidelity must be >= 99.90%, got {:.4}%",
        result.min_braiding_fidelity * 100.0
    );

    // Assert non-Abelian phase error <= 1.0e-4 rad
    assert!(
        result.mean_phase_error_rad <= 1.0e-4,
        "Mean non-Abelian phase error must be <= 1.0e-4 rad, got {:.3e} rad",
        result.mean_phase_error_rad
    );
    assert!(
        result.max_phase_error_rad <= 1.0e-4,
        "Worst-case non-Abelian phase error must be <= 1.0e-4 rad, got {:.3e} rad",
        result.max_phase_error_rad
    );

    // Assert parity readout contrast >= 95.0%
    assert!(
        result.mean_parity_contrast >= 0.950,
        "Mean parity readout contrast must be >= 95.0%, got {:.2}%",
        result.mean_parity_contrast * 100.0
    );
    assert!(
        result.min_parity_contrast >= 0.950,
        "Worst-case parity readout contrast must be >= 95.0%, got {:.2}%",
        result.min_parity_contrast * 100.0
    );

    // Assert braiding cycle period <= 50.0 ns
    assert!(
        result.mean_braiding_period_ns <= 50.0,
        "Mean braiding cycle period must be <= 50.0 ns, got {:.2} ns",
        result.mean_braiding_period_ns
    );
    assert!(
        result.max_braiding_period_ns <= 50.0,
        "Worst-case braiding cycle period must be <= 50.0 ns, got {:.2} ns",
        result.max_braiding_period_ns
    );

    assert!(
        result.throughput_sweeps_per_sec > 0.0,
        "Throughput must be positive, got {:.2} sweeps/sec",
        result.throughput_sweeps_per_sec
    );
}
