//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for quantum acoustic frequency combs and phononic microresonator soliton synthesizers.

#![deny(unsafe_code)]

use phonon_solver::acoustic_microcomb_soliton::MicrocombBenchmarkRunner;

#[test]
fn test_10k_acoustic_microcomb_parallel_sweep() {
    let cycles = 10_000;
    let result = MicrocombBenchmarkRunner::run_benchmark(cycles);

    assert_eq!(
        result.total_cycles, cycles,
        "Total benchmark cycles must equal requested 10,000"
    );

    println!(
        "Acoustic Microcomb 10k Sweep: f_rep={:.4} GHz (min {:.4} GHz), stab={:.4e} (max {:.4e}), eff={:.4} (min {:.4}), PN={:.2} dBc/Hz (max {:.2} dBc/Hz), span={:.4} octaves (min {:.4}), jitter={:.4} fs (max {:.4} fs), compliance={:.1}%, throughput={:.2} sweeps/sec",
        result.mean_repetition_rate_ghz,
        result.min_repetition_rate_ghz,
        result.mean_spacing_stability,
        result.max_spacing_stability,
        result.mean_conversion_efficiency,
        result.min_conversion_efficiency,
        result.mean_phase_noise_dbc,
        result.max_phase_noise_dbc,
        result.mean_octave_span,
        result.min_octave_span,
        result.mean_timing_jitter_fs,
        result.max_timing_jitter_fs,
        result.physical_compliance_fraction * 100.0,
        result.throughput_sweeps_per_sec,
    );

    // Verify 100% physical compliance across all parameter sweep variations
    assert!(
        (result.physical_compliance_fraction - 1.0).abs() < 1.0e-6,
        "Physical compliance fraction must be 100.0%, got {:.2}%",
        result.physical_compliance_fraction * 100.0
    );

    // Assert repetition rate >= 1.0 GHz
    assert!(
        result.mean_repetition_rate_ghz >= 1.0,
        "Mean comb repetition rate must be >= 1.0 GHz, got {:.4} GHz",
        result.mean_repetition_rate_ghz
    );
    assert!(
        result.min_repetition_rate_ghz >= 1.0,
        "Worst-case comb repetition rate must be >= 1.0 GHz, got {:.4} GHz",
        result.min_repetition_rate_ghz
    );

    // Assert comb spacing stability <= 1.0e-11
    assert!(
        result.mean_spacing_stability <= 1.0e-11,
        "Mean comb spacing stability must be <= 1.0e-11, got {:.4e}",
        result.mean_spacing_stability
    );
    assert!(
        result.max_spacing_stability <= 1.0e-11,
        "Worst-case comb spacing stability must be <= 1.0e-11, got {:.4e}",
        result.max_spacing_stability
    );

    // Assert conversion efficiency >= 0.350
    assert!(
        result.mean_conversion_efficiency >= 0.350,
        "Mean conversion efficiency must be >= 0.350, got {:.4}",
        result.mean_conversion_efficiency
    );
    assert!(
        result.min_conversion_efficiency >= 0.350,
        "Worst-case conversion efficiency must be >= 0.350, got {:.4}",
        result.min_conversion_efficiency
    );

    // Assert phase noise <= -125.0 dBc/Hz
    assert!(
        result.mean_phase_noise_dbc <= -125.0,
        "Mean phase noise must be <= -125.0 dBc/Hz, got {:.2} dBc/Hz",
        result.mean_phase_noise_dbc
    );
    assert!(
        result.max_phase_noise_dbc <= -125.0,
        "Worst-case phase noise must be <= -125.0 dBc/Hz, got {:.2} dBc/Hz",
        result.max_phase_noise_dbc
    );

    // Assert octave span >= 1.00 octaves
    assert!(
        result.mean_octave_span >= 1.00,
        "Mean comb octave span must be >= 1.00 octaves, got {:.4}",
        result.mean_octave_span
    );
    assert!(
        result.min_octave_span >= 1.00,
        "Worst-case comb octave span must be >= 1.00 octaves, got {:.4}",
        result.min_octave_span
    );

    // Assert timing jitter <= 5.0 fs
    assert!(
        result.mean_timing_jitter_fs <= 5.0,
        "Mean timing jitter must be <= 5.0 fs, got {:.4} fs",
        result.mean_timing_jitter_fs
    );
    assert!(
        result.max_timing_jitter_fs <= 5.0,
        "Worst-case timing jitter must be <= 5.0 fs, got {:.4} fs",
        result.max_timing_jitter_fs
    );

    assert!(
        result.throughput_sweeps_per_sec > 0.0,
        "Throughput must be positive, got {:.2} sweeps/sec",
        result.throughput_sweeps_per_sec
    );
}
