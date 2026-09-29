//! Multi-threaded Rayon benchmark verification for chiral acoustic Majorana braiding across 10,000 parameter sweeps.

use phonon_solver::majorana_chiral_phonon::MajoranaBraidingBenchmarkRunner;

#[test]
fn test_majorana_braiding_parallel_benchmark() {
    let runner = MajoranaBraidingBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!("\n=== Chiral Acoustic Majorana Braiding 10,000 Parameter Sweep Benchmark ===");
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
        "Mean Braiding Fidelity:       {:.3}% (min: {:.3}%, max: {:.3}%)",
        report.mean_braiding_fidelity_pct,
        report.min_braiding_fidelity_pct,
        report.max_braiding_fidelity_pct
    );
    println!(
        "Mean Non-Abelian Phase:       {:.4} rad (ideal: {:.4} rad)",
        report.mean_non_abelian_phase_rad,
        std::f64::consts::FRAC_PI_2
    );
    println!(
        "Max Phase Error:              {:.4} rad (threshold: <= 0.05 rad)",
        report.max_phase_error_rad
    );
    println!(
        "Mean Landau-Zener Leakage:    {:.4e} (max: {:.4e})",
        report.mean_landau_zener_leakage, report.max_landau_zener_leakage
    );
    println!(
        "Mean Parity Readout SNR:      {:.2} dB (min: {:.2} dB)",
        report.mean_parity_readout_snr_db, report.min_parity_readout_snr_db
    );
    println!(
        "Mean Dephasing Time:          {:.2} us",
        report.mean_dephasing_time_us
    );
    println!(
        "Physical Compliance Fraction: {:.4} (100% required)",
        report.compliance_fraction
    );

    assert_eq!(report.total_cycles, 10_000);
    assert!(report.mean_braiding_fidelity_pct >= 99.0);
    assert!(report.min_braiding_fidelity_pct >= 99.0);
    assert!(report.max_phase_error_rad <= 0.05);
    assert!(report.max_landau_zener_leakage <= 1.0e-3);
    assert!(report.mean_parity_readout_snr_db >= 20.0);
    assert!(report.min_parity_readout_snr_db >= 20.0);
    assert!(report.mean_dephasing_time_us >= 10.0);
    assert_eq!(
        report.compliance_fraction, 1.0,
        "All 10,000 parameter sweeps must satisfy physical bounds"
    );
}
