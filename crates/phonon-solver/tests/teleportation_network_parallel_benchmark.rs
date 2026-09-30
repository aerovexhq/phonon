#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for the Phonon Universal Multi-Scale Visual Studio Autonomous Non-Abelian Topological
//! Quantum State Teleportation Network Engine across multi-threaded Rayon workers.

use phonon_solver::teleportation_network::TeleportationNetworkBenchmarkRunner;

#[test]
fn test_10k_teleportation_network_parallel_sweep() {
    let cycles = 10_000;
    let result = TeleportationNetworkBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 224 Phonon Universal Multi-Scale Visual Studio Autonomous Non-Abelian Topological Quantum State Teleportation Network Engine Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Teleportation Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_teleportation_fidelity,
        result.min_teleportation_fidelity,
        result.max_teleportation_fidelity
    );
    println!(
        "Network State Retention Fraction: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_network_state_retention_fraction,
        result.min_network_state_retention_fraction,
        result.max_network_state_retention_fraction
    );
    println!(
        "Topological Protection Gap (MHz): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_topological_protection_gap_mhz,
        result.min_topological_protection_gap_mhz,
        result.max_topological_protection_gap_mhz
    );
    println!(
        "Inter-Node Crosstalk Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_inter_node_crosstalk_isolation_db,
        result.min_inter_node_crosstalk_isolation_db,
        result.max_inter_node_crosstalk_isolation_db
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
        "Physical compliance must be exactly 100.0%, got {:.2}%",
        result.physical_compliance_fraction * 100.0
    );

    // Verify mean and extreme bounds against design specifications
    assert!(
        result.mean_teleportation_fidelity >= 0.9980,
        "Mean teleportation fidelity must be >= 0.9980"
    );
    assert!(
        result.min_teleportation_fidelity >= 0.9980,
        "Minimum teleportation fidelity must be >= 0.9980"
    );

    assert!(
        result.mean_network_state_retention_fraction >= 0.9970,
        "Mean network state retention fraction must be >= 0.9970"
    );
    assert!(
        result.min_network_state_retention_fraction >= 0.9970,
        "Minimum network state retention fraction must be >= 0.9970"
    );

    assert!(
        result.mean_topological_protection_gap_mhz >= 45.0,
        "Mean topological protection gap must be >= 45.0 MHz"
    );
    assert!(
        result.min_topological_protection_gap_mhz >= 45.0,
        "Minimum topological protection gap must be >= 45.0 MHz"
    );

    assert!(
        result.mean_inter_node_crosstalk_isolation_db >= 55.0,
        "Mean inter-node crosstalk isolation must be >= 55.0 dB"
    );
    assert!(
        result.min_inter_node_crosstalk_isolation_db >= 55.0,
        "Minimum inter-node crosstalk isolation must be >= 55.0 dB"
    );

    assert!(
        result.mean_topological_mode_dephasing_rate_hz <= 12.0,
        "Mean topological mode dephasing rate must be <= 12.0 Hz"
    );
    assert!(
        result.max_topological_mode_dephasing_rate_hz <= 12.0,
        "Maximum topological mode dephasing rate must be <= 12.0 Hz"
    );
}
