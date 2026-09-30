#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for quantum acoustic non-Hermitian higher-order topological skin sensors
//! and chiral octupole phonon lasers across multi-threaded Rayon workers.

use phonon_solver::non_hermitian_skin_octupole_laser::SkinOctupoleBenchmarkRunner;

#[test]
fn test_10k_non_hermitian_skin_octupole_laser_parallel_sweep() {
    let cycles = 10_000;
    let result = SkinOctupoleBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 168 Quantum Acoustic Non-Hermitian Higher-Order Topological Skin Sensors & Chiral Octupole Phonon Lasers Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Corner Lasing Mode Purity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_corner_lasing_mode_purity,
        result.min_corner_lasing_mode_purity,
        result.max_corner_lasing_mode_purity
    );
    println!(
        "Skin Sensitivity Factor: mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_skin_sensitivity_factor,
        result.min_skin_sensitivity_factor,
        result.max_skin_sensitivity_factor
    );
    println!(
        "Higher-Order Skin Topological Gap (MHz): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_higher_order_skin_topological_gap_mhz,
        result.min_higher_order_skin_topological_gap_mhz,
        result.max_higher_order_skin_topological_gap_mhz
    );
    println!(
        "Corner-to-Bulk Crosstalk Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_corner_to_bulk_crosstalk_isolation_db,
        result.min_corner_to_bulk_crosstalk_isolation_db,
        result.max_corner_to_bulk_crosstalk_isolation_db
    );
    println!(
        "Topological Mode Dephasing Rate (Hz): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_topological_mode_dephasing_rate_hz,
        result.min_topological_mode_dephasing_rate_hz,
        result.max_topological_mode_dephasing_rate_hz
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
        result.min_corner_lasing_mode_purity >= 0.9980,
        "Minimum corner lasing mode purity must be >= 0.9980, got {:.6}",
        result.min_corner_lasing_mode_purity
    );
    assert!(
        result.min_skin_sensitivity_factor >= 95.0,
        "Minimum skin sensitivity factor must be >= 95.0, got {:.4}",
        result.min_skin_sensitivity_factor
    );
    assert!(
        result.min_higher_order_skin_topological_gap_mhz >= 48.0,
        "Minimum higher-order skin topological gap must be >= 48.0 MHz, got {:.4} MHz",
        result.min_higher_order_skin_topological_gap_mhz
    );
    assert!(
        result.min_corner_to_bulk_crosstalk_isolation_db >= 55.0,
        "Minimum corner-to-bulk crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        result.min_corner_to_bulk_crosstalk_isolation_db
    );
    assert!(
        result.max_topological_mode_dephasing_rate_hz <= 13.0,
        "Maximum topological mode dephasing rate must be <= 13.0 Hz, got {:.4} Hz",
        result.max_topological_mode_dephasing_rate_hz
    );
}
