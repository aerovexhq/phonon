//! Parallel Rayon benchmark verifying cavity quantum magnomechanics,
//! continuous-variable logarithmic negativity (E_N > 0), quantum state fidelity (>= 90%),
//! transduction efficiency (>= 50%), and ground state cooling (n_eff < 1.0) across 10,000 parameter sweeps.

use phonon_solver::cavity_magnomechanics::CavityMagnomechanicsBenchmarkRunner;

#[test]
fn test_cavity_magnomechanics_parallel_benchmark_10k_sweeps() {
    let runner = CavityMagnomechanicsBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!("==================================================================");
    println!("PHASE 79: CAVITY QUANTUM MAGNOMECHANICS & ENTANGLED PHONON STATES");
    println!("==================================================================");
    println!("Total Parameter Sweeps:          {}", report.total_cycles);
    println!(
        "Elapsed Time:                    {:.4} s",
        report.elapsed_seconds
    );
    println!(
        "Throughput:                      {:.2} sweeps/sec",
        report.throughput_cycles_per_sec
    );
    println!(
        "Mean Logarithmic Negativity:     {:.4} (E_N > 0.0 required)",
        report.mean_log_negativity
    );
    println!(
        "Min Logarithmic Negativity:      {:.4}",
        report.min_log_negativity
    );
    println!(
        "Mean Quantum State Fidelity:     {:.2}% (>= 90.0% required)",
        report.mean_quantum_fidelity * 100.0
    );
    println!(
        "Min Quantum State Fidelity:      {:.2}%",
        report.min_quantum_fidelity * 100.0
    );
    println!(
        "Mean Transduction Efficiency:    {:.2}% (>= 50.0% required)",
        report.mean_transduction_efficiency * 100.0
    );
    println!(
        "Min Transduction Efficiency:     {:.2}%",
        report.min_transduction_efficiency * 100.0
    );
    println!(
        "Mean Effective Phonon Number:    {:.5} (n_eff < 1.0 required)",
        report.mean_phonon_occupation
    );
    println!(
        "Max Effective Phonon Number:     {:.5}",
        report.max_phonon_occupation
    );
    println!(
        "Compliance Fraction:             {:.4} (100% required)",
        report.compliance_fraction
    );
    println!("==================================================================");

    assert_eq!(report.total_cycles, 10_000);
    assert!(
        report.mean_log_negativity > 0.0,
        "Mean log negativity {:.4} must be > 0.0",
        report.mean_log_negativity
    );
    assert!(
        report.min_log_negativity > 0.0,
        "Min log negativity {:.4} must be > 0.0",
        report.min_log_negativity
    );
    assert!(
        report.mean_quantum_fidelity >= 0.90,
        "Mean quantum state fidelity {:.4} must be >= 0.90",
        report.mean_quantum_fidelity
    );
    assert!(
        report.min_quantum_fidelity >= 0.90,
        "Min quantum state fidelity {:.4} must be >= 0.90",
        report.min_quantum_fidelity
    );
    assert!(
        report.mean_transduction_efficiency >= 0.50,
        "Mean transduction efficiency {:.4} must be >= 0.50",
        report.mean_transduction_efficiency
    );
    assert!(
        report.min_transduction_efficiency >= 0.50,
        "Min transduction efficiency {:.4} must be >= 0.50",
        report.min_transduction_efficiency
    );
    assert!(
        report.mean_phonon_occupation < 1.0,
        "Mean phonon occupation {:.5} must be < 1.0",
        report.mean_phonon_occupation
    );
    assert!(
        report.max_phonon_occupation < 1.0,
        "Max phonon occupation {:.5} must be < 1.0",
        report.max_phonon_occupation
    );
    assert!(
        (report.compliance_fraction - 1.0).abs() < 1e-6,
        "Compliance fraction {:.4} must be 100%",
        report.compliance_fraction
    );
    assert!(
        report.throughput_cycles_per_sec > 10_000.0,
        "Throughput {:.2} sweeps/sec must exceed 10,000 sweeps/sec",
        report.throughput_cycles_per_sec
    );
}
