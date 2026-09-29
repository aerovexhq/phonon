//! Parallel Rayon benchmark test for Terahertz magnon polaritons,
//! quantum paramagnons, and antiferromagnetic spintronics across 10,000 parameter sweeps.

use phonon_solver::afm_spintronics::AfmMagnonBenchmarkRunner;

#[test]
fn test_afm_spintronics_parallel_benchmark_10000_sweeps() {
    let runner = AfmMagnonBenchmarkRunner::new(10_000);
    let report = runner.run_parallel_benchmark();

    assert_eq!(
        report.total_cycles, 10_000,
        "Benchmark must execute exactly 10,000 sweeps"
    );
    assert!(
        report.elapsed_seconds > 0.0,
        "Elapsed time must be non-zero"
    );
    assert!(
        report.throughput_cycles_per_sec > 10_000.0,
        "Throughput must exceed 10,000 sweeps/sec, achieved {:.2} sweeps/sec",
        report.throughput_cycles_per_sec
    );

    // Physical requirements
    assert!(
        report.mean_rabi_splitting_ghz > 100.0,
        "Mean vacuum Rabi splitting must exceed 100 GHz, got {:.2} GHz",
        report.mean_rabi_splitting_ghz
    );
    assert!(
        report.min_rabi_splitting_ghz > 100.0,
        "Minimum Rabi splitting across all sweeps must exceed 100 GHz, got {:.2} GHz",
        report.min_rabi_splitting_ghz
    );

    assert!(
        report.mean_dw_velocity_m_s > 5000.0,
        "Mean domain wall velocity must exceed 5000 m/s, got {:.1} m/s",
        report.mean_dw_velocity_m_s
    );
    assert!(
        report.min_dw_velocity_m_s > 5000.0,
        "Minimum domain wall velocity must exceed 5000 m/s, got {:.1} m/s",
        report.min_dw_velocity_m_s
    );

    assert!(
        report.mean_transit_time_ps < 1.0,
        "Mean synaptic transit time must be sub-picosecond (< 1.0 ps), got {:.3} ps",
        report.mean_transit_time_ps
    );
    assert!(
        report.max_transit_time_ps < 1.0,
        "Maximum synaptic transit time must remain < 1.0 ps, got {:.3} ps",
        report.max_transit_time_ps
    );

    assert!(
        report.mean_diode_rectification_db >= 15.0,
        "Mean magnon diode rectification must be >= 15.0 dB, got {:.2} dB",
        report.mean_diode_rectification_db
    );
    assert!(
        report.min_diode_rectification_db >= 15.0,
        "Minimum magnon diode rectification must be >= 15.0 dB, got {:.2} dB",
        report.min_diode_rectification_db
    );

    assert!(
        report.mean_cooperativity > 100.0,
        "Mean polariton cooperativity must exceed 100, got {:.1}",
        report.mean_cooperativity
    );

    assert_eq!(
        report.compliance_fraction,
        1.0,
        "All sweeps must be 100% compliant with physical criteria, got {:.2}%",
        report.compliance_fraction * 100.0
    );

    println!(
        "Phase 76 AFM Spintronics Benchmark Completed:\n\
         - Sweeps: {}\n\
         - Elapsed: {:.4} s\n\
         - Throughput: {:.2} sweeps/sec\n\
         - Mean Rabi Splitting: {:.2} GHz (min: {:.2} GHz)\n\
         - Mean DW Velocity: {:.1} m/s (min: {:.1} m/s)\n\
         - Mean Transit Time: {:.3} ps (max: {:.3} ps)\n\
         - Mean Diode Rectification: {:.2} dB (min: {:.2} dB)\n\
         - Mean Cooperativity: {:.1}\n\
         - Compliance: {:.1}%",
        report.total_cycles,
        report.elapsed_seconds,
        report.throughput_cycles_per_sec,
        report.mean_rabi_splitting_ghz,
        report.min_rabi_splitting_ghz,
        report.mean_dw_velocity_m_s,
        report.min_dw_velocity_m_s,
        report.mean_transit_time_ps,
        report.max_transit_time_ps,
        report.mean_diode_rectification_db,
        report.min_diode_rectification_db,
        report.mean_cooperativity,
        report.compliance_fraction * 100.0
    );
}
