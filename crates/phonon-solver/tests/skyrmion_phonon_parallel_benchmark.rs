//! Multi-threaded Rayon benchmark verification for chiral skyrmion-phonon drag across 10,000 parameter sweeps.

use phonon_solver::skyrmion_phonon_drag::SkyrmionPhononBenchmarkRunner;

#[test]
fn test_skyrmion_phonon_parallel_benchmark() {
    let runner = SkyrmionPhononBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!("\n=== Chiral Skyrmion-Phonon Drag 10,000 Parameter Sweep Benchmark ===");
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
        "Mean Drift Velocity:          {:.2} m/s (min: {:.2} m/s, max: {:.2} m/s)",
        report.mean_drift_velocity_m_s,
        report.min_drift_velocity_m_s,
        report.max_drift_velocity_m_s
    );
    println!(
        "Mean Hall Deflection Angle:   {:.2} deg (min: {:.2} deg, max: {:.2} deg)",
        report.mean_hall_angle_deg, report.min_hall_angle_deg, report.max_hall_angle_deg
    );
    println!(
        "Mean Acoustic Drag Force:     {:.3} pN",
        report.mean_drag_force_pn
    );
    println!(
        "Mean Logic Contrast:          {:.2} dB (min: {:.2} dB)",
        report.mean_logic_contrast_db, report.min_logic_contrast_db
    );
    println!(
        "Mean Circulator Isolation:    {:.2} dB (min: {:.2} dB)",
        report.mean_circulator_isolation_db, report.min_circulator_isolation_db
    );
    println!(
        "Mean Energy per Bit:          {:.3} fJ (max: {:.3} fJ)",
        report.mean_energy_bit_fj, report.max_energy_bit_fj
    );
    println!(
        "Mean Stability Factor:        {:.2}",
        report.mean_stability_factor
    );
    println!(
        "Physical Compliance Fraction: {:.4} (100% required)",
        report.compliance_fraction
    );

    assert_eq!(report.total_cycles, 10_000);
    assert!(report.mean_drift_velocity_m_s > 100.0);
    assert!(report.min_drift_velocity_m_s > 100.0);
    assert!(report.mean_hall_angle_deg >= 10.0 && report.mean_hall_angle_deg <= 70.0);
    assert!(report.mean_logic_contrast_db >= 25.0);
    assert!(report.min_logic_contrast_db >= 25.0);
    assert!(report.mean_circulator_isolation_db >= 20.0);
    assert!(report.min_circulator_isolation_db >= 20.0);
    assert!(report.mean_energy_bit_fj < 1.0);
    assert!(report.max_energy_bit_fj < 1.0);
    assert!(report.mean_stability_factor >= 1.0);
    assert_eq!(
        report.compliance_fraction, 1.0,
        "All 10,000 parameter sweeps must satisfy physical bounds"
    );
}
