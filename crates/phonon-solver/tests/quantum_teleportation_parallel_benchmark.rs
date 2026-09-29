//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for superconducting optomechanical quantum teleportation across phononic crystal waveguides.

use phonon_solver::quantum_teleportation_waveguide::QuantumTeleportationBenchmarkRunner;

#[test]
fn test_10k_quantum_teleportation_parallel_sweep() {
    let cycles = 10_000;
    let result = QuantumTeleportationBenchmarkRunner::run_benchmark(cycles);

    assert_eq!(
        result.total_cycles, cycles,
        "Total benchmark cycles must equal requested 10,000"
    );

    println!(
        "Quantum Teleportation 10k Sweep: fid={:.4} (min {:.4}), purity={:.4} (min {:.4}), loss={:.4} dB/cm (max {:.4} dB/cm), T2={:.4} ms (min {:.4} ms), concurrence={:.4} (min {:.4}), compliance={:.1}%, throughput={:.2} sweeps/sec",
        result.mean_fidelity,
        result.min_fidelity,
        result.mean_distillation_purity,
        result.min_distillation_purity,
        result.mean_waveguide_loss_db_per_cm,
        result.max_waveguide_loss_db_per_cm,
        result.mean_quantum_memory_t2_ms,
        result.min_quantum_memory_t2_ms,
        result.mean_bell_concurrence,
        result.min_bell_concurrence,
        result.physical_compliance_fraction * 100.0,
        result.throughput_sweeps_per_sec,
    );

    // Verify 100% physical compliance across all parameter sweep variations
    assert!(
        (result.physical_compliance_fraction - 1.0).abs() < 1.0e-6,
        "Physical compliance fraction must be 100.0%, got {:.2}%",
        result.physical_compliance_fraction * 100.0
    );

    // Assert teleportation fidelity >= 85.0% (0.850)
    assert!(
        result.mean_fidelity >= 0.850,
        "Mean teleportation fidelity must be >= 0.850, got {:.4}",
        result.mean_fidelity
    );
    assert!(
        result.min_fidelity >= 0.850,
        "Worst-case teleportation fidelity must be >= 0.850, got {:.4}",
        result.min_fidelity
    );

    // Assert entanglement distillation purity >= 92.0% (0.920)
    assert!(
        result.mean_distillation_purity >= 0.920,
        "Mean distillation purity must be >= 0.920, got {:.4}",
        result.mean_distillation_purity
    );
    assert!(
        result.min_distillation_purity >= 0.920,
        "Worst-case distillation purity must be >= 0.920, got {:.4}",
        result.min_distillation_purity
    );

    // Assert waveguide acoustic propagation loss <= 0.050 dB/cm
    assert!(
        result.mean_waveguide_loss_db_per_cm <= 0.050,
        "Mean waveguide loss must be <= 0.050 dB/cm, got {:.4} dB/cm",
        result.mean_waveguide_loss_db_per_cm
    );
    assert!(
        result.max_waveguide_loss_db_per_cm <= 0.050,
        "Worst-case waveguide loss must be <= 0.050 dB/cm, got {:.4} dB/cm",
        result.max_waveguide_loss_db_per_cm
    );

    // Assert quantum memory coherence time T2 >= 1.00 ms
    assert!(
        result.mean_quantum_memory_t2_ms >= 1.00,
        "Mean memory coherence time T2 must be >= 1.00 ms, got {:.4} ms",
        result.mean_quantum_memory_t2_ms
    );
    assert!(
        result.min_quantum_memory_t2_ms >= 1.00,
        "Worst-case memory coherence time T2 must be >= 1.00 ms, got {:.4} ms",
        result.min_quantum_memory_t2_ms
    );

    // Assert remote Bell-state concurrence >= 0.800
    assert!(
        result.mean_bell_concurrence >= 0.800,
        "Mean Bell-state concurrence must be >= 0.800, got {:.4}",
        result.mean_bell_concurrence
    );
    assert!(
        result.min_bell_concurrence >= 0.800,
        "Worst-case Bell-state concurrence must be >= 0.800, got {:.4}",
        result.min_bell_concurrence
    );

    assert!(
        result.throughput_sweeps_per_sec > 0.0,
        "Throughput must be positive, got {:.2} sweeps/sec",
        result.throughput_sweeps_per_sec
    );
}
