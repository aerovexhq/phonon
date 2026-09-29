//! Parallel Rayon benchmark verifying phononic Kerr microcombs,
//! octave-spanning comb generation (>= 1.0 octave), comb lines (>= 50),
//! timing jitter (<= 100 fs), and soliton contrast (>= 20.0 dB) across 10,000 parameter sweeps.

use phonon_solver::phononic_microcomb::PhononicMicrocombBenchmarkRunner;

#[test]
fn test_phononic_microcomb_parallel_benchmark_10k_sweeps() {
    let runner = PhononicMicrocombBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!("==================================================================");
    println!("PHASE 80: PHONONIC KERR SOLITON MICROCOMBS & NON-LINEAR PHONONICS");
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
        "Mean Comb Span:                  {:.2} octaves (>= 1.0 octave required)",
        report.mean_comb_span_octaves
    );
    println!(
        "Min Comb Span:                   {:.2} octaves",
        report.min_comb_span_octaves
    );
    println!(
        "Mean Comb Lines:                 {:.1} (>= 50 required)",
        report.mean_comb_lines
    );
    println!("Min Comb Lines:                  {}", report.min_comb_lines);
    println!(
        "Mean Timing Jitter:              {:.2} fs (<= 100.0 fs required)",
        report.mean_timing_jitter_fs
    );
    println!(
        "Max Timing Jitter:               {:.2} fs",
        report.max_timing_jitter_fs
    );
    println!(
        "Mean Soliton Contrast:           {:.2} dB (>= 20.0 dB required)",
        report.mean_soliton_contrast_db
    );
    println!(
        "Min Soliton Contrast:            {:.2} dB",
        report.min_soliton_contrast_db
    );
    println!(
        "Compliance Fraction:             {:.4} (100% required)",
        report.compliance_fraction
    );
    println!("==================================================================");

    assert_eq!(report.total_cycles, 10_000);
    assert!(
        report.mean_comb_span_octaves >= 1.0,
        "Mean comb span {:.2} octaves must be >= 1.0",
        report.mean_comb_span_octaves
    );
    assert!(
        report.min_comb_span_octaves >= 1.0,
        "Min comb span {:.2} octaves must be >= 1.0",
        report.min_comb_span_octaves
    );
    assert!(
        report.mean_comb_lines >= 50.0,
        "Mean comb lines {:.1} must be >= 50",
        report.mean_comb_lines
    );
    assert!(
        report.min_comb_lines >= 50,
        "Min comb lines {} must be >= 50",
        report.min_comb_lines
    );
    assert!(
        report.mean_timing_jitter_fs <= 100.0,
        "Mean timing jitter {:.2} fs must be <= 100.0 fs",
        report.mean_timing_jitter_fs
    );
    assert!(
        report.max_timing_jitter_fs <= 100.0,
        "Max timing jitter {:.2} fs must be <= 100.0 fs",
        report.max_timing_jitter_fs
    );
    assert!(
        report.mean_soliton_contrast_db >= 20.0,
        "Mean soliton contrast {:.2} dB must be >= 20.0 dB",
        report.mean_soliton_contrast_db
    );
    assert!(
        report.min_soliton_contrast_db >= 20.0,
        "Min soliton contrast {:.2} dB must be >= 20.0 dB",
        report.min_soliton_contrast_db
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
