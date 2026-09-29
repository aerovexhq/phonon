//! Parallel Rayon benchmark verifying chiral phonon-driven superconductivity,
//! transient pairing enhancement (> 50.0%), parametric gain (>= 15.0 dB),
//! modulation contrast (>= 20.0 dB), and ultrafast switching (<= 0.5 ps) across 10,000 parameter sweeps.

use phonon_solver::chiral_phonon_sc::ChiralPhononScBenchmarkRunner;

#[test]
fn test_chiral_phonon_sc_parallel_benchmark_10k_sweeps() {
    let runner = ChiralPhononScBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!("==================================================================");
    println!("PHASE 78: CHIRAL PHONON SUPERCONDUCTIVITY & JOSEPHSON MODULATION");
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
        "Mean Pairing Enhancement:        {:.2}% (> 50.0% required)",
        report.mean_pairing_enhancement_percent
    );
    println!(
        "Min Pairing Enhancement:         {:.2}%",
        report.min_pairing_enhancement_percent
    );
    println!(
        "Mean Parametric Gain:            {:.2} dB (>= 15.0 dB required)",
        report.mean_parametric_gain_db
    );
    println!(
        "Min Parametric Gain:             {:.2} dB",
        report.min_parametric_gain_db
    );
    println!(
        "Mean Modulation Contrast:        {:.2} dB (>= 20.0 dB required)",
        report.mean_modulation_contrast_db
    );
    println!(
        "Min Modulation Contrast:         {:.2} dB",
        report.min_modulation_contrast_db
    );
    println!(
        "Mean Switching Time:             {:.3} ps (<= 0.50 ps required)",
        report.mean_switching_time_ps
    );
    println!(
        "Max Switching Time:              {:.3} ps",
        report.max_switching_time_ps
    );
    println!(
        "Compliance Fraction:             {:.4} (100% required)",
        report.compliance_fraction
    );
    println!("==================================================================");

    assert_eq!(report.total_cycles, 10_000);
    assert!(
        report.mean_pairing_enhancement_percent > 50.0,
        "Mean pairing enhancement {:.2}% must exceed 50.0%",
        report.mean_pairing_enhancement_percent
    );
    assert!(
        report.min_pairing_enhancement_percent > 50.0,
        "Min pairing enhancement {:.2}% must exceed 50.0%",
        report.min_pairing_enhancement_percent
    );
    assert!(
        report.mean_parametric_gain_db >= 15.0,
        "Mean parametric gain {:.2} dB must be >= 15.0 dB",
        report.mean_parametric_gain_db
    );
    assert!(
        report.min_parametric_gain_db >= 15.0,
        "Min parametric gain {:.2} dB must be >= 15.0 dB",
        report.min_parametric_gain_db
    );
    assert!(
        report.mean_modulation_contrast_db >= 20.0,
        "Mean modulation contrast {:.2} dB must be >= 20.0 dB",
        report.mean_modulation_contrast_db
    );
    assert!(
        report.min_modulation_contrast_db >= 20.0,
        "Min modulation contrast {:.2} dB must be >= 20.0 dB",
        report.min_modulation_contrast_db
    );
    assert!(
        report.mean_switching_time_ps <= 0.50,
        "Mean switching time {:.3} ps must be <= 0.50 ps",
        report.mean_switching_time_ps
    );
    assert!(
        report.max_switching_time_ps <= 0.50,
        "Max switching time {:.3} ps must be <= 0.50 ps",
        report.max_switching_time_ps
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
