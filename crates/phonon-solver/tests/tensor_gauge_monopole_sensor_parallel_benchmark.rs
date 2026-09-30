#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for topological acoustic higher-rank tensor gauge fields and chiral
//! monopole-plaquette phononic sensors across multi-threaded Rayon workers.

use phonon_solver::tensor_gauge_monopole_sensor::TensorGaugeBenchmarkRunner;

#[test]
fn test_10k_tensor_gauge_monopole_sensor_parallel_sweep() {
    let cycles = 10_000;
    let result = TensorGaugeBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 159 Topological Acoustic Higher-Rank Tensor Gauge Fields & Chiral Monopole-Plaquette Phononic Sensors Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Tensor Charge Sensitivity Enhancement: mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_tensor_charge_sensitivity_enhancement,
        result.min_tensor_charge_sensitivity_enhancement,
        result.max_tensor_charge_sensitivity_enhancement
    );
    println!(
        "Plaquette Phase Stability Error (rad): mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_plaquette_phase_stability_error_rad,
        result.min_plaquette_phase_stability_error_rad,
        result.max_plaquette_phase_stability_error_rad
    );
    println!(
        "Sub-Dimensional Leakage: mean = {:.4e}, min = {:.4e}, max = {:.4e}",
        result.mean_sub_dimensional_leakage,
        result.min_sub_dimensional_leakage,
        result.max_sub_dimensional_leakage
    );
    println!(
        "Topological Monopole Lifetime (ms): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_topological_monopole_lifetime_ms,
        result.min_topological_monopole_lifetime_ms,
        result.max_topological_monopole_lifetime_ms
    );
    println!(
        "Tensor Gauge Flux Quantization Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_tensor_gauge_flux_quantization_fidelity,
        result.min_tensor_gauge_flux_quantization_fidelity,
        result.max_tensor_gauge_flux_quantization_fidelity
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
        result.min_tensor_charge_sensitivity_enhancement >= 75.0,
        "Minimum tensor charge sensitivity enhancement must be >= 75.0, got {:.4}",
        result.min_tensor_charge_sensitivity_enhancement
    );
    assert!(
        result.max_plaquette_phase_stability_error_rad <= 0.0015,
        "Maximum plaquette phase stability error must be <= 0.0015 rad, got {:.6}",
        result.max_plaquette_phase_stability_error_rad
    );
    assert!(
        result.max_sub_dimensional_leakage <= 1.0e-5,
        "Maximum sub-dimensional leakage must be <= 1.0e-5, got {:.4e}",
        result.max_sub_dimensional_leakage
    );
    assert!(
        result.min_topological_monopole_lifetime_ms >= 25.0,
        "Minimum topological monopole lifetime must be >= 25.0 ms, got {:.4}",
        result.min_topological_monopole_lifetime_ms
    );
    assert!(
        result.min_tensor_gauge_flux_quantization_fidelity >= 0.9970,
        "Minimum tensor gauge flux quantization fidelity must be >= 0.9970, got {:.6}",
        result.min_tensor_gauge_flux_quantization_fidelity
    );
}
