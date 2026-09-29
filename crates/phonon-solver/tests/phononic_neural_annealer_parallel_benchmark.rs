#![deny(unsafe_code)]

use phonon_solver::phononic_neural_annealer::PhononicAnnealerBenchmarkRunner;

#[test]
fn test_phononic_neural_annealer_parallel_benchmark() {
    let runner = PhononicAnnealerBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!("\n=== Phase 100: Quantum Phononic Neural Annealer Benchmark ===");
    println!("Total cycles: {}", report.total_cycles);
    println!("Elapsed time: {:.4} s", report.elapsed_seconds);
    println!(
        "Throughput: {:.2} sweeps/s",
        report.throughput_cycles_per_sec
    );
    println!(
        "Mean convergence fidelity: {:.2}% (target >= 98.0%)",
        report.mean_fidelity_pct
    );
    println!("Min convergence fidelity: {:.2}%", report.min_fidelity_pct);
    println!(
        "Mean computational speedup: {:.2}x (target >= 100.0x)",
        report.mean_speedup
    );
    println!("Min computational speedup: {:.2}x", report.min_speedup);
    println!(
        "Mean energy per flip: {:.2} fJ (target <= 50.0 fJ)",
        report.mean_energy_fj
    );
    println!("Max energy per flip: {:.2} fJ", report.max_energy_fj);
    println!(
        "Mean graph approx ratio: {:.4} (target >= 0.95)",
        report.mean_approx_ratio
    );
    println!("Min graph approx ratio: {:.4}", report.min_approx_ratio);
    println!(
        "Mean solution time: {:.3} us (target <= 10.0 us)",
        report.mean_solution_us
    );
    println!("Max solution time: {:.3} us", report.max_solution_us);
    println!(
        "Mean bifurcation contrast: {:.2} dB (target >= 25.0 dB)",
        report.mean_contrast_db
    );
    println!("Min bifurcation contrast: {:.2} dB", report.min_contrast_db);
    println!(
        "Physical compliance fraction: {:.4}",
        report.compliance_fraction
    );

    assert_eq!(report.total_cycles, 10_000);
    assert!(report.throughput_cycles_per_sec > 100_000.0);
    assert!(report.mean_fidelity_pct >= 98.0);
    assert!(report.min_fidelity_pct >= 98.0);
    assert!(report.mean_speedup >= 100.0);
    assert!(report.min_speedup >= 100.0);
    assert!(report.mean_energy_fj <= 50.0);
    assert!(report.max_energy_fj <= 50.0);
    assert!(report.mean_approx_ratio >= 0.95);
    assert!(report.min_approx_ratio >= 0.95);
    assert!(report.mean_solution_us <= 10.0);
    assert!(report.max_solution_us <= 10.0);
    assert!(report.mean_contrast_db >= 25.0);
    assert!(report.min_contrast_db >= 25.0);
    assert_eq!(report.compliance_fraction, 1.0);
}
