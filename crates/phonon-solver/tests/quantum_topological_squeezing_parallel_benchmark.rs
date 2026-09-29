//! Parallel Rayon benchmark test for quantum topological phonon squeezing across 10,000 parameter sweeps.

use phonon_solver::quantum_topological_squeezing::QuantumPhononSqueezingBenchmarkRunner;

#[test]
fn test_quantum_topological_squeezing_10k_benchmark() {
    let runner = QuantumPhononSqueezingBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!("--- Quantum Topological Phonon Squeezing Benchmark Report ---");
    println!("Total parameter sweeps: {}", report.total_cycles);
    println!("Elapsed time: {:.4} s", report.elapsed_seconds);
    println!(
        "Throughput: {:.2} sweeps/sec",
        report.throughput_cycles_per_sec
    );
    println!(
        "Mean Quadrature Squeezing: {:.2} dB (min: {:.2} dB, max: {:.2} dB)",
        report.mean_squeezing_db, report.min_squeezing_db, report.max_squeezing_db
    );
    println!(
        "Mean Schrödinger Cat Fidelity: {:.2}% (min: {:.2}%)",
        report.mean_fidelity_pct, report.min_fidelity_pct
    );
    println!(
        "Mean Wigner Negativity: {:.4} (min: {:.4})",
        report.mean_wigner_negativity, report.min_wigner_negativity
    );
    println!(
        "Mean Detectable Force Sensitivity: {:.2} aN/sqrt(Hz) (max: {:.2} aN/sqrt(Hz))",
        report.mean_force_sensitivity, report.max_force_sensitivity
    );
    println!(
        "Mean Quantum Gravimetry Precision: {:.2} nano-g (max: {:.2} nano-g)",
        report.mean_gravimetry_nano_g, report.max_gravimetry_nano_g
    );
    println!(
        "Mean Continuous Entanglement: {:.2} ebits",
        report.mean_entanglement_ebits
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
        report.mean_squeezing_db >= 6.0,
        "Mean squeezing must be >= 6.0 dB, got {}",
        report.mean_squeezing_db
    );
    assert!(
        report.min_squeezing_db >= 6.0,
        "Min squeezing must be >= 6.0 dB, got {}",
        report.min_squeezing_db
    );
    assert!(
        report.mean_fidelity_pct >= 90.0,
        "Mean cat state fidelity must be >= 90.0%, got {}",
        report.mean_fidelity_pct
    );
    assert!(
        report.min_fidelity_pct >= 90.0,
        "Min cat state fidelity must be >= 90.0%, got {}",
        report.min_fidelity_pct
    );
    assert!(
        report.mean_wigner_negativity >= 0.15,
        "Mean Wigner negativity must be >= 0.15, got {}",
        report.mean_wigner_negativity
    );
    assert!(
        report.min_wigner_negativity >= 0.15,
        "Min Wigner negativity must be >= 0.15, got {}",
        report.min_wigner_negativity
    );
    assert!(
        report.mean_force_sensitivity <= 25.0,
        "Mean force sensitivity must be <= 25.0 aN/sqrt(Hz), got {}",
        report.mean_force_sensitivity
    );
    assert!(
        report.max_force_sensitivity <= 25.0,
        "Max force sensitivity must be <= 25.0 aN/sqrt(Hz), got {}",
        report.max_force_sensitivity
    );
    assert!(
        report.mean_gravimetry_nano_g <= 5.0,
        "Mean gravimetry precision must be <= 5.0 nano-g, got {}",
        report.mean_gravimetry_nano_g
    );
    assert!(
        report.max_gravimetry_nano_g <= 5.0,
        "Max gravimetry precision must be <= 5.0 nano-g, got {}",
        report.max_gravimetry_nano_g
    );
    assert!(
        report.mean_entanglement_ebits >= 1.0,
        "Mean continuous entanglement must be >= 1.0 ebits, got {}",
        report.mean_entanglement_ebits
    );
    assert!(
        (report.compliance_fraction - 1.0).abs() < 1e-6,
        "Compliance fraction must be 100%, got {}",
        report.compliance_fraction
    );
}
