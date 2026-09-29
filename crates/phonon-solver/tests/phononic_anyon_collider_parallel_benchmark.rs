#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for quantum phononic non-Abelian anyon colliders across multi-threaded Rayon workers.

use phonon_solver::phononic_anyon_collider::ColliderBenchmarkRunner;

#[test]
fn test_10k_phononic_anyon_collider_parallel_sweep() {
    let cycles = 10_000;
    let result = ColliderBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 122 Quantum Phononic Anyon Collider Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!("Physical Compliance Fraction: {:.2}%", result.physical_compliance_fraction * 100.0);
    println!("Collision Visibility: mean = {:.4}, min = {:.4}", result.mean_collision_visibility, result.min_collision_visibility);
    println!("Noise Suppression (dB): mean = {:.2}, min = {:.2}", result.mean_cross_correlation_noise_suppression_db, result.min_cross_correlation_noise_suppression_db);
    println!("Braiding Phase Error (rad): mean = {:.4e}, max = {:.4e}", result.mean_braiding_phase_error_rad, result.max_braiding_phase_error_rad);
    println!("Topological Parity Readout Fidelity: mean = {:.5}, min = {:.5}", result.mean_topological_parity_readout_fidelity, result.min_topological_parity_readout_fidelity);
    println!("Anyonic Fano Factor: mean = {:.4}, max = {:.4}", result.mean_anyonic_fano_factor, result.max_anyonic_fano_factor);

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

    // Assert collision visibility >= 0.920 (92.0%)
    assert!(
        result.mean_collision_visibility >= 0.920,
        "Mean collision visibility must be >= 0.920, got {:.4}",
        result.mean_collision_visibility
    );
    assert!(
        result.min_collision_visibility >= 0.920,
        "Worst-case collision visibility must be >= 0.920, got {:.4}",
        result.min_collision_visibility
    );

    // Assert cross-correlation noise suppression >= 25.0 dB
    assert!(
        result.mean_cross_correlation_noise_suppression_db >= 25.0,
        "Mean cross-correlation noise suppression must be >= 25.0 dB, got {:.2} dB",
        result.mean_cross_correlation_noise_suppression_db
    );
    assert!(
        result.min_cross_correlation_noise_suppression_db >= 25.0,
        "Worst-case cross-correlation noise suppression must be >= 25.0 dB, got {:.2} dB",
        result.min_cross_correlation_noise_suppression_db
    );

    // Assert braiding phase error <= 1.0e-4 rad
    assert!(
        result.mean_braiding_phase_error_rad <= 1.0e-4,
        "Mean braiding phase error must be <= 1.0e-4 rad, got {:.4e} rad",
        result.mean_braiding_phase_error_rad
    );
    assert!(
        result.max_braiding_phase_error_rad <= 1.0e-4,
        "Worst-case braiding phase error must be <= 1.0e-4 rad, got {:.4e} rad",
        result.max_braiding_phase_error_rad
    );

    // Assert topological parity readout fidelity >= 0.998 (99.8%)
    assert!(
        result.mean_topological_parity_readout_fidelity >= 0.998,
        "Mean topological parity readout fidelity must be >= 0.998, got {:.5}",
        result.mean_topological_parity_readout_fidelity
    );
    assert!(
        result.min_topological_parity_readout_fidelity >= 0.998,
        "Worst-case topological parity readout fidelity must be >= 0.998, got {:.5}",
        result.min_topological_parity_readout_fidelity
    );

    // Assert anyonic Fano factor <= 0.35
    assert!(
        result.mean_anyonic_fano_factor <= 0.35,
        "Mean anyonic Fano factor must be <= 0.35, got {:.4}",
        result.mean_anyonic_fano_factor
    );
    assert!(
        result.max_anyonic_fano_factor <= 0.35,
        "Worst-case anyonic Fano factor must be <= 0.35, got {:.4}",
        result.max_anyonic_fano_factor
    );
}
