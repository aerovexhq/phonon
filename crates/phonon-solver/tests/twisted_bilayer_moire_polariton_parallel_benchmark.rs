#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for quantum acoustic twisted bilayer moire polariton superlattices and
//! flat-band phonon superconductors across multi-threaded Rayon workers.

use phonon_solver::twisted_bilayer_moire_polariton::TwistedBilayerMoireBenchmarkRunner;

#[test]
fn test_10k_twisted_bilayer_moire_polariton_parallel_sweep() {
    let cycles = 10_000;
    let result = TwistedBilayerMoireBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 158 Quantum Acoustic Twisted Bilayer Moire Polariton Superlattices & Flat-Band Phonon Superconductors Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Polariton Superconducting Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_polariton_superconducting_fidelity,
        result.min_polariton_superconducting_fidelity,
        result.max_polariton_superconducting_fidelity
    );
    println!(
        "Flat-Band Group Velocity (m/s): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_flat_band_group_velocity_mps,
        result.min_flat_band_group_velocity_mps,
        result.max_flat_band_group_velocity_mps
    );
    println!(
        "Tc Enhancement Factor: mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_tc_enhancement_factor,
        result.min_tc_enhancement_factor,
        result.max_tc_enhancement_factor
    );
    println!(
        "Inter-Valley Crosstalk Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_inter_valley_crosstalk_isolation_db,
        result.min_inter_valley_crosstalk_isolation_db,
        result.max_inter_valley_crosstalk_isolation_db
    );
    println!(
        "Magic-Angle Alignment Tolerance Fraction: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_magic_angle_alignment_tolerance_fraction,
        result.min_magic_angle_alignment_tolerance_fraction,
        result.max_magic_angle_alignment_tolerance_fraction
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
        result.min_polariton_superconducting_fidelity >= 0.9970,
        "Minimum polariton superconducting fidelity must be >= 0.9970, got {:.6}",
        result.min_polariton_superconducting_fidelity
    );
    assert!(
        result.max_flat_band_group_velocity_mps <= 150.0,
        "Maximum flat-band group velocity must be <= 150.0 m/s, got {:.4}",
        result.max_flat_band_group_velocity_mps
    );
    assert!(
        result.min_tc_enhancement_factor >= 4.50,
        "Minimum Tc enhancement factor must be >= 4.50, got {:.4}",
        result.min_tc_enhancement_factor
    );
    assert!(
        result.min_inter_valley_crosstalk_isolation_db >= 50.0,
        "Minimum inter-valley crosstalk isolation must be >= 50.0 dB, got {:.4}",
        result.min_inter_valley_crosstalk_isolation_db
    );
    assert!(
        result.min_magic_angle_alignment_tolerance_fraction >= 0.9980,
        "Minimum magic-angle alignment tolerance fraction must be >= 0.9980, got {:.6}",
        result.min_magic_angle_alignment_tolerance_fraction
    );
}
