#![deny(unsafe_code)]

//! Large-scale parallel multi-physics validation benchmark for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven
//! Floquet-Chern Parafermion Topological Quantum Processor & Surface Code Hub
//! Engine (Phase 295 Milestone) across multi-core Rayon threads.

use phonon_solver::floquet_parafermion_processor::FloquetParafermionProcessorBenchmarkRunner;

#[test]
fn test_parallel_10k_floquet_parafermion_processor_benchmark() {
    let cycles = 10_000;
    let result = FloquetParafermionProcessorBenchmarkRunner::run_benchmark(cycles);

    println!(
        "Phase 295 Benchmark Completed: {} cycles in {:.6} s ({:.2} sweeps/sec)",
        result.total_cycles, result.elapsed_seconds, result.throughput_sweeps_per_sec
    );
    println!(
        "Processor Fidelity: mean={:.6}, min={:.6}, max={:.6}",
        result.mean_processor_fidelity,
        result.min_processor_fidelity,
        result.max_processor_fidelity
    );
    println!(
        "Topological State Retention Fraction: mean={:.6}, min={:.6}, max={:.6}",
        result.mean_topological_state_retention_fraction,
        result.min_topological_state_retention_fraction,
        result.max_topological_state_retention_fraction
    );
    println!(
        "Topological Protection Gap (MHz): mean={:.4}, min={:.4}, max={:.4}",
        result.mean_topological_protection_gap_mhz,
        result.min_topological_protection_gap_mhz,
        result.max_topological_protection_gap_mhz
    );
    println!(
        "Inter-Qudit Crosstalk Isolation (dB): mean={:.4}, min={:.4}, max={:.4}",
        result.mean_inter_qudit_crosstalk_isolation_db,
        result.min_inter_qudit_crosstalk_isolation_db,
        result.max_inter_qudit_crosstalk_isolation_db
    );
    println!(
        "Topological Mode Dephasing Rate (Hz): mean={:.4}, min={:.4}, max={:.4}",
        result.mean_topological_mode_dephasing_rate_hz,
        result.min_topological_mode_dephasing_rate_hz,
        result.max_topological_mode_dephasing_rate_hz
    );
    println!(
        "Physical Compliance Fraction: {:.4}%",
        result.physical_compliance_fraction * 100.0
    );

    // Validate benchmark execution integrity
    assert_eq!(result.total_cycles, 10_000);
    assert!(result.elapsed_seconds > 0.0);
    assert!(result.throughput_sweeps_per_sec > 10_000.0);

    // Validate 100% compliance across all 10,000 parameter sweeps
    assert_eq!(
        result.physical_compliance_fraction, 1.0,
        "Physical compliance must be exactly 100.0% across all 10k sweep cycles"
    );

    // Validate target performance thresholds for means
    assert!(
        result.mean_processor_fidelity >= 0.9980,
        "Mean processor fidelity must be >= 0.9980, got {:.6}",
        result.mean_processor_fidelity
    );
    assert!(
        result.mean_topological_state_retention_fraction >= 0.9970,
        "Mean topological state retention fraction must be >= 0.9970, got {:.6}",
        result.mean_topological_state_retention_fraction
    );
    assert!(
        result.mean_topological_protection_gap_mhz >= 45.0,
        "Mean topological protection gap must be >= 45.0 MHz, got {:.4}",
        result.mean_topological_protection_gap_mhz
    );
    assert!(
        result.mean_inter_qudit_crosstalk_isolation_db >= 55.0,
        "Mean inter-qudit crosstalk isolation must be >= 55.0 dB, got {:.4}",
        result.mean_inter_qudit_crosstalk_isolation_db
    );
    assert!(
        result.mean_topological_mode_dephasing_rate_hz <= 12.0,
        "Mean topological mode dephasing rate must be <= 12.0 Hz, got {:.4}",
        result.mean_topological_mode_dephasing_rate_hz
    );

    // Validate absolute worst-case bounds across all 10k cycles
    assert!(
        result.min_processor_fidelity >= 0.9980,
        "Worst-case processor fidelity must be >= 0.9980, got {:.6}",
        result.min_processor_fidelity
    );
    assert!(
        result.min_topological_state_retention_fraction >= 0.9970,
        "Worst-case topological state retention fraction must be >= 0.9970, got {:.6}",
        result.min_topological_state_retention_fraction
    );
    assert!(
        result.min_topological_protection_gap_mhz >= 45.0,
        "Worst-case topological protection gap must be >= 45.0 MHz, got {:.4}",
        result.min_topological_protection_gap_mhz
    );
    assert!(
        result.min_inter_qudit_crosstalk_isolation_db >= 55.0,
        "Worst-case inter-qudit crosstalk isolation must be >= 55.0 dB, got {:.4}",
        result.min_inter_qudit_crosstalk_isolation_db
    );
    assert!(
        result.max_topological_mode_dephasing_rate_hz <= 12.0,
        "Worst-case topological mode dephasing rate must be <= 12.0 Hz, got {:.4}",
        result.max_topological_mode_dephasing_rate_hz
    );
}
