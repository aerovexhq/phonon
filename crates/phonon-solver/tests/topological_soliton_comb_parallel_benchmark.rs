//! Multi-threaded Rayon benchmark verification for topological soliton combs across 10,000 parameter sweeps.

use phonon_solver::topological_soliton_comb::TopologicalSolitonCombBenchmarkRunner;

#[test]
fn test_topological_soliton_comb_parallel_benchmark() {
    let runner = TopologicalSolitonCombBenchmarkRunner::new(10_000);
    let report = runner.run_benchmark();

    println!(
        "\n=== Topological Soliton Microcavity Frequency Comb 10,000 Parameter Sweep Benchmark ==="
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
        "Mean Comb Bandwidth Span:     {:.2} octaves (min: {:.2} octaves, max: {:.2} octaves)",
        report.mean_comb_span_octaves, report.min_comb_span_octaves, report.max_comb_span_octaves
    );
    println!(
        "Mean Timing Jitter:           {:.3} fs (max: {:.3} fs)",
        report.mean_timing_jitter_fs, report.max_timing_jitter_fs
    );
    println!(
        "Mean Beat-Note SNR:           {:.2} dB (min: {:.2} dB)",
        report.mean_beat_note_snr_db, report.min_beat_note_snr_db
    );
    println!(
        "Mean Soliton Pulse Duration:  {:.2} ps (max: {:.2} ps)",
        report.mean_soliton_pulse_duration_ps, report.max_soliton_pulse_duration_ps
    );
    println!(
        "Mean Comb Lines Count:        {} (min: {})",
        report.mean_comb_lines_count, report.min_comb_lines_count
    );
    println!(
        "Mean Allan Deviation Floor:   {:.4e} (max: {:.4e})",
        report.mean_allan_deviation_floor, report.max_allan_deviation_floor
    );
    println!(
        "Physical Compliance Fraction: {:.4} (100% required)",
        report.compliance_fraction
    );

    assert_eq!(report.total_cycles, 10_000);
    assert!(report.mean_comb_span_octaves >= 2.0);
    assert!(report.min_comb_span_octaves >= 2.0);
    assert!(report.max_timing_jitter_fs <= 10.0);
    assert!(report.mean_beat_note_snr_db >= 30.0);
    assert!(report.min_beat_note_snr_db >= 30.0);
    assert!(report.max_soliton_pulse_duration_ps <= 15.0);
    assert!(report.min_comb_lines_count >= 100);
    assert!(report.max_allan_deviation_floor <= 1.0e-12);
    assert_eq!(
        report.compliance_fraction, 1.0,
        "All 10,000 parameter sweeps must satisfy physical bounds"
    );
}
