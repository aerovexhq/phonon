#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for the Phonon Universal Multi-Scale Visual Studio Autonomous Photonic-Phononic
//! Quantum Memory Register and Non-Volatile Polariton Qubit Synthesizer across multi-threaded Rayon workers.

use phonon_solver::polariton_quantum_memory::PolaritonQuantumMemoryBenchmarkRunner;

#[test]
fn test_10k_polariton_quantum_memory_parallel_sweep() {
    let cycles = 10_000;
    let result = PolaritonQuantumMemoryBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 228 Phonon Universal Multi-Scale Visual Studio Autonomous Photonic-Phononic Quantum Memory Register & Non-Volatile Polariton Qubit Synthesizer Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Memory Storage Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_memory_storage_fidelity,
        result.min_memory_storage_fidelity,
        result.max_memory_storage_fidelity
    );
    println!(
        "Polariton Qubit Retention Fraction: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_polariton_qubit_retention_fraction,
        result.min_polariton_qubit_retention_fraction,
        result.max_polariton_qubit_retention_fraction
    );
    println!(
        "Topological Protection Gap (MHz): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_topological_protection_gap_mhz,
        result.min_topological_protection_gap_mhz,
        result.max_topological_protection_gap_mhz
    );
    println!(
        "Inter-Cell Crosstalk Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_inter_cell_crosstalk_isolation_db,
        result.min_inter_cell_crosstalk_isolation_db,
        result.max_inter_cell_crosstalk_isolation_db
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
        result.mean_memory_storage_fidelity >= 0.9980,
        "Mean memory storage fidelity must be >= 0.9980"
    );
    assert!(
        result.min_memory_storage_fidelity >= 0.9980,
        "Minimum memory storage fidelity must be >= 0.9980"
    );

    assert!(
        result.mean_polariton_qubit_retention_fraction >= 0.9970,
        "Mean polariton qubit retention fraction must be >= 0.9970"
    );
    assert!(
        result.min_polariton_qubit_retention_fraction >= 0.9970,
        "Minimum polariton qubit retention fraction must be >= 0.9970"
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
        result.mean_inter_cell_crosstalk_isolation_db >= 55.0,
        "Mean inter-cell crosstalk isolation must be >= 55.0 dB"
    );
    assert!(
        result.min_inter_cell_crosstalk_isolation_db >= 55.0,
        "Minimum inter-cell crosstalk isolation must be >= 55.0 dB"
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
