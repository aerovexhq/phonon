#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for cavity quantum acoustomagnonic polariton condensation and chiral superfluid
//! spin-phonon lasers across multi-threaded Rayon workers.

use phonon_solver::acoustomagnonic_polariton_laser::PolaritonLaserBenchmarkRunner;

#[test]
fn test_10k_acoustomagnonic_polariton_laser_parallel_sweep() {
    let cycles = 10_000;
    let result = PolaritonLaserBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 149 Cavity Quantum Acoustomagnonic Polariton Condensation & Chiral Superfluid Spin-Phonon Lasers Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Polariton Condensation Threshold (uW): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_polariton_condensation_threshold_uw,
        result.min_polariton_condensation_threshold_uw,
        result.max_polariton_condensation_threshold_uw
    );
    println!(
        "Condensate Coherence Lifetime (us): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_condensate_coherence_lifetime_us,
        result.min_condensate_coherence_lifetime_us,
        result.max_condensate_coherence_lifetime_us
    );
    println!(
        "Side-Mode Suppression Ratio (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_side_mode_suppression_ratio_db,
        result.min_side_mode_suppression_ratio_db,
        result.max_side_mode_suppression_ratio_db
    );
    println!(
        "Linewidth Narrowing Factor: mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_linewidth_narrowing_factor,
        result.min_linewidth_narrowing_factor,
        result.max_linewidth_narrowing_factor
    );
    println!(
        "Polariton Superfluid Fraction: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_polariton_superfluid_fraction,
        result.min_polariton_superfluid_fraction,
        result.max_polariton_superfluid_fraction
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
        result.max_polariton_condensation_threshold_uw <= 15.0,
        "Maximum polariton condensation threshold must be <= 15.0 uW, got {:.4} uW",
        result.max_polariton_condensation_threshold_uw
    );
    assert!(
        result.min_condensate_coherence_lifetime_us >= 120.0,
        "Minimum condensate coherence lifetime must be >= 120.0 us, got {:.4} us",
        result.min_condensate_coherence_lifetime_us
    );
    assert!(
        result.min_side_mode_suppression_ratio_db >= 45.0,
        "Minimum side-mode suppression ratio must be >= 45.0 dB, got {:.4} dB",
        result.min_side_mode_suppression_ratio_db
    );
    assert!(
        result.min_linewidth_narrowing_factor >= 80.0,
        "Minimum linewidth narrowing factor must be >= 80.0x, got {:.4}x",
        result.min_linewidth_narrowing_factor
    );
    assert!(
        result.min_polariton_superfluid_fraction >= 0.850,
        "Minimum polariton superfluid fraction must be >= 0.850, got {:.6}",
        result.min_polariton_superfluid_fraction
    );
}
