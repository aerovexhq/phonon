//! Multi-threaded Rayon benchmark verification for topological acoustic circulators and cloaks across 10,000 sweeps.

use phonon_solver::metamaterial_circulator_cloak::MetamaterialCirculatorCloakBenchmarkRunner;

#[test]
fn test_metamaterial_circulator_cloak_parallel_benchmark() {
    let runner = MetamaterialCirculatorCloakBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!(
        "\n=== Topological Acoustic Circulators & Cloaks 10,000 Parameter Sweep Benchmark ==="
    );
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
        "Mean Cloaking Reduction:      {:.2} dB (min: {:.2} dB, max: {:.2} dB)",
        report.mean_cloaking_reduction_db,
        report.min_cloaking_reduction_db,
        report.max_cloaking_reduction_db
    );
    println!(
        "Mean Circulator Isolation:    {:.2} dB (min: {:.2} dB)",
        report.mean_circulator_isolation_db, report.min_circulator_isolation_db
    );
    println!(
        "Mean Insertion Loss:          {:.3} dB (max: {:.3} dB)",
        report.mean_forward_insertion_loss_db, report.max_forward_insertion_loss_db
    );
    println!(
        "Mean Bend Transmission:       {:.2}% (min: {:.2}%)",
        report.mean_topological_bend_transmission_pct, report.min_topological_bend_transmission_pct
    );
    println!(
        "Mean Linear Contrast Ratio:   {:.1} (threshold: >= 100.0)",
        report.mean_non_reciprocal_contrast_ratio
    );
    println!(
        "Physical Compliance Fraction: {:.4} (100% required)",
        report.compliance_fraction
    );

    assert_eq!(report.total_cycles, 10_000);
    assert!(report.mean_cloaking_reduction_db >= 20.0);
    assert!(report.min_cloaking_reduction_db >= 20.0);
    assert!(report.mean_circulator_isolation_db >= 25.0);
    assert!(report.min_circulator_isolation_db >= 25.0);
    assert!(report.max_forward_insertion_loss_db <= 1.5);
    assert!(report.min_topological_bend_transmission_pct >= 90.0);
    assert!(report.mean_non_reciprocal_contrast_ratio >= 100.0);
    assert_eq!(
        report.compliance_fraction, 1.0,
        "All 10,000 parameter sweeps must satisfy physical bounds"
    );
}
