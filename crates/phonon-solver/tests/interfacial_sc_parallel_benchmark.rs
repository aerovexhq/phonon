//! Parallel Rayon benchmark test for interfacial high-Tc superconductivity,
//! nematic fluctuations, and Josephson diode arrays across 10,000 parameter sweeps.

use phonon_solver::interfacial_superconductivity::InterfacialScBenchmarkRunner;

#[test]
fn test_interfacial_sc_parallel_benchmark_10000_sweeps() {
    let runner = InterfacialScBenchmarkRunner::new(10_000);
    let report = runner.run_parallel_benchmark();

    assert_eq!(
        report.total_cycles, 10_000,
        "Benchmark must execute exactly 10,000 sweeps"
    );
    assert!(
        report.elapsed_seconds > 0.0,
        "Benchmark elapsed time must be non-zero"
    );
    assert!(
        report.throughput_cycles_per_sec > 10_000.0,
        "Benchmark throughput must exceed 10,000 sweeps/sec, achieved {:.2} sweeps/sec",
        report.throughput_cycles_per_sec
    );

    // Physical requirements
    assert!(
        report.mean_critical_temp_k > 65.0,
        "Mean critical temperature must exceed 65 K, got {:.2} K",
        report.mean_critical_temp_k
    );
    assert!(
        report.min_critical_temp_k > 65.0,
        "Minimum critical temperature across all sweeps must exceed 65 K, got {:.2} K",
        report.min_critical_temp_k
    );

    assert!(
        report.mean_diode_efficiency >= 0.20,
        "Mean diode efficiency must be >= 0.20, got {:.4}",
        report.mean_diode_efficiency
    );
    assert!(
        report.min_diode_efficiency >= 0.20,
        "Minimum diode efficiency across all sweeps must be >= 0.20, got {:.4}",
        report.min_diode_efficiency
    );

    assert!(
        report.mean_rectification_ratio_db >= 3.0,
        "Mean rectification ratio must be >= 3.0 dB, got {:.2} dB",
        report.mean_rectification_ratio_db
    );
    assert!(
        report.min_rectification_ratio_db >= 3.0,
        "Minimum rectification ratio across all sweeps must be >= 3.0 dB, got {:.2} dB",
        report.min_rectification_ratio_db
    );

    assert!(
        report.mean_bcs_ratio >= 3.8,
        "Mean BCS strong-coupling ratio must be >= 3.8, got {:.3}",
        report.mean_bcs_ratio
    );

    assert_eq!(
        report.compliance_fraction,
        1.0,
        "All sweeps must be 100% compliant with physical criteria, got {:.2}%",
        report.compliance_fraction * 100.0
    );

    println!(
        "Phase 75 Interfacial Superconductivity Benchmark Completed:\n         - Sweeps: {}\n         - Elapsed: {:.4} s\n         - Throughput: {:.2} sweeps/sec\n         - Mean Tc: {:.2} K (min: {:.2} K)\n         - Mean Diode Efficiency: {:.2}% (min: {:.2}%)\n         - Mean Rectification: {:.2} dB (min: {:.2} dB)\n         - Mean BCS 2Delta/kBTc: {:.3}\n         - Compliance: {:.1}%",
        report.total_cycles,
        report.elapsed_seconds,
        report.throughput_cycles_per_sec,
        report.mean_critical_temp_k,
        report.min_critical_temp_k,
        report.mean_diode_efficiency * 100.0,
        report.min_diode_efficiency * 100.0,
        report.mean_rectification_ratio_db,
        report.min_rectification_ratio_db,
        report.mean_bcs_ratio,
        report.compliance_fraction * 100.0
    );
}
