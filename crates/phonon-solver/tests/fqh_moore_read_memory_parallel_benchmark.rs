#![deny(unsafe_code)]

//! Large-scale parallel multi-physics validation benchmark for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven
//! Fractional Quantum Hall Moore-Read Anyon Quantum Memory
//! & Surface Code Router Engine (Phase 293) across multi-core Rayon threads.

use phonon_solver::fqh_moore_read_memory::FqhMooreReadMemoryBenchmarkRunner;

#[test]
fn test_parallel_10k_fqh_moore_read_memory_benchmark() {
    let cycles = 10_000;
    let result = FqhMooreReadMemoryBenchmarkRunner::run_benchmark(cycles);

    println!(
        "Phase 293 Benchmark Completed: {} cycles in {:.6} s ({:.2} sweeps/sec)",
        result.total_cycles, result.elapsed_seconds, result.throughput_sweeps_per_sec
    );
    println!(
        "Memory Fidelity: mean={:.6}, min={:.6}, max={:.6}",
        result.mean_memory_fidelity,
        result.min_memory_fidelity,
        result.max_memory_fidelity
    );
    println!(
        "Pfaffian State Retention Fraction: mean={:.6}, min={:.6}, max={:.6}",
        result.mean_pfaffian_state_retention_fraction,
        result.min_pfaffian_state_retention_fraction,
        result.max_pfaffian_state_retention_fraction
    );
    println!(
        "Topological Protection Gap (MHz): mean={:.4}, min={:.4}, max={:.4}",
        result.mean_topological_protection_gap_mhz,
        result.min_topological_protection_gap_mhz,
        result.max_topological_protection_gap_mhz
    );
    println!(
        "Inter-Node Crosstalk Isolation (dB): mean={:.4}, min={:.4}, max={:.4}",
        result.mean_inter_node_crosstalk_isolation_db,
        result.min_inter_node_crosstalk_isolation_db,
        result.max_inter_node_crosstalk_isolation_db
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
        "Physical compliance must be 100.0% across all 10,000 parallel parameter sweeps"
    );

    // Validate multi-physics metric bounds
    assert!(result.mean_memory_fidelity >= 0.9980);
    assert!(result.min_memory_fidelity >= 0.9980);
    assert!(result.max_memory_fidelity <= 0.99995);

    assert!(result.mean_pfaffian_state_retention_fraction >= 0.9970);
    assert!(result.min_pfaffian_state_retention_fraction >= 0.9970);
    assert!(result.max_pfaffian_state_retention_fraction <= 0.99990);

    assert!(result.mean_topological_protection_gap_mhz >= 45.0);
    assert!(result.min_topological_protection_gap_mhz >= 45.0);
    assert!(result.max_topological_protection_gap_mhz <= 160.0);

    assert!(result.mean_inter_node_crosstalk_isolation_db >= 55.0);
    assert!(result.min_inter_node_crosstalk_isolation_db >= 55.0);
    assert!(result.max_inter_node_crosstalk_isolation_db <= 115.0);

    assert!(result.mean_topological_mode_dephasing_rate_hz <= 12.0);
    assert!(result.min_topological_mode_dephasing_rate_hz >= 0.50);
    assert!(result.max_topological_mode_dephasing_rate_hz <= 12.0);
}
