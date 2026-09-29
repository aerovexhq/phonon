#![deny(unsafe_code)]

use phonon_solver::high_harmonic_bloch::HighHarmonicBlochBenchmarkRunner;

#[test]
fn test_high_harmonic_bloch_parallel_benchmark() {
    let runner = HighHarmonicBlochBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!("\n=== Phase 98: High-Harmonic Acoustic Bloch & Frequency Synthesizer Benchmark ===");
    println!("Total cycles: {}", report.total_cycles);
    println!("Elapsed time: {:.4} s", report.elapsed_seconds);
    println!(
        "Throughput: {:.2} sweeps/s",
        report.throughput_cycles_per_sec
    );
    println!(
        "Mean Bloch frequency: {:.2} GHz (target >= 50.0 GHz)",
        report.mean_bloch_freq_ghz
    );
    println!("Min Bloch frequency: {:.2} GHz", report.min_bloch_freq_ghz);
    println!(
        "Mean cutoff order: {:.2} (target >= 25)",
        report.mean_cutoff_order
    );
    println!("Min cutoff order: {}", report.min_cutoff_order);
    println!(
        "Mean spectral purity: {:.2} dB (target >= 45.0 dB)",
        report.mean_spectral_purity_db
    );
    println!(
        "Min spectral purity: {:.2} dB",
        report.min_spectral_purity_db
    );
    println!(
        "Mean coherent oscillations: {:.2} (target >= 3.0)",
        report.mean_oscillations_count
    );
    println!(
        "Min coherent oscillations: {:.2}",
        report.min_oscillations_count
    );
    println!(
        "Mean synthesizer efficiency: {:.2}% (target >= 15.0%)",
        report.mean_efficiency_pct
    );
    println!(
        "Min synthesizer efficiency: {:.2}%",
        report.min_efficiency_pct
    );
    println!(
        "Mean Zener leakage: {:.3e} (target <= 0.05)",
        report.mean_zener_leakage
    );
    println!("Max Zener leakage: {:.3e}", report.max_zener_leakage);
    println!(
        "Physical compliance fraction: {:.4}",
        report.compliance_fraction
    );

    assert_eq!(report.total_cycles, 10_000);
    assert!(report.throughput_cycles_per_sec > 100_000.0);
    assert!(report.mean_bloch_freq_ghz >= 50.0);
    assert!(report.min_bloch_freq_ghz >= 50.0);
    assert!(report.mean_cutoff_order >= 25.0);
    assert!(report.min_cutoff_order >= 25);
    assert!(report.mean_spectral_purity_db >= 45.0);
    assert!(report.min_spectral_purity_db >= 45.0);
    assert!(report.mean_oscillations_count >= 3.0);
    assert!(report.min_oscillations_count >= 3.0);
    assert!(report.mean_efficiency_pct >= 15.0);
    assert!(report.min_efficiency_pct >= 15.0);
    assert!(report.mean_zener_leakage <= 0.05);
    assert!(report.max_zener_leakage <= 0.05);
    assert_eq!(report.compliance_fraction, 1.0);
}
