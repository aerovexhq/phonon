//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for non-Hermitian topological acoustic lasers across multi-threaded Rayon workers.

use phonon_solver::non_hermitian_acoustic_laser::NonHermitianLaserBenchmarkRunner;

#[test]
fn test_10k_non_hermitian_laser_parallel_sweep() {
    let cycles = 10_000;
    let result = NonHermitianLaserBenchmarkRunner::run_benchmark(cycles);

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

    // Assert skin mode localization >= 92.0%
    assert!(
        result.mean_skin_localization >= 0.920,
        "Mean skin mode localization must be >= 92.0%, got {:.2}%",
        result.mean_skin_localization * 100.0
    );
    assert!(
        result.min_skin_localization >= 0.920,
        "Worst-case skin mode localization must be >= 92.0%, got {:.2}%",
        result.min_skin_localization * 100.0
    );

    // Assert SMSR >= 35.0 dB
    assert!(
        result.mean_laser_smsr_db >= 35.0,
        "Mean laser SMSR must be >= 35.0 dB, got {:.2} dB",
        result.mean_laser_smsr_db
    );
    assert!(
        result.min_laser_smsr_db >= 35.0,
        "Worst-case laser SMSR must be >= 35.0 dB, got {:.2} dB",
        result.min_laser_smsr_db
    );

    // Assert laser output power >= 15.0 mW
    assert!(
        result.mean_laser_output_power_mw >= 15.0,
        "Mean laser output power must be >= 15.0 mW, got {:.2} mW",
        result.mean_laser_output_power_mw
    );
    assert!(
        result.min_laser_output_power_mw >= 15.0,
        "Worst-case laser output power must be >= 15.0 mW, got {:.2} mW",
        result.min_laser_output_power_mw
    );

    // Assert non-reciprocal isolation >= 30.0 dB
    assert!(
        result.mean_non_reciprocal_isolation_db >= 30.0,
        "Mean non-reciprocal isolation must be >= 30.0 dB, got {:.2} dB",
        result.mean_non_reciprocal_isolation_db
    );
    assert!(
        result.min_non_reciprocal_isolation_db >= 30.0,
        "Worst-case non-reciprocal isolation must be >= 30.0 dB, got {:.2} dB",
        result.min_non_reciprocal_isolation_db
    );

    // Assert corner mode fidelity >= 0.950
    assert!(
        result.mean_corner_fidelity >= 0.950,
        "Mean corner mode fidelity must be >= 0.950, got {:.4}",
        result.mean_corner_fidelity
    );
    assert!(
        result.min_corner_fidelity >= 0.950,
        "Worst-case corner mode fidelity must be >= 0.950, got {:.4}",
        result.min_corner_fidelity
    );

    assert!(
        result.throughput_sweeps_per_sec > 0.0,
        "Throughput must be positive, got {:.2} sweeps/sec",
        result.throughput_sweeps_per_sec
    );
}
