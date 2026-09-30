#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for the Phonon Universal Multi-Scale Visual Studio Autonomous Acoustically
//! Driven Superconducting Flux Qubit Coupler & Ultra-Low Jitter Clock Engine across multi-threaded Rayon workers.

use phonon_solver::flux_qubit_coupler::FluxQubitCouplerBenchmarkRunner;

#[test]
fn test_10k_flux_qubit_coupler_parallel_sweep() {
    let cycles = 10_000;
    let result = FluxQubitCouplerBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 232 Phonon Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Superconducting Flux Qubit Coupler & Ultra-Low Jitter Clock Engine Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Flux Coupling Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_flux_coupling_fidelity,
        result.min_flux_coupling_fidelity,
        result.max_flux_coupling_fidelity
    );
    println!(
        "Clock State Retention Fraction: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_clock_state_retention_fraction,
        result.min_clock_state_retention_fraction,
        result.max_clock_state_retention_fraction
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
        "Physical compliance fraction must be 100.0%, got {:.4}%",
        result.physical_compliance_fraction * 100.0
    );

    // Assert metric bounds across all iterations
    assert!(
        result.min_flux_coupling_fidelity >= 0.9980,
        "Minimum flux coupling fidelity must be >= 0.9980, got {:.6}",
        result.min_flux_coupling_fidelity
    );
    assert!(
        result.min_clock_state_retention_fraction >= 0.9970,
        "Minimum clock state retention fraction must be >= 0.9970, got {:.6}",
        result.min_clock_state_retention_fraction
    );
    assert!(
        result.min_topological_protection_gap_mhz >= 45.0,
        "Minimum topological protection gap must be >= 45.0 MHz, got {:.4}",
        result.min_topological_protection_gap_mhz
    );
    assert!(
        result.min_inter_node_crosstalk_isolation_db >= 55.0,
        "Minimum inter-node crosstalk isolation must be >= 55.0 dB, got {:.4}",
        result.min_inter_node_crosstalk_isolation_db
    );
    assert!(
        result.max_topological_mode_dephasing_rate_hz <= 12.0,
        "Maximum topological mode dephasing rate must be <= 12.0 Hz, got {:.4}",
        result.max_topological_mode_dephasing_rate_hz
    );
}
