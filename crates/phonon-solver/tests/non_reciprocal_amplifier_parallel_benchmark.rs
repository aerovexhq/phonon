//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for non-reciprocal topological phonon amplification and directional quantum routing.

use phonon_solver::non_reciprocal_phonon_amplifier::AmplifierBenchmarkRunner;

#[test]
fn test_10k_non_reciprocal_amplifier_parallel_sweep() {
    let cycles = 10_000;
    let result = AmplifierBenchmarkRunner::run_benchmark(cycles);

    assert_eq!(
        result.total_cycles, cycles,
        "Total benchmark cycles must equal requested 10,000"
    );

    println!(
        "Non-Reciprocal Amplifier 10k Sweep: gain={:.4} dB (min {:.4} dB), isolation={:.4} dB (min {:.4} dB), added_noise={:.6} (max {:.6}), BW={:.4} MHz (min {:.4} MHz), fidelity={:.6} (min {:.6}), compliance={:.1}%, throughput={:.2} sweeps/sec",
        result.mean_forward_gain_db,
        result.min_forward_gain_db,
        result.mean_backward_isolation_db,
        result.min_backward_isolation_db,
        result.mean_added_noise_photons,
        result.max_added_noise_photons,
        result.mean_bandwidth_mhz,
        result.min_bandwidth_mhz,
        result.mean_routing_fidelity,
        result.min_routing_fidelity,
        result.physical_compliance_fraction * 100.0,
        result.throughput_sweeps_per_sec,
    );

    // Verify 100% physical compliance across all parameter sweep variations
    assert!(
        (result.physical_compliance_fraction - 1.0).abs() < 1.0e-6,
        "Physical compliance fraction must be 100.0%, got {:.2}%",
        result.physical_compliance_fraction * 100.0
    );

    // Assert forward non-reciprocal acoustic gain >= 20.0 dB
    assert!(
        result.mean_forward_gain_db >= 20.0,
        "Mean forward gain must be >= 20.0 dB, got {:.4} dB",
        result.mean_forward_gain_db
    );
    assert!(
        result.min_forward_gain_db >= 20.0,
        "Worst-case forward gain must be >= 20.0 dB, got {:.4} dB",
        result.min_forward_gain_db
    );

    // Assert backward acoustic isolation >= 30.0 dB
    assert!(
        result.mean_backward_isolation_db >= 30.0,
        "Mean backward isolation must be >= 30.0 dB, got {:.4} dB",
        result.mean_backward_isolation_db
    );
    assert!(
        result.min_backward_isolation_db >= 30.0,
        "Worst-case backward isolation must be >= 30.0 dB, got {:.4} dB",
        result.min_backward_isolation_db
    );

    // Assert added noise quanta <= 0.50
    assert!(
        result.mean_added_noise_photons <= 0.50,
        "Mean added noise must be <= 0.50 quanta, got {:.6}",
        result.mean_added_noise_photons
    );
    assert!(
        result.max_added_noise_photons <= 0.50,
        "Worst-case added noise must be <= 0.50 quanta, got {:.6}",
        result.max_added_noise_photons
    );

    // Assert instantaneous amplification bandwidth >= 15.0 MHz
    assert!(
        result.mean_bandwidth_mhz >= 15.0,
        "Mean bandwidth must be >= 15.0 MHz, got {:.4} MHz",
        result.mean_bandwidth_mhz
    );
    assert!(
        result.min_bandwidth_mhz >= 15.0,
        "Worst-case bandwidth must be >= 15.0 MHz, got {:.4} MHz",
        result.min_bandwidth_mhz
    );

    // Assert directional quantum routing fidelity >= 0.960
    assert!(
        result.mean_routing_fidelity >= 0.960,
        "Mean routing fidelity must be >= 0.960, got {:.6}",
        result.mean_routing_fidelity
    );
    assert!(
        result.min_routing_fidelity >= 0.960,
        "Worst-case routing fidelity must be >= 0.960, got {:.6}",
        result.min_routing_fidelity
    );

    assert!(
        result.throughput_sweeps_per_sec > 0.0,
        "Throughput must be positive, got {:.2} sweeps/sec",
        result.throughput_sweeps_per_sec
    );
}
