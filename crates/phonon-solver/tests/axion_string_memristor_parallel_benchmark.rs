#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for quantum acoustic non-Abelian chiral topological axion string-vortex
//! entanglement networks and chiral gauge-symmetric quantum memristors across
//! multi-threaded Rayon workers.

use phonon_solver::axion_string_memristor::AxionStringBenchmarkRunner;

#[test]
fn test_10k_axion_string_memristor_parallel_sweep() {
    let cycles = 10_000;
    let result = AxionStringBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 177 Quantum Acoustic Non-Abelian Chiral Topological Axion String-Vortex Entanglement Networks & Chiral Gauge-Symmetric Quantum Memristors Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Memristive Retention Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_memristive_retention_fidelity,
        result.min_memristive_retention_fidelity,
        result.max_memristive_retention_fidelity
    );
    println!(
        "String-Vortex State Retention Fraction: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_string_vortex_state_retention_fraction,
        result.min_string_vortex_state_retention_fraction,
        result.max_string_vortex_state_retention_fraction
    );
    println!(
        "Topological Protection Gap (MHz): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_topological_protection_gap_mhz,
        result.min_topological_protection_gap_mhz,
        result.max_topological_protection_gap_mhz
    );
    println!(
        "Inter-String Crosstalk Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_inter_string_crosstalk_isolation_db,
        result.min_inter_string_crosstalk_isolation_db,
        result.max_inter_string_crosstalk_isolation_db
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
        result.min_memristive_retention_fidelity >= 0.9980,
        "Minimum memristive retention fidelity must be >= 0.9980, got {:.6}",
        result.min_memristive_retention_fidelity
    );
    assert!(
        result.min_string_vortex_state_retention_fraction >= 0.9970,
        "Minimum string-vortex state retention fraction must be >= 0.9970, got {:.6}",
        result.min_string_vortex_state_retention_fraction
    );
    assert!(
        result.min_topological_protection_gap_mhz >= 45.0,
        "Minimum topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        result.min_topological_protection_gap_mhz
    );
    assert!(
        result.min_inter_string_crosstalk_isolation_db >= 54.0,
        "Minimum inter-string crosstalk isolation must be >= 54.0 dB, got {:.4} dB",
        result.min_inter_string_crosstalk_isolation_db
    );
    assert!(
        result.max_topological_mode_dephasing_rate_hz <= 12.0,
        "Maximum topological mode dephasing rate must be <= 12.0 Hz, got {:.4} Hz",
        result.max_topological_mode_dephasing_rate_hz
    );
}
