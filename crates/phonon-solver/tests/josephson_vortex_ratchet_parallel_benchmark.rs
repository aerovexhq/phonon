//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for quantum acoustoelectric Josephson vortex ratchets across multi-threaded Rayon workers.

use phonon_solver::josephson_vortex_ratchet::JosephsonVortexRatchetBenchmarkRunner;

#[test]
fn test_10k_josephson_vortex_ratchet_parallel_sweep() {
    let cycles = 10_000;
    let result = JosephsonVortexRatchetBenchmarkRunner::run_benchmark(cycles);

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

    // Assert ratchet rectification efficiency >= 92.0%
    assert!(
        result.mean_rectification_efficiency >= 0.920,
        "Mean ratchet efficiency must be >= 92.0%, got {:.2}%",
        result.mean_rectification_efficiency * 100.0
    );
    assert!(
        result.min_rectification_efficiency >= 0.920,
        "Worst-case ratchet efficiency must be >= 92.0%, got {:.2}%",
        result.min_rectification_efficiency * 100.0
    );

    // Assert normalized soliton velocity >= 0.850
    assert!(
        result.mean_soliton_velocity >= 0.850,
        "Mean soliton velocity must be >= 0.850, got {:.4}",
        result.mean_soliton_velocity
    );
    assert!(
        result.min_soliton_velocity >= 0.850,
        "Worst-case soliton velocity must be >= 0.850, got {:.4}",
        result.min_soliton_velocity
    );

    // Assert threshold power <= 0.50 uW
    assert!(
        result.mean_threshold_power_uw <= 0.50,
        "Mean threshold power must be <= 0.50 uW, got {:.4} uW",
        result.mean_threshold_power_uw
    );
    assert!(
        result.max_threshold_power_uw <= 0.50,
        "Worst-case threshold power must be <= 0.50 uW, got {:.4} uW",
        result.max_threshold_power_uw
    );

    // Assert phase-slip locking precision <= 1.0e-9
    assert!(
        result.mean_locking_precision <= 1.0e-9,
        "Mean locking precision must be <= 1.0e-9, got {:.3e}",
        result.mean_locking_precision
    );
    assert!(
        result.max_locking_precision <= 1.0e-9,
        "Worst-case locking precision must be <= 1.0e-9, got {:.3e}",
        result.max_locking_precision
    );

    assert!(
        result.throughput_sweeps_per_sec > 0.0,
        "Throughput must be positive, got {:.2} sweeps/sec",
        result.throughput_sweeps_per_sec
    );
}
