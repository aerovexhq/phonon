#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for quantum acoustic non-Hermitian Floquet exceptional-ring synthesizers
//! and chiral skin sensors across multi-threaded Rayon workers.

use phonon_solver::floquet_exceptional_ring_sensor::ExceptionalRingBenchmarkRunner;

#[test]
fn test_10k_floquet_exceptional_ring_sensor_parallel_sweep() {
    let cycles = 10_000;
    let result = ExceptionalRingBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 154 Quantum Acoustic Non-Hermitian Floquet Exceptional-Ring Synthesizers & Chiral Skin Sensors Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Skin Mode Localization Ratio: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_skin_mode_localization_ratio,
        result.min_skin_mode_localization_ratio,
        result.max_skin_mode_localization_ratio
    );
    println!(
        "Sensitivity Enhancement Factor: mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_sensitivity_enhancement_factor,
        result.min_sensitivity_enhancement_factor,
        result.max_sensitivity_enhancement_factor
    );
    println!(
        "Reverse Backscattering Suppression (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_reverse_backscattering_suppression_db,
        result.min_reverse_backscattering_suppression_db,
        result.max_reverse_backscattering_suppression_db
    );
    println!(
        "Sensor Noise Figure (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_sensor_noise_figure_db,
        result.min_sensor_noise_figure_db,
        result.max_sensor_noise_figure_db
    );
    println!(
        "Exceptional Ring Topological Charge: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_exceptional_ring_topological_charge,
        result.min_exceptional_ring_topological_charge,
        result.max_exceptional_ring_topological_charge
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
        result.min_skin_mode_localization_ratio >= 0.940,
        "Minimum skin mode localization ratio must be >= 0.940, got {:.6}",
        result.min_skin_mode_localization_ratio
    );
    assert!(
        result.min_sensitivity_enhancement_factor >= 85.0,
        "Minimum sensitivity enhancement factor must be >= 85.0, got {:.4}",
        result.min_sensitivity_enhancement_factor
    );
    assert!(
        result.min_reverse_backscattering_suppression_db >= 52.0,
        "Minimum reverse backscattering suppression must be >= 52.0 dB, got {:.4} dB",
        result.min_reverse_backscattering_suppression_db
    );
    assert!(
        result.max_sensor_noise_figure_db <= 0.45,
        "Maximum sensor noise figure must be <= 0.45 dB, got {:.4} dB",
        result.max_sensor_noise_figure_db
    );
    assert!(
        result.min_exceptional_ring_topological_charge >= 0.990,
        "Minimum exceptional ring topological charge must be >= 0.990, got {:.6}",
        result.min_exceptional_ring_topological_charge
    );
}
