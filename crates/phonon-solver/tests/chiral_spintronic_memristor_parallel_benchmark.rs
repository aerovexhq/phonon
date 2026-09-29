#![deny(unsafe_code)]

use phonon_solver::chiral_spintronic_memristor::SpintronicMemristorBenchmarkRunner;

#[test]
fn test_chiral_spintronic_memristor_parallel_benchmark() {
    let runner = SpintronicMemristorBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!("\n=== Phase 97: Chiral Spintronic Memristor & Crossbar Benchmark ===");
    println!("Total cycles: {}", report.total_cycles);
    println!("Elapsed time: {:.4} s", report.elapsed_seconds);
    println!(
        "Throughput: {:.2} sweeps/s",
        report.throughput_cycles_per_sec
    );
    println!(
        "Mean programming energy: {:.3} fJ (target <= 10.0 fJ)",
        report.mean_energy_fj
    );
    println!("Max programming energy: {:.3} fJ", report.max_energy_fj);
    println!(
        "Mean retention time: {:.2} years (target >= 10.0 years)",
        report.mean_retention_years
    );
    println!(
        "Min retention time: {:.2} years",
        report.min_retention_years
    );
    println!(
        "Mean on/off ratio: {:.2} (target >= 10.0)",
        report.mean_on_off_ratio
    );
    println!("Min on/off ratio: {:.2}", report.min_on_off_ratio);
    println!(
        "Mean STDP fidelity: {:.2}% (target >= 95.0%)",
        report.mean_stdp_fidelity_pct
    );
    println!("Min STDP fidelity: {:.2}%", report.min_stdp_fidelity_pct);
    println!(
        "Mean crossbar efficiency: {:.2} TOPS/W (target >= 150.0 TOPS/W)",
        report.mean_efficiency_topsw
    );
    println!(
        "Min crossbar efficiency: {:.2} TOPS/W",
        report.min_efficiency_topsw
    );
    println!(
        "Mean linearity error: {:.3}% (target <= 2.5%)",
        report.mean_linearity_error_pct
    );
    println!(
        "Max linearity error: {:.3}%",
        report.max_linearity_error_pct
    );
    println!(
        "Physical compliance fraction: {:.4}",
        report.compliance_fraction
    );

    assert_eq!(report.total_cycles, 10_000);
    assert!(report.throughput_cycles_per_sec > 100_000.0);
    assert!(report.mean_energy_fj <= 10.0);
    assert!(report.max_energy_fj <= 10.0);
    assert!(report.mean_retention_years >= 10.0);
    assert!(report.min_retention_years >= 10.0);
    assert!(report.mean_on_off_ratio >= 10.0);
    assert!(report.min_on_off_ratio >= 10.0);
    assert!(report.mean_stdp_fidelity_pct >= 95.0);
    assert!(report.min_stdp_fidelity_pct >= 95.0);
    assert!(report.mean_efficiency_topsw >= 150.0);
    assert!(report.min_efficiency_topsw >= 150.0);
    assert!(report.mean_linearity_error_pct <= 2.5);
    assert!(report.max_linearity_error_pct <= 2.5);
    assert_eq!(report.compliance_fraction, 1.0);
}
