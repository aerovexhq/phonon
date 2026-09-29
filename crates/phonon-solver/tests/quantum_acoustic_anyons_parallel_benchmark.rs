//! Parallel Rayon benchmark test for quantum acoustic anyon braiding networks across 10,000 parameter sweeps.

use phonon_solver::quantum_acoustic_anyons::QuantumAcousticAnyonBenchmarkRunner;

#[test]
fn test_quantum_acoustic_anyons_10k_benchmark() {
    let runner = QuantumAcousticAnyonBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!("--- Quantum Acoustic Anyon Braiding Benchmark Report ---");
    println!("Total parameter sweeps: {}", report.total_cycles);
    println!("Elapsed time: {:.4} s", report.elapsed_seconds);
    println!(
        "Throughput: {:.2} sweeps/sec",
        report.throughput_cycles_per_sec
    );
    println!(
        "Mean Braiding Gate Fidelity: {:.4}% (min: {:.4}%)",
        report.mean_fidelity_pct, report.min_fidelity_pct
    );
    println!(
        "Mean Non-Adiabatic Leakage: {:.3e} (max: {:.3e})",
        report.mean_leakage, report.max_leakage
    );
    println!(
        "Mean Effective Topological Minigap: {:.2} MHz (min: {:.2} MHz)",
        report.mean_gap_mhz, report.min_gap_mhz
    );
    println!(
        "Mean Braiding Phase Error: {:.5} rad (max: {:.5} rad)",
        report.mean_phase_error_rad, report.max_phase_error_rad
    );
    println!(
        "Mean Parity Readout SNR: {:.2} dB (min: {:.2} dB)",
        report.mean_readout_snr_db, report.min_readout_snr_db
    );
    println!(
        "Mean Coherence Time T2*: {:.2} us (min: {:.2} us)",
        report.mean_coherence_time_us, report.min_coherence_time_us
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
        report.mean_fidelity_pct >= 99.90,
        "Mean fidelity must be >= 99.90%, got {}",
        report.mean_fidelity_pct
    );
    assert!(
        report.min_fidelity_pct >= 99.90,
        "Min fidelity must be >= 99.90%, got {}",
        report.min_fidelity_pct
    );
    assert!(
        report.mean_leakage <= 1.0e-5,
        "Mean leakage must be <= 1.0e-5, got {}",
        report.mean_leakage
    );
    assert!(
        report.max_leakage <= 1.0e-5,
        "Max leakage must be <= 1.0e-5, got {}",
        report.max_leakage
    );
    assert!(
        report.mean_gap_mhz >= 15.0,
        "Mean gap must be >= 15.0 MHz, got {}",
        report.mean_gap_mhz
    );
    assert!(
        report.min_gap_mhz >= 15.0,
        "Min gap must be >= 15.0 MHz, got {}",
        report.min_gap_mhz
    );
    assert!(
        report.mean_phase_error_rad <= 0.005,
        "Mean phase error must be <= 0.005 rad, got {}",
        report.mean_phase_error_rad
    );
    assert!(
        report.max_phase_error_rad <= 0.005,
        "Max phase error must be <= 0.005 rad, got {}",
        report.max_phase_error_rad
    );
    assert!(
        report.mean_readout_snr_db >= 30.0,
        "Mean readout SNR must be >= 30.0 dB, got {}",
        report.mean_readout_snr_db
    );
    assert!(
        report.min_readout_snr_db >= 30.0,
        "Min readout SNR must be >= 30.0 dB, got {}",
        report.min_readout_snr_db
    );
    assert!(
        report.mean_coherence_time_us >= 50.0,
        "Mean coherence time must be >= 50.0 us, got {}",
        report.mean_coherence_time_us
    );
    assert!(
        report.min_coherence_time_us >= 50.0,
        "Min coherence time must be >= 50.0 us, got {}",
        report.min_coherence_time_us
    );
    assert!(
        (report.compliance_fraction - 1.0).abs() < 1e-6,
        "Compliance fraction must be 100%, got {}",
        report.compliance_fraction
    );
}
