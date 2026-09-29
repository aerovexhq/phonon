#![deny(unsafe_code)]

use phonon_solver::topological_acoustic_axion::AcousticAxionBenchmarkRunner;

#[test]
fn test_topological_acoustic_axion_parallel_benchmark() {
    let runner = AcousticAxionBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!("\n=== Phase 99: Topological Acoustic Axion Polariton Benchmark ===");
    println!("Total cycles: {}", report.total_cycles);
    println!("Elapsed time: {:.4} s", report.elapsed_seconds);
    println!(
        "Throughput: {:.2} sweeps/s",
        report.throughput_cycles_per_sec
    );
    println!(
        "Mean magnetoelectric isolation: {:.2} dB (target >= 30.0 dB)",
        report.mean_isolation_db
    );
    println!(
        "Min magnetoelectric isolation: {:.2} dB",
        report.min_isolation_db
    );
    println!(
        "Mean axion cooperativity: {:.2} (target >= 50.0)",
        report.mean_cooperativity
    );
    println!("Min axion cooperativity: {:.2}", report.min_cooperativity);
    println!(
        "Mean dark matter SNR: {:.2} dB (target >= 25.0 dB)",
        report.mean_snr_db
    );
    println!("Min dark matter SNR: {:.2} dB", report.min_snr_db);
    println!(
        "Mean chiral anomaly purity: {:.2}% (target >= 95.0%)",
        report.mean_purity_pct
    );
    println!("Min chiral anomaly purity: {:.2}%", report.min_purity_pct);
    println!(
        "Mean insertion loss: {:.3} dB (target <= 1.0 dB)",
        report.mean_insertion_loss_db
    );
    println!("Max insertion loss: {:.3} dB", report.max_insertion_loss_db);
    println!(
        "Mean anti-crossing gap: {:.2} GHz (target >= 1.5 GHz)",
        report.mean_gap_ghz
    );
    println!("Min anti-crossing gap: {:.2} GHz", report.min_gap_ghz);
    println!(
        "Physical compliance fraction: {:.4}",
        report.compliance_fraction
    );

    assert_eq!(report.total_cycles, 10_000);
    assert!(report.throughput_cycles_per_sec > 100_000.0);
    assert!(report.mean_isolation_db >= 30.0);
    assert!(report.min_isolation_db >= 30.0);
    assert!(report.mean_cooperativity >= 50.0);
    assert!(report.min_cooperativity >= 50.0);
    assert!(report.mean_snr_db >= 25.0);
    assert!(report.min_snr_db >= 25.0);
    assert!(report.mean_purity_pct >= 95.0);
    assert!(report.min_purity_pct >= 95.0);
    assert!(report.mean_insertion_loss_db <= 1.0);
    assert!(report.max_insertion_loss_db <= 1.0);
    assert!(report.mean_gap_ghz >= 1.5);
    assert!(report.min_gap_ghz >= 1.5);
    assert_eq!(report.compliance_fraction, 1.0);
}
