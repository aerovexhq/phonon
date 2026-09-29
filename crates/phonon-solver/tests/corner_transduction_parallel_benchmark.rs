//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for Floquet second-order topological phononic corner states across multi-threaded Rayon workers.

use phonon_solver::floquet_corner_transduction::CornerTransductionBenchmarkRunner;

#[test]
fn test_10k_corner_transduction_parallel_sweep() {
    let cycles = 10_000;
    let result = CornerTransductionBenchmarkRunner::run_benchmark(cycles);

    assert_eq!(
        result.total_cycles, cycles,
        "Total benchmark cycles must equal requested 10,000"
    );

    println!(
        "Corner Transduction 10k Sweep: purity={:.4} (min {:.4}), eff={:.4} (min {:.4}), noise={:.4} (max {:.4}), Q={:.2e} (min {:.2e}), topo={:.3}, compliance={:.1}%, throughput={:.2} sweeps/sec",
        result.mean_purity,
        result.min_purity,
        result.mean_efficiency,
        result.min_efficiency,
        result.mean_noise_photons,
        result.max_noise_photons,
        result.mean_quality_factor,
        result.min_quality_factor,
        result.mean_topological_invariant,
        result.physical_compliance_fraction * 100.0,
        result.throughput_sweeps_per_sec,
    );

    // Verify 100% physical compliance across all parameter sweep variations
    assert!(
        (result.physical_compliance_fraction - 1.0).abs() < 1.0e-6,
        "Physical compliance fraction must be 100.0%, got {:.2}%",
        result.physical_compliance_fraction * 100.0
    );

    // Assert corner mode localization purity >= 96.0% (0.960)
    assert!(
        result.mean_purity >= 0.960,
        "Mean localization purity must be >= 0.960, got {:.4}",
        result.mean_purity
    );
    assert!(
        result.min_purity >= 0.960,
        "Worst-case localization purity must be >= 0.960, got {:.4}",
        result.min_purity
    );

    // Assert bidirectional transduction efficiency >= 45.0% (0.450)
    assert!(
        result.mean_efficiency >= 0.450,
        "Mean transduction efficiency must be >= 0.450, got {:.4}",
        result.mean_efficiency
    );
    assert!(
        result.min_efficiency >= 0.450,
        "Worst-case transduction efficiency must be >= 0.450, got {:.4}",
        result.min_efficiency
    );

    // Assert added noise photons <= 0.20
    assert!(
        result.mean_noise_photons <= 0.20,
        "Mean added noise photons must be <= 0.20, got {:.4}",
        result.mean_noise_photons
    );
    assert!(
        result.max_noise_photons <= 0.20,
        "Worst-case added noise photons must be <= 0.20, got {:.4}",
        result.max_noise_photons
    );

    // Assert corner acoustic quality factor >= 1.5e5
    assert!(
        result.mean_quality_factor >= 1.5e5,
        "Mean acoustic quality factor must be >= 1.5e5, got {:.2e}",
        result.mean_quality_factor
    );
    assert!(
        result.min_quality_factor >= 1.5e5,
        "Worst-case acoustic quality factor must be >= 1.5e5, got {:.2e}",
        result.min_quality_factor
    );

    // Assert quantized quadrupole topological invariant q_xy = 0.500
    assert!(
        (result.mean_topological_invariant - 0.500).abs() < 1.0e-6,
        "Mean quadrupole topological invariant must be 0.500, got {:.4}",
        result.mean_topological_invariant
    );
    assert!(
        (result.min_topological_invariant - 0.500).abs() < 1.0e-6,
        "Worst-case quadrupole topological invariant must be 0.500, got {:.4}",
        result.min_topological_invariant
    );

    assert!(
        result.throughput_sweeps_per_sec > 0.0,
        "Throughput must be positive, got {:.2} sweeps/sec",
        result.throughput_sweeps_per_sec
    );
}
