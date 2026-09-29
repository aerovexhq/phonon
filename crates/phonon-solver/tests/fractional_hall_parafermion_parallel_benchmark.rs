#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for fractional quantum Hall acoustic metamaterials and non-Abelian
//! parafermion interferometers across multi-threaded Rayon workers.

use phonon_solver::fractional_hall_parafermion::ParafermionBenchmarkRunner;

#[test]
fn test_10k_fractional_hall_parafermion_parallel_sweep() {
    let cycles = 10_000;
    let result = ParafermionBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 139 Fractional Quantum Hall Acoustic Metamaterials & Non-Abelian Parafermion Interferometers Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Braid Phase Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_braid_phase_fidelity,
        result.min_braid_phase_fidelity,
        result.max_braid_phase_fidelity
    );
    println!(
        "Fractional State Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_fractional_state_fidelity,
        result.min_fractional_state_fidelity,
        result.max_fractional_state_fidelity
    );
    println!(
        "Fractional Quantization Error: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_fractional_quantization_error,
        result.min_fractional_quantization_error,
        result.max_fractional_quantization_error
    );
    println!(
        "Topological Fractional Gap (MHz): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_topological_fractional_gap_mhz,
        result.min_topological_fractional_gap_mhz,
        result.max_topological_fractional_gap_mhz
    );
    println!(
        "Braiding Visibility: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_braiding_visibility,
        result.min_braiding_visibility,
        result.max_braiding_visibility
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
        result.min_braid_phase_fidelity >= 0.9970,
        "Minimum braid phase fidelity must be >= 0.9970"
    );
    assert!(
        result.min_fractional_state_fidelity >= 0.9950,
        "Minimum fractional state fidelity must be >= 0.9950"
    );
    assert!(
        result.max_fractional_quantization_error <= 0.0050,
        "Maximum fractional quantization error must be <= 0.0050"
    );
    assert!(
        result.min_topological_fractional_gap_mhz >= 15.0,
        "Minimum topological fractional gap must be >= 15.0 MHz"
    );
    assert!(
        result.min_braiding_visibility >= 0.9600,
        "Minimum braiding visibility must be >= 0.9600"
    );
}
