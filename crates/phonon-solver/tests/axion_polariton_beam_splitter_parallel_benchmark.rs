#![deny(unsafe_code)]

//! Large-scale parallel multi-physics validation benchmark for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Topological
//! Axion-Polariton Waveguide & Quantum Hall Beam Splitter Engine across multi-core Rayon threads.

use phonon_solver::axion_polariton_beam_splitter::AxionPolaritonBeamSplitterBenchmarkRunner;

#[test]
fn test_parallel_10k_axion_polariton_beam_splitter_benchmark() {
    let cycles = 10_000;
    let result = AxionPolaritonBeamSplitterBenchmarkRunner::run_benchmark(cycles);

    println!(
        "Phase 264 Benchmark Completed: {} cycles in {:.6} s ({:.2} sweeps/sec)",
        result.total_cycles, result.elapsed_seconds, result.throughput_sweeps_per_sec
    );
    println!(
        "Beam Splitter Fidelity: mean={:.6}, min={:.6}, max={:.6}",
        result.mean_beam_splitter_fidelity,
        result.min_beam_splitter_fidelity,
        result.max_beam_splitter_fidelity
    );
    println!(
        "Quantum Hall State Retention Fraction: mean={:.6}, min={:.6}, max={:.6}",
        result.mean_quantum_hall_state_retention_fraction,
        result.min_quantum_hall_state_retention_fraction,
        result.max_quantum_hall_state_retention_fraction
    );
    println!(
        "Topological Protection Gap (MHz): mean={:.4}, min={:.4}, max={:.4}",
        result.mean_topological_protection_gap_mhz,
        result.min_topological_protection_gap_mhz,
        result.max_topological_protection_gap_mhz
    );
    println!(
        "Inter-Port Crosstalk Isolation (dB): mean={:.4}, min={:.4}, max={:.4}",
        result.mean_inter_port_crosstalk_isolation_db,
        result.min_inter_port_crosstalk_isolation_db,
        result.max_inter_port_crosstalk_isolation_db
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
        "All 10,000 parameter sweeps must satisfy physical compliance criteria"
    );

    // Validate mean and extreme metric thresholds against roadmap design specifications
    assert!(
        result.mean_beam_splitter_fidelity >= 0.9980,
        "Mean beam splitter fidelity must be >= 0.9980, got {:.6}",
        result.mean_beam_splitter_fidelity
    );
    assert!(
        result.min_beam_splitter_fidelity >= 0.9980,
        "Minimum beam splitter fidelity must be >= 0.9980, got {:.6}",
        result.min_beam_splitter_fidelity
    );

    assert!(
        result.mean_quantum_hall_state_retention_fraction >= 0.9970,
        "Mean quantum Hall state retention fraction must be >= 0.9970, got {:.6}",
        result.mean_quantum_hall_state_retention_fraction
    );
    assert!(
        result.min_quantum_hall_state_retention_fraction >= 0.9970,
        "Minimum quantum Hall state retention fraction must be >= 0.9970, got {:.6}",
        result.min_quantum_hall_state_retention_fraction
    );

    assert!(
        result.mean_topological_protection_gap_mhz >= 45.0,
        "Mean topological protection gap must be >= 45.0 MHz, got {:.4}",
        result.mean_topological_protection_gap_mhz
    );
    assert!(
        result.min_topological_protection_gap_mhz >= 45.0,
        "Minimum topological protection gap must be >= 45.0 MHz, got {:.4}",
        result.min_topological_protection_gap_mhz
    );

    assert!(
        result.mean_inter_port_crosstalk_isolation_db >= 55.0,
        "Mean inter-port crosstalk isolation must be >= 55.0 dB, got {:.4}",
        result.mean_inter_port_crosstalk_isolation_db
    );
    assert!(
        result.min_inter_port_crosstalk_isolation_db >= 55.0,
        "Minimum inter-port crosstalk isolation must be >= 55.0 dB, got {:.4}",
        result.min_inter_port_crosstalk_isolation_db
    );

    assert!(
        result.mean_topological_mode_dephasing_rate_hz <= 12.0,
        "Mean topological mode dephasing rate must be <= 12.0 Hz, got {:.4}",
        result.mean_topological_mode_dephasing_rate_hz
    );
    assert!(
        result.max_topological_mode_dephasing_rate_hz <= 12.0,
        "Maximum topological mode dephasing rate must be <= 12.0 Hz, got {:.4}",
        result.max_topological_mode_dephasing_rate_hz
    );
}
