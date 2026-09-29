//! Multi-threaded Rayon benchmark verification for Non-Hermitian Chiral HOTIs across 10,000 parameter sweeps.

use phonon_solver::non_hermitian_chiral_hoti::NonHermitianHotiBenchmarkRunner;

#[test]
fn test_non_hermitian_hoti_parallel_benchmark() {
    let runner = NonHermitianHotiBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!("\n=== Non-Hermitian Chiral HOTI 10,000 Parameter Sweep Benchmark ===");
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
        "Mean Corner Contrast:         {:.2} dB (min: {:.2} dB, max: {:.2} dB)",
        report.mean_localization_contrast_db,
        report.min_localization_contrast_db,
        report.max_localization_contrast_db
    );
    println!(
        "Mean Corner Skin Depth:       {:.3} cells (max: {:.3} cells)",
        report.mean_skin_depth_cells, report.max_skin_depth_cells
    );
    println!(
        "Mean Acoustoelectric Rect:    {:.2} dB (min: {:.2} dB)",
        report.mean_rectification_db, report.min_rectification_db
    );
    println!(
        "Mean Corner Sensor SNR:       {:.2} dB (min: {:.2} dB)",
        report.mean_sensor_snr_db, report.min_sensor_snr_db
    );
    println!(
        "Mean Acoustoelectric Current: {:.2} A/m^2",
        report.mean_acoustoelectric_current_a_m2
    );
    println!(
        "Physical Compliance Fraction: {:.4} (100% required)",
        report.compliance_fraction
    );

    assert_eq!(report.total_cycles, 10_000);
    assert!(report.mean_localization_contrast_db >= 30.0);
    assert!(report.min_localization_contrast_db >= 30.0);
    assert!(report.mean_skin_depth_cells <= 2.0);
    assert!(report.max_skin_depth_cells <= 2.0);
    assert!(report.mean_rectification_db >= 20.0);
    assert!(report.min_rectification_db >= 20.0);
    assert!(report.mean_sensor_snr_db >= 20.0);
    assert!(report.min_sensor_snr_db >= 20.0);
    assert_eq!(
        report.compliance_fraction, 1.0,
        "All 10,000 parameter sweeps must satisfy physical bounds"
    );
}
