//! Parallel Rayon benchmark verifying non-Hermitian skin effect,
//! skin depth (<= 3.0 cells), localization contrast (>= 25.0 dB),
//! sensitivity enhancement (>= 10.0x), laser threshold (<= 5.0 mW),
//! SMSR (>= 25.0 dB), and directional gain (>= 25.0 dB) across 10,000 parameter sweeps.

use phonon_solver::non_hermitian_topo::NonHermitianTopoBenchmarkRunner;

#[test]
fn test_non_hermitian_topo_parallel_benchmark_10k_sweeps() {
    let runner = NonHermitianTopoBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!("==================================================================");
    println!("PHASE 81: NON-HERMITIAN SKIN EFFECT & TOPOLOGICAL PHONON LASERS");
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
        "Mean Skin Depth:                 {:.2} cells (<= 3.0 cells required)",
        report.mean_skin_depth
    );
    println!(
        "Max Skin Depth:                  {:.2} cells",
        report.max_skin_depth
    );
    println!(
        "Mean Skin Contrast:              {:.2} dB (>= 25.0 dB required)",
        report.mean_skin_contrast_db
    );
    println!(
        "Min Skin Contrast:               {:.2} dB",
        report.min_skin_contrast_db
    );
    println!(
        "Mean Sensitivity Enhancement:    {:.1}x (>= 10.0x required)",
        report.mean_sensitivity_enhancement
    );
    println!(
        "Min Sensitivity Enhancement:     {:.1}x",
        report.min_sensitivity_enhancement
    );
    println!(
        "Mean Laser Threshold:            {:.2} mW (<= 5.0 mW required)",
        report.mean_laser_threshold_mw
    );
    println!(
        "Max Laser Threshold:             {:.2} mW",
        report.max_laser_threshold_mw
    );
    println!(
        "Mean SMSR:                       {:.2} dB (>= 25.0 dB required)",
        report.mean_smsr_db
    );
    println!(
        "Min SMSR:                        {:.2} dB",
        report.min_smsr_db
    );
    println!(
        "Mean Directional Gain:           {:.2} dB (>= 25.0 dB required)",
        report.mean_directional_gain_db
    );
    println!(
        "Min Directional Gain:            {:.2} dB",
        report.min_directional_gain_db
    );
    println!(
        "Compliance Fraction:             {:.4} (100% required)",
        report.compliance_fraction
    );
    println!("==================================================================");

    assert_eq!(report.total_cycles, 10_000);
    assert!(
        report.mean_skin_depth <= 3.0,
        "Mean skin depth {:.2} must be <= 3.0",
        report.mean_skin_depth
    );
    assert!(
        report.max_skin_depth <= 3.0,
        "Max skin depth {:.2} must be <= 3.0",
        report.max_skin_depth
    );
    assert!(
        report.mean_skin_contrast_db >= 25.0,
        "Mean skin contrast {:.2} dB must be >= 25.0 dB",
        report.mean_skin_contrast_db
    );
    assert!(
        report.min_skin_contrast_db >= 25.0,
        "Min skin contrast {:.2} dB must be >= 25.0 dB",
        report.min_skin_contrast_db
    );
    assert!(
        report.mean_sensitivity_enhancement >= 10.0,
        "Mean sensitivity enhancement {:.1}x must be >= 10.0x",
        report.mean_sensitivity_enhancement
    );
    assert!(
        report.min_sensitivity_enhancement >= 10.0,
        "Min sensitivity enhancement {:.1}x must be >= 10.0x",
        report.min_sensitivity_enhancement
    );
    assert!(
        report.mean_laser_threshold_mw <= 5.0,
        "Mean laser threshold {:.2} mW must be <= 5.0 mW",
        report.mean_laser_threshold_mw
    );
    assert!(
        report.max_laser_threshold_mw <= 5.0,
        "Max laser threshold {:.2} mW must be <= 5.0 mW",
        report.max_laser_threshold_mw
    );
    assert!(
        report.mean_smsr_db >= 25.0,
        "Mean SMSR {:.2} dB must be >= 25.0 dB",
        report.mean_smsr_db
    );
    assert!(
        report.min_smsr_db >= 25.0,
        "Min SMSR {:.2} dB must be >= 25.0 dB",
        report.min_smsr_db
    );
    assert!(
        report.mean_directional_gain_db >= 25.0,
        "Mean directional gain {:.2} dB must be >= 25.0 dB",
        report.mean_directional_gain_db
    );
    assert!(
        report.min_directional_gain_db >= 25.0,
        "Min directional gain {:.2} dB must be >= 25.0 dB",
        report.min_directional_gain_db
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
