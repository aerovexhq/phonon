#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for quantum acoustic higher-order axion electrodynamics and chiral quadrupole-hinge
//! polariton circulators across multi-threaded Rayon workers.

use phonon_solver::hotp_axion_hinge_circulator::HotpAxionHingeBenchmarkRunner;

#[test]
fn test_10k_hotp_axion_hinge_circulator_parallel_sweep() {
    let cycles = 10_000;
    let result = HotpAxionHingeBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 166 Quantum Acoustic Higher-Order Axion Electrodynamics & Chiral Quadrupole-Hinge Polariton Circulators Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Hinge Polariton Transmission Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_hinge_polariton_transmission_fidelity,
        result.min_hinge_polariton_transmission_fidelity,
        result.max_hinge_polariton_transmission_fidelity
    );
    println!(
        "Higher-Order Topological Gap (MHz): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_higher_order_topological_gap_mhz,
        result.min_higher_order_topological_gap_mhz,
        result.max_higher_order_topological_gap_mhz
    );
    println!(
        "Dynamic Non-Reciprocal Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_dynamic_non_reciprocal_isolation_db,
        result.min_dynamic_non_reciprocal_isolation_db,
        result.max_dynamic_non_reciprocal_isolation_db
    );
    println!(
        "Inter-Hinge Crosstalk Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_inter_hinge_crosstalk_isolation_db,
        result.min_inter_hinge_crosstalk_isolation_db,
        result.max_inter_hinge_crosstalk_isolation_db
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
        result.min_hinge_polariton_transmission_fidelity >= 0.9980,
        "Minimum hinge polariton transmission fidelity must be >= 0.9980, got {:.6}",
        result.min_hinge_polariton_transmission_fidelity
    );
    assert!(
        result.min_higher_order_topological_gap_mhz >= 46.0,
        "Minimum higher-order topological gap must be >= 46.0 MHz, got {:.4} MHz",
        result.min_higher_order_topological_gap_mhz
    );
    assert!(
        result.min_dynamic_non_reciprocal_isolation_db >= 54.0,
        "Minimum dynamic non-reciprocal isolation must be >= 54.0 dB, got {:.4} dB",
        result.min_dynamic_non_reciprocal_isolation_db
    );
    assert!(
        result.min_inter_hinge_crosstalk_isolation_db >= 53.0,
        "Minimum inter-hinge crosstalk isolation must be >= 53.0 dB, got {:.4} dB",
        result.min_inter_hinge_crosstalk_isolation_db
    );
    assert!(
        result.max_topological_mode_dephasing_rate_hz <= 15.0,
        "Maximum topological mode dephasing rate must be <= 15.0 Hz, got {:.4} Hz",
        result.max_topological_mode_dephasing_rate_hz
    );
}
