#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for quantum acoustic higher-order topological quadrupole-octupole superlattices
//! and non-Hermitian corner metasurfaces across multi-threaded Rayon workers.

use phonon_solver::hotp_quadrupole_octupole_metasurface::HotpBenchmarkRunner;

#[test]
fn test_10k_hotp_quadrupole_octupole_parallel_sweep() {
    let cycles = 10_000;
    let result = HotpBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 162 Quantum Acoustic Higher-Order Topological Quadrupole-Octupole Superlattices & Non-Hermitian Corner Metasurfaces Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Corner State Localization Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_corner_state_localization_fidelity,
        result.min_corner_state_localization_fidelity,
        result.max_corner_state_localization_fidelity
    );
    println!(
        "Higher-Order Topological Gap (MHz): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_higher_order_topological_gap_mhz,
        result.min_higher_order_topological_gap_mhz,
        result.max_higher_order_topological_gap_mhz
    );
    println!(
        "Multipole Topological Charge: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_multipole_topological_charge,
        result.min_multipole_topological_charge,
        result.max_multipole_topological_charge
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
        result.min_corner_state_localization_fidelity >= 0.9980,
        "Minimum corner state localization fidelity must be >= 0.9980, got {:.6}",
        result.min_corner_state_localization_fidelity
    );
    assert!(
        result.min_higher_order_topological_gap_mhz >= 45.0,
        "Minimum higher-order topological gap must be >= 45.0 MHz, got {:.4}",
        result.min_higher_order_topological_gap_mhz
    );
    assert!(
        result.min_multipole_topological_charge >= 0.990,
        "Minimum multipole topological charge must be >= 0.990, got {:.6}",
        result.min_multipole_topological_charge
    );
    assert!(
        result.min_corner_to_bulk_crosstalk_isolation_db >= 54.0,
        "Minimum corner-to-bulk crosstalk isolation must be >= 54.0 dB, got {:.4}",
        result.min_corner_to_bulk_crosstalk_isolation_db
    );
    assert!(
        result.max_topological_mode_dephasing_rate_hz <= 15.0,
        "Maximum topological mode dephasing rate must be <= 15.0 Hz, got {:.4}",
        result.max_topological_mode_dephasing_rate_hz
    );
}
