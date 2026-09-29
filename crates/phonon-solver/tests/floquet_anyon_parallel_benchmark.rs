//! Multi-threaded Rayon benchmark verification for Floquet anyon braiding across 10,000 parameter sweeps.

use phonon_solver::floquet_anyon_braiding::FloquetAnyonBenchmarkRunner;

#[test]
fn test_floquet_anyon_parallel_benchmark() {
    let runner = FloquetAnyonBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!("\n=== Floquet-Engineered Anyon Braiding 10,000 Parameter Sweep Benchmark ===");
    println!("Total sweeps:                 {}", report.total_cycles);
    println!(
        "Elapsed time:                 {:.4} s",
        report.elapsed_seconds
    );
    println!(
        "Throughput:                   {:.2} sweeps/s",
        report.throughput_cycles_per_sec
    );
    println!(
        "Mean Braiding Gate Fidelity:  {:.4}% (min: {:.4}%, max: {:.4}%)",
        report.mean_braiding_gate_fidelity_pct,
        report.min_braiding_gate_fidelity_pct,
        report.max_braiding_gate_fidelity_pct
    );
    println!(
        "Mean Leakage Error Rate:      {:.4e} (max: {:.4e})",
        report.mean_leakage_error_rate, report.max_leakage_error_rate
    );
    println!(
        "Mean Gauge Commutator Norm:   {:.3} (min: {:.3})",
        report.mean_synthetic_gauge_commutator_norm, report.min_synthetic_gauge_commutator_norm
    );
    println!(
        "Mean Floquet Bandgap:         {:.1} kHz",
        report.mean_floquet_gap_khz
    );
    println!(
        "Mean Logical Readout SNR:     {:.2} dB (min: {:.2} dB)",
        report.mean_logical_readout_snr_db, report.min_logical_readout_snr_db
    );
    println!(
        "Physical Compliance Fraction: {:.4} (100% required)",
        report.compliance_fraction
    );

    assert_eq!(report.total_cycles, 10_000);
    assert!(report.mean_braiding_gate_fidelity_pct >= 99.50);
    assert!(report.min_braiding_gate_fidelity_pct >= 99.50);
    assert!(report.max_leakage_error_rate <= 1.0e-4);
    assert!(report.mean_synthetic_gauge_commutator_norm >= 0.50);
    assert!(report.min_synthetic_gauge_commutator_norm >= 0.50);
    assert!(report.mean_floquet_gap_khz >= 50.0);
    assert!(report.mean_logical_readout_snr_db >= 25.0);
    assert!(report.min_logical_readout_snr_db >= 25.0);
    assert_eq!(
        report.compliance_fraction, 1.0,
        "All 10,000 parameter sweeps must satisfy physical bounds"
    );
}
