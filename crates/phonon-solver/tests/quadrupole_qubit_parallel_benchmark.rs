#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for the Phonon Universal Multi-Scale Visual Studio Autonomous Second-Order Topological
//! Quadrupole Insulator & Corner-State Qubit Engine across multi-threaded Rayon workers.

use phonon_solver::quadrupole_qubit::QuadrupoleQubitBenchmarkRunner;

#[test]
fn test_10k_quadrupole_qubit_parallel_sweep() {
    let cycles = 10_000;
    let result = QuadrupoleQubitBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 218 Phonon Universal Multi-Scale Visual Studio Autonomous Second-Order Topological Quadrupole Insulator & Corner-State Qubit Engine Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Corner Localization Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_corner_localization_fidelity,
        result.min_corner_localization_fidelity,
        result.max_corner_localization_fidelity
    );
    println!(
        "Qubit State Retention Fraction: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_qubit_state_retention_fraction,
        result.min_qubit_state_retention_fraction,
        result.max_qubit_state_retention_fraction
    );
    println!(
        "Topological Protection Gap (MHz): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_topological_protection_gap_mhz,
        result.min_topological_protection_gap_mhz,
        result.max_topological_protection_gap_mhz
    );
    println!(
        "Inter-Corner Crosstalk Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_inter_corner_crosstalk_isolation_db,
        result.min_inter_corner_crosstalk_isolation_db,
        result.max_inter_corner_crosstalk_isolation_db
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
        result.min_corner_localization_fidelity >= 0.9980,
        "Minimum corner localization fidelity must be >= 0.9980, got {:.6}",
        result.min_corner_localization_fidelity
    );
    assert!(
        result.min_qubit_state_retention_fraction >= 0.9970,
        "Minimum qubit state retention fraction must be >= 0.9970, got {:.6}",
        result.min_qubit_state_retention_fraction
    );
    assert!(
        result.min_topological_protection_gap_mhz >= 45.0,
        "Minimum topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        result.min_topological_protection_gap_mhz
    );
    assert!(
        result.min_inter_corner_crosstalk_isolation_db >= 55.0,
        "Minimum inter-corner crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        result.min_inter_corner_crosstalk_isolation_db
    );
    assert!(
        result.max_topological_mode_dephasing_rate_hz <= 12.0,
        "Maximum topological mode dephasing rate must be <= 12.0 Hz, got {:.4} Hz",
        result.max_topological_mode_dephasing_rate_hz
    );
}
