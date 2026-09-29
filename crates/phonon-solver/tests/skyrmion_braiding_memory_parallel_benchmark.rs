#![deny(unsafe_code)]

use phonon_solver::skyrmion_braiding_memory::SkyrmionMemoryBenchmarkRunner;

#[test]
fn test_skyrmion_braiding_memory_parallel_benchmark() {
    let runner = SkyrmionMemoryBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!("\n=== Phase 101: Chiral Phonon-Magnon Skyrmion Braiding & Memory Benchmark ===");
    println!("Total cycles: {}", report.total_cycles);
    println!("Elapsed time: {:.4} s", report.elapsed_seconds);
    println!(
        "Throughput: {:.2} sweeps/s",
        report.throughput_cycles_per_sec
    );
    println!(
        "Mean drift velocity: {:.2} m/s (target >= 250.0 m/s)",
        report.mean_drift_velocity
    );
    println!("Min drift velocity: {:.2} m/s", report.min_drift_velocity);
    println!(
        "Mean bit error rate: {:.3e} (target <= 1.0e-12)",
        report.mean_bit_error_rate
    );
    println!("Max bit error rate: {:.3e}", report.max_bit_error_rate);
    println!(
        "Mean retention time: {:.2} years (target >= 15.0 years)",
        report.mean_retention_years
    );
    println!(
        "Min retention time: {:.2} years",
        report.min_retention_years
    );
    println!(
        "Mean write energy: {:.3} fJ (target <= 0.5 fJ)",
        report.mean_write_energy_fj
    );
    println!("Max write energy: {:.3} fJ", report.max_write_energy_fj);
    println!(
        "Mean braiding fidelity: {:.4}% (target >= 99.5%)",
        report.mean_braiding_fidelity_pct
    );
    println!(
        "Min braiding fidelity: {:.4}%",
        report.min_braiding_fidelity_pct
    );
    println!(
        "Mean Hall suppression: {:.2}% (target >= 90.0%)",
        report.mean_hall_suppression_pct
    );
    println!(
        "Min Hall suppression: {:.2}%",
        report.min_hall_suppression_pct
    );
    println!(
        "Physical compliance fraction: {:.4}",
        report.compliance_fraction
    );

    assert_eq!(report.total_cycles, 10_000);
    assert!(report.throughput_cycles_per_sec > 100_000.0);
    assert!(report.mean_drift_velocity >= 250.0);
    assert!(report.min_drift_velocity >= 250.0);
    assert!(report.mean_bit_error_rate <= 1.0e-12);
    assert!(report.max_bit_error_rate <= 1.0e-12);
    assert!(report.mean_retention_years >= 15.0);
    assert!(report.min_retention_years >= 15.0);
    assert!(report.mean_write_energy_fj <= 0.5);
    assert!(report.max_write_energy_fj <= 0.5);
    assert!(report.mean_braiding_fidelity_pct >= 99.5);
    assert!(report.min_braiding_fidelity_pct >= 99.5);
    assert!(report.mean_hall_suppression_pct >= 90.0);
    assert!(report.min_hall_suppression_pct >= 90.0);
    assert_eq!(report.compliance_fraction, 1.0);
}
