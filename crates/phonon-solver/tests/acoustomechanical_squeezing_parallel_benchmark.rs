//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for quantum cavity acoustomechanical squeezing across multi-threaded Rayon workers.

use phonon_solver::quantum_cavity_acoustomechanics::AcoustomechanicalBenchmarkRunner;

#[test]
fn test_10k_acoustomechanical_squeezing_parallel_sweep() {
    let cycles = 10_000;
    let result = AcoustomechanicalBenchmarkRunner::run_benchmark(cycles);

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

    // Assert ponderomotive squeezing >= 10.0 dB
    assert!(
        result.mean_squeezing_db >= 10.0,
        "Mean ponderomotive squeezing must be >= 10.0 dB, got {:.2} dB",
        result.mean_squeezing_db
    );
    assert!(
        result.min_squeezing_db >= 10.0,
        "Worst-case ponderomotive squeezing must be >= 10.0 dB, got {:.2} dB",
        result.min_squeezing_db
    );

    // Assert QND measurement fidelity >= 98.0%
    assert!(
        result.mean_qnd_fidelity >= 0.980,
        "Mean QND fidelity must be >= 98.0%, got {:.4}%",
        result.mean_qnd_fidelity * 100.0
    );
    assert!(
        result.min_qnd_fidelity >= 0.980,
        "Worst-case QND fidelity must be >= 98.0%, got {:.4}%",
        result.min_qnd_fidelity * 100.0
    );

    // Assert mechanical decoherence rate <= 10.0 Hz
    assert!(
        result.mean_decoherence_rate_hz <= 10.0,
        "Mean decoherence rate must be <= 10.0 Hz, got {:.3} Hz",
        result.mean_decoherence_rate_hz
    );
    assert!(
        result.max_decoherence_rate_hz <= 10.0,
        "Worst-case decoherence rate must be <= 10.0 Hz, got {:.3} Hz",
        result.max_decoherence_rate_hz
    );

    // Assert intracavity photon number >= 5.0e5
    assert!(
        result.mean_photons >= 5.0e5,
        "Mean intracavity photon number must be >= 5.0e5, got {:.2e}",
        result.mean_photons
    );
    assert!(
        result.min_photons >= 5.0e5,
        "Worst-case intracavity photon number must be >= 5.0e5, got {:.2e}",
        result.min_photons
    );

    // Assert backaction evasion purity >= 95.0%
    assert!(
        result.mean_bae_purity >= 0.950,
        "Mean BAE purity must be >= 95.0%, got {:.4}%",
        result.mean_bae_purity * 100.0
    );
    assert!(
        result.min_bae_purity >= 0.950,
        "Worst-case BAE purity must be >= 95.0%, got {:.4}%",
        result.min_bae_purity * 100.0
    );

    assert!(
        result.throughput_sweeps_per_sec > 0.0,
        "Throughput must be positive, got {:.2} sweeps/sec",
        result.throughput_sweeps_per_sec
    );
}
