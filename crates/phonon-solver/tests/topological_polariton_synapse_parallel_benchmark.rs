#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for the Phonon Universal Multi-Scale Visual Studio Autonomous Acoustically Driven
//! Topological Polariton Neural Network Synapse & Optical Vector Engine across multi-threaded Rayon workers.

use phonon_solver::topological_polariton_synapse::TopologicalPolaritonSynapseBenchmarkRunner;

#[test]
fn test_10k_topological_polariton_synapse_parallel_sweep() {
    let cycles = 10_000;
    let result = TopologicalPolaritonSynapseBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 237 Phonon Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Topological Polariton Neural Network Synapse & Optical Vector Engine Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Synaptic Weight Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_synaptic_weight_fidelity,
        result.min_synaptic_weight_fidelity,
        result.max_synaptic_weight_fidelity
    );
    println!(
        "Polariton State Retention Fraction: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_polariton_state_retention_fraction,
        result.min_polariton_state_retention_fraction,
        result.max_polariton_state_retention_fraction
    );
    println!(
        "Topological Protection Gap (MHz): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_topological_protection_gap_mhz,
        result.min_topological_protection_gap_mhz,
        result.max_topological_protection_gap_mhz
    );
    println!(
        "Inter-Synapse Crosstalk Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_inter_synapse_crosstalk_isolation_db,
        result.min_inter_synapse_crosstalk_isolation_db,
        result.max_inter_synapse_crosstalk_isolation_db
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
        "Physical compliance must be 100%, got {:.2}%",
        result.physical_compliance_fraction * 100.0
    );

    // Verify all mean metrics strictly meet the roadmap design specifications
    assert!(
        result.mean_synaptic_weight_fidelity >= 0.9980,
        "Mean synaptic weight fidelity must be >= 0.9980, got {:.6}",
        result.mean_synaptic_weight_fidelity
    );
    assert!(
        result.min_synaptic_weight_fidelity >= 0.9980,
        "Min synaptic weight fidelity must be >= 0.9980, got {:.6}",
        result.min_synaptic_weight_fidelity
    );

    assert!(
        result.mean_polariton_state_retention_fraction >= 0.9970,
        "Mean polariton state retention fraction must be >= 0.9970, got {:.6}",
        result.mean_polariton_state_retention_fraction
    );
    assert!(
        result.min_polariton_state_retention_fraction >= 0.9970,
        "Min polariton state retention fraction must be >= 0.9970, got {:.6}",
        result.min_polariton_state_retention_fraction
    );

    assert!(
        result.mean_topological_protection_gap_mhz >= 45.0,
        "Mean topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        result.mean_topological_protection_gap_mhz
    );
    assert!(
        result.min_topological_protection_gap_mhz >= 45.0,
        "Min topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        result.min_topological_protection_gap_mhz
    );

    assert!(
        result.mean_inter_synapse_crosstalk_isolation_db >= 55.0,
        "Mean inter-synapse crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        result.mean_inter_synapse_crosstalk_isolation_db
    );
    assert!(
        result.min_inter_synapse_crosstalk_isolation_db >= 55.0,
        "Min inter-synapse crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        result.min_inter_synapse_crosstalk_isolation_db
    );

    assert!(
        result.mean_topological_mode_dephasing_rate_hz <= 12.0,
        "Mean topological mode dephasing rate must be <= 12.0 Hz, got {:.4} Hz",
        result.mean_topological_mode_dephasing_rate_hz
    );
    assert!(
        result.max_topological_mode_dephasing_rate_hz <= 12.0,
        "Max topological mode dephasing rate must be <= 12.0 Hz, got {:.4} Hz",
        result.max_topological_mode_dephasing_rate_hz
    );
}
