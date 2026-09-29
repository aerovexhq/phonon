//! Parallel Rayon benchmark test for quantum acoustoelectric moiré superlattices across 10,000 parameter sweeps.

use phonon_solver::acoustoelectric_moire::AcoustoelectricMoireBenchmarkRunner;

#[test]
fn test_acoustoelectric_moire_10k_benchmark() {
    let runner = AcoustoelectricMoireBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!("--- Quantum Acoustoelectric Moiré Superlattice Benchmark Report ---");
    println!("Total parameter sweeps: {}", report.total_cycles);
    println!("Elapsed time: {:.4} s", report.elapsed_seconds);
    println!(
        "Throughput: {:.2} sweeps/sec",
        report.throughput_cycles_per_sec
    );
    println!(
        "Mean Bandwidth Quenching Ratio: {:.2}x (min: {:.2}x, max: {:.2}x)",
        report.mean_quenching_ratio, report.min_quenching_ratio, report.max_quenching_ratio
    );
    println!(
        "Mean Electron-Phonon Pairing Ratio: {:.2}x (min: {:.2}x)",
        report.mean_pairing_ratio, report.min_pairing_ratio
    );
    println!(
        "Mean Correlation Ratio U/W: {:.2} (min: {:.2})",
        report.mean_correlation_ratio, report.min_correlation_ratio
    );
    println!(
        "Mean Moiré Minigap: {:.2} meV (min: {:.2} meV)",
        report.mean_minigap_mev, report.min_minigap_mev
    );
    println!(
        "Mean Simulation Fidelity: {:.2}% (min: {:.2}%)",
        report.mean_simulation_fidelity_pct, report.min_simulation_fidelity_pct
    );
    println!(
        "Mean Wigner Melting Temp: {:.2} K (min: {:.2} K)",
        report.mean_melting_temp_k, report.min_melting_temp_k
    );
    println!(
        "Compliance fraction: {:.4} ({}%)",
        report.compliance_fraction,
        report.compliance_fraction * 100.0
    );

    assert_eq!(
        report.total_cycles, 10_000,
        "Total sweeps must equal 10,000"
    );
    assert!(
        report.mean_quenching_ratio >= 10.0,
        "Mean quenching ratio must be >= 10.0x, got {}",
        report.mean_quenching_ratio
    );
    assert!(
        report.min_quenching_ratio >= 10.0,
        "Min quenching ratio must be >= 10.0x, got {}",
        report.min_quenching_ratio
    );
    assert!(
        report.mean_pairing_ratio >= 3.0,
        "Mean pairing ratio must be >= 3.0x, got {}",
        report.mean_pairing_ratio
    );
    assert!(
        report.min_pairing_ratio >= 3.0,
        "Min pairing ratio must be >= 3.0x, got {}",
        report.min_pairing_ratio
    );
    assert!(
        report.mean_correlation_ratio >= 3.0,
        "Mean correlation ratio must be >= 3.0, got {}",
        report.mean_correlation_ratio
    );
    assert!(
        report.min_correlation_ratio >= 3.0,
        "Min correlation ratio must be >= 3.0, got {}",
        report.min_correlation_ratio
    );
    assert!(
        report.mean_minigap_mev >= 5.0,
        "Mean minigap must be >= 5.0 meV, got {}",
        report.mean_minigap_mev
    );
    assert!(
        report.min_minigap_mev >= 5.0,
        "Min minigap must be >= 5.0 meV, got {}",
        report.min_minigap_mev
    );
    assert!(
        report.mean_simulation_fidelity_pct >= 98.0,
        "Mean simulation fidelity must be >= 98.0%, got {}",
        report.mean_simulation_fidelity_pct
    );
    assert!(
        report.min_simulation_fidelity_pct >= 98.0,
        "Min simulation fidelity must be >= 98.0%, got {}",
        report.min_simulation_fidelity_pct
    );
    assert!(
        report.mean_melting_temp_k >= 20.0,
        "Mean melting temp must be >= 20.0 K, got {}",
        report.mean_melting_temp_k
    );
    assert!(
        report.min_melting_temp_k >= 20.0,
        "Min melting temp must be >= 20.0 K, got {}",
        report.min_melting_temp_k
    );
    assert!(
        (report.compliance_fraction - 1.0).abs() < 1e-6,
        "Compliance fraction must be 100%, got {}",
        report.compliance_fraction
    );
}
