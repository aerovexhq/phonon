#![deny(unsafe_code)]

//! Large-scale parallel multi-physics validation benchmark for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Quantum
//! Metamaterial Frequency-Comb Beamformer & Hyperspectral Lidar Engine
//! across multi-core Rayon threads.

use phonon_solver::quantum_metamaterial_beamformer::QuantumMetamaterialBeamformerBenchmarkRunner;

#[test]
fn test_parallel_10k_quantum_metamaterial_beamformer_benchmark() {
    let cycles = 10_000;
    let result = QuantumMetamaterialBeamformerBenchmarkRunner::run_benchmark(cycles);

    println!(
        "Phase 263 Benchmark Completed: {} cycles in {:.6} s ({:.2} sweeps/sec)",
        result.total_cycles, result.elapsed_seconds, result.throughput_sweeps_per_sec
    );
    println!(
        "Beamformer Fidelity: mean={:.6}, min={:.6}, max={:.6}",
        result.mean_beamformer_fidelity,
        result.min_beamformer_fidelity,
        result.max_beamformer_fidelity
    );
    println!(
        "Frequency Comb State Retention Fraction: mean={:.6}, min={:.6}, max={:.6}",
        result.mean_frequency_comb_state_retention_fraction,
        result.min_frequency_comb_state_retention_fraction,
        result.max_frequency_comb_state_retention_fraction
    );
    println!(
        "Topological Protection Gap (MHz): mean={:.4}, min={:.4}, max={:.4}",
        result.mean_topological_protection_gap_mhz,
        result.min_topological_protection_gap_mhz,
        result.max_topological_protection_gap_mhz
    );
    println!(
        "Inter-Element Isolation (dB): mean={:.4}, min={:.4}, max={:.4}",
        result.mean_inter_element_crosstalk_isolation_db,
        result.min_inter_element_crosstalk_isolation_db,
        result.max_inter_element_crosstalk_isolation_db
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

    // Validate mean metric thresholds against roadmap design specifications
    assert!(
        result.mean_beamformer_fidelity >= 0.9980,
        "Mean beamformer fidelity must be >= 0.9980, got {:.6}",
        result.mean_beamformer_fidelity
    );
    assert!(
        result.min_beamformer_fidelity >= 0.9980,
        "Minimum beamformer fidelity must be >= 0.9980, got {:.6}",
        result.min_beamformer_fidelity
    );

    assert!(
        result.mean_frequency_comb_state_retention_fraction >= 0.9970,
        "Mean frequency comb state retention fraction must be >= 0.9970, got {:.6}",
        result.mean_frequency_comb_state_retention_fraction
    );
    assert!(
        result.min_frequency_comb_state_retention_fraction >= 0.9970,
        "Minimum frequency comb state retention fraction must be >= 0.9970, got {:.6}",
        result.min_frequency_comb_state_retention_fraction
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
        result.mean_inter_element_crosstalk_isolation_db >= 55.0,
        "Mean inter-element crosstalk isolation must be >= 55.0 dB, got {:.4}",
        result.mean_inter_element_crosstalk_isolation_db
    );
    assert!(
        result.min_inter_element_crosstalk_isolation_db >= 55.0,
        "Minimum inter-element crosstalk isolation must be >= 55.0 dB, got {:.4}",
        result.min_inter_element_crosstalk_isolation_db
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
