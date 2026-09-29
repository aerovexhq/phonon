#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for non-Hermitian higher-order topological phononic lasers and chiral
//! quadrupole acoustical frequency synthesizers across multi-threaded Rayon workers.

use phonon_solver::non_hermitian_quadrupole_laser::QuadrupoleLaserBenchmarkRunner;

#[test]
fn test_10k_non_hermitian_quadrupole_laser_parallel_sweep() {
    let cycles = 10_000;
    let result = QuadrupoleLaserBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 143 Non-Hermitian Higher-Order Topological Phononic Lasers & Chiral Quadrupole Acoustical Frequency Synthesizers Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Corner Mode Lasing Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_corner_mode_lasing_fidelity,
        result.min_corner_mode_lasing_fidelity,
        result.max_corner_mode_lasing_fidelity
    );
    println!(
        "Fractional Frequency Instability: mean = {:.4e}, min = {:.4e}, max = {:.4e}",
        result.mean_fractional_frequency_instability,
        result.min_fractional_frequency_instability,
        result.max_fractional_frequency_instability
    );
    println!(
        "Side Mode Suppression Ratio (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_side_mode_suppression_ratio_db,
        result.min_side_mode_suppression_ratio_db,
        result.max_side_mode_suppression_ratio_db
    );
    println!(
        "Topological Corner Mode Lifetime (ms): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_topological_corner_mode_lifetime_ms,
        result.min_topological_corner_mode_lifetime_ms,
        result.max_topological_corner_mode_lifetime_ms
    );
    println!(
        "PT-Symmetry Confinement Ratio: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_pt_symmetry_confinement_ratio,
        result.min_pt_symmetry_confinement_ratio,
        result.max_pt_symmetry_confinement_ratio
    );

    assert_eq!(
        result.total_cycles, cycles,
        "Total benchmark cycles must equal requested 10,000"
    );

    // Verify 100% physical compliance across all parameter sweep variations
    assert!(
        (result.physical_compliance_fraction - 1.0).abs() < 1.0e-6,
        "Physical compliance must be 100.0%, got {:.2}%",
        result.physical_compliance_fraction * 100.0
    );

    // Verify roadmap thresholds
    assert!(
        result.min_corner_mode_lasing_fidelity >= 0.9970,
        "Minimum corner mode lasing fidelity must be >= 0.9970"
    );
    assert!(
        result.max_fractional_frequency_instability <= 1.5e-12,
        "Maximum fractional frequency instability must be <= 1.5e-12"
    );
    assert!(
        result.min_side_mode_suppression_ratio_db >= 45.0,
        "Minimum side-mode suppression ratio must be >= 45.0 dB"
    );
    assert!(
        result.min_topological_corner_mode_lifetime_ms >= 80.0,
        "Minimum topological corner mode lifetime must be >= 80.0 ms"
    );
    assert!(
        result.min_pt_symmetry_confinement_ratio >= 0.920,
        "Minimum PT-symmetry confinement ratio must be >= 0.920"
    );
}
