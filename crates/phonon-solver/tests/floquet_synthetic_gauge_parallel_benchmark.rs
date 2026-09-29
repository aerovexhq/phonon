#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for Floquet-Bloch synthetic gauge acoustic fields and dynamically
//! reconfigurable phononic quantum simulators across multi-threaded Rayon workers.

use phonon_solver::floquet_synthetic_gauge::GaugeBenchmarkRunner;

#[test]
fn test_10k_floquet_synthetic_gauge_parallel_sweep() {
    let cycles = 10_000;
    let result = GaugeBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 137 Floquet-Bloch Synthetic Gauge Fields & Reconfigurable Simulators Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Dynamical State Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_dynamical_state_fidelity,
        result.min_dynamical_state_fidelity,
        result.max_dynamical_state_fidelity
    );
    println!(
        "Synthetic Magnetic Flux Ratio: mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_synthetic_magnetic_flux_ratio,
        result.min_synthetic_magnetic_flux_ratio,
        result.max_synthetic_magnetic_flux_ratio
    );
    println!(
        "Flux Quantization Error: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_flux_quantization_error,
        result.min_flux_quantization_error,
        result.max_flux_quantization_error
    );
    println!(
        "Chern Switching Time (ns): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_chern_switching_time_ns,
        result.min_chern_switching_time_ns,
        result.max_chern_switching_time_ns
    );
    println!(
        "Topological Band Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_topological_band_isolation_db,
        result.min_topological_band_isolation_db,
        result.max_topological_band_isolation_db
    );

    assert_eq!(
        result.total_cycles, cycles,
        "Total benchmark cycles must equal requested 10,000"
    );

    // Verify 100% physical compliance across all parameter sweep variations
    assert!(
        (result.physical_compliance_fraction - 1.0).abs() < 1.0e-6,
        "Physical compliance fraction must be 100.0%, got {:.2}%",
        result.physical_compliance_fraction * 100.0
    );

    // Assert dynamical state fidelity >= 0.9950
    assert!(
        result.mean_dynamical_state_fidelity >= 0.9950,
        "Mean state fidelity must be >= 0.9950, got {:.6}",
        result.mean_dynamical_state_fidelity
    );
    assert!(
        result.min_dynamical_state_fidelity >= 0.9950,
        "Min state fidelity must be >= 0.9950, got {:.6}",
        result.min_dynamical_state_fidelity
    );

    // Assert synthetic magnetic flux ratio >= 0.500
    assert!(
        result.mean_synthetic_magnetic_flux_ratio >= 0.500,
        "Mean flux ratio must be >= 0.500, got {:.4}",
        result.mean_synthetic_magnetic_flux_ratio
    );
    assert!(
        result.min_synthetic_magnetic_flux_ratio >= 0.500,
        "Min flux ratio must be >= 0.500, got {:.4}",
        result.min_synthetic_magnetic_flux_ratio
    );

    // Assert flux quantization error <= 0.010
    assert!(
        result.mean_flux_quantization_error <= 0.010,
        "Mean flux quantization error must be <= 0.010, got {:.6}",
        result.mean_flux_quantization_error
    );
    assert!(
        result.max_flux_quantization_error <= 0.010,
        "Max flux quantization error must be <= 0.010, got {:.6}",
        result.max_flux_quantization_error
    );

    // Assert Chern switching time <= 20.0 ns
    assert!(
        result.mean_chern_switching_time_ns <= 20.0,
        "Mean Chern switching time must be <= 20.0 ns, got {:.4}",
        result.mean_chern_switching_time_ns
    );
    assert!(
        result.max_chern_switching_time_ns <= 20.0,
        "Max Chern switching time must be <= 20.0 ns, got {:.4}",
        result.max_chern_switching_time_ns
    );

    // Assert topological band isolation >= 30.0 dB
    assert!(
        result.mean_topological_band_isolation_db >= 30.0,
        "Mean band isolation must be >= 30.0 dB, got {:.4}",
        result.mean_topological_band_isolation_db
    );
    assert!(
        result.min_topological_band_isolation_db >= 30.0,
        "Min band isolation must be >= 30.0 dB, got {:.4}",
        result.min_topological_band_isolation_db
    );
}
