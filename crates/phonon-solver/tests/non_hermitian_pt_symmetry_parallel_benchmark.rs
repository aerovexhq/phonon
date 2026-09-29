//! Parallel Rayon benchmark test for acoustic PT-symmetry across 10,000 parameter sweeps.

use phonon_solver::non_hermitian_pt_symmetry::PtSymmetryBenchmarkRunner;

#[test]
fn test_acoustic_pt_symmetry_10k_benchmark() {
    let runner = PtSymmetryBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!("--- Acoustic PT-Symmetry Benchmark Report ---");
    println!("Total parameter sweeps: {}", report.total_cycles);
    println!("Elapsed time: {:.4} s", report.elapsed_seconds);
    println!(
        "Throughput: {:.2} sweeps/sec",
        report.throughput_cycles_per_sec
    );
    println!(
        "Mean Sensitivity Enhancement: {:.2} dB (min: {:.2} dB, max: {:.2} dB)",
        report.mean_enhancement_db, report.min_enhancement_db, report.max_enhancement_db
    );
    println!(
        "Mean Threshold Power: {:.3} mW (max: {:.3} mW)",
        report.mean_threshold_power_mw, report.max_threshold_power_mw
    );
    println!(
        "Mean Reverse Isolation: {:.2} dB (min: {:.2} dB)",
        report.mean_reverse_isolation_db, report.min_reverse_isolation_db
    );
    println!(
        "Mean Directional Absorption: {:.2}% (min: {:.2}%)",
        report.mean_directional_absorption_pct, report.min_directional_absorption_pct
    );
    println!(
        "Mean Petermann Phase Rigidity: {:.4} (max: {:.4})",
        report.mean_phase_rigidity, report.max_phase_rigidity
    );
    println!(
        "Mean Coalescence Fidelity: {:.2}%",
        report.mean_coalescence_fidelity_pct
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
        report.mean_enhancement_db >= 35.0,
        "Mean enhancement must be >= 35.0 dB, got {}",
        report.mean_enhancement_db
    );
    assert!(
        report.min_enhancement_db >= 35.0,
        "Min enhancement must be >= 35.0 dB, got {}",
        report.min_enhancement_db
    );
    assert!(
        report.mean_threshold_power_mw <= 2.0,
        "Mean threshold power must be <= 2.0 mW, got {}",
        report.mean_threshold_power_mw
    );
    assert!(
        report.max_threshold_power_mw <= 2.0,
        "Max threshold power must be <= 2.0 mW, got {}",
        report.max_threshold_power_mw
    );
    assert!(
        report.mean_reverse_isolation_db >= 25.0,
        "Mean reverse isolation must be >= 25.0 dB, got {}",
        report.mean_reverse_isolation_db
    );
    assert!(
        report.min_reverse_isolation_db >= 25.0,
        "Min reverse isolation must be >= 25.0 dB, got {}",
        report.min_reverse_isolation_db
    );
    assert!(
        report.mean_directional_absorption_pct >= 90.0,
        "Mean directional absorption must be >= 90.0%, got {}",
        report.mean_directional_absorption_pct
    );
    assert!(
        report.min_directional_absorption_pct >= 90.0,
        "Min directional absorption must be >= 90.0%, got {}",
        report.min_directional_absorption_pct
    );
    assert!(
        report.mean_phase_rigidity <= 0.20,
        "Mean phase rigidity must be <= 0.20, got {}",
        report.mean_phase_rigidity
    );
    assert!(
        report.max_phase_rigidity <= 0.20,
        "Max phase rigidity must be <= 0.20, got {}",
        report.max_phase_rigidity
    );
    assert!(
        report.mean_coalescence_fidelity_pct >= 95.0,
        "Mean coalescence fidelity must be >= 95.0%, got {}",
        report.mean_coalescence_fidelity_pct
    );
    assert!(
        (report.compliance_fraction - 1.0).abs() < 1e-6,
        "Compliance fraction must be 100%, got {}",
        report.compliance_fraction
    );
}
