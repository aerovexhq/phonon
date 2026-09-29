//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for cavity magnon-polariton frequency combs across multi-threaded Rayon workers.

use phonon_solver::cavity_magnon_polariton_comb::MagnonPolaritonCombBenchmarkRunner;

#[test]
fn test_10k_magnon_polariton_comb_parallel_sweep() {
    let cycles = 10_000;
    let result = MagnonPolaritonCombBenchmarkRunner::run_benchmark(cycles);

    assert_eq!(
        result.total_cycles, cycles,
        "Total benchmark cycles must equal requested 10,000"
    );

    // Verify 100% physical compliance across all parameter sweep variations
    assert!(
        (result.physical_compliance_fraction - 1.0).abs() < 1.0e-6,
        "Physical compliance fraction must be 100.0%, got {:.2}%",
        result.physical_compliance_fraction * 100.0
    );

    // Assert threshold power <= 1.00 mW
    assert!(
        result.mean_threshold_power_mw <= 1.00,
        "Mean threshold pump power must be <= 1.00 mW, got {:.4} mW",
        result.mean_threshold_power_mw
    );
    assert!(
        result.max_threshold_power_mw <= 1.00,
        "Worst-case threshold pump power must be <= 1.00 mW, got {:.4} mW",
        result.max_threshold_power_mw
    );

    // Assert comb octave span >= 1.50 octaves
    assert!(
        result.mean_comb_octave_span >= 1.50,
        "Mean comb octave span must be >= 1.50 octaves, got {:.3}",
        result.mean_comb_octave_span
    );
    assert!(
        result.min_comb_octave_span >= 1.50,
        "Worst-case comb octave span must be >= 1.50 octaves, got {:.3}",
        result.min_comb_octave_span
    );

    // Assert sub-shot-noise improvement >= 6.00 dB
    assert!(
        result.mean_sub_shot_noise_db >= 6.00,
        "Mean sub-shot-noise improvement must be >= 6.00 dB, got {:.2} dB",
        result.mean_sub_shot_noise_db
    );
    assert!(
        result.min_sub_shot_noise_db >= 6.00,
        "Worst-case sub-shot-noise improvement must be >= 6.00 dB, got {:.2} dB",
        result.min_sub_shot_noise_db
    );

    // Assert polariton entanglement logarithmic negativity >= 0.850
    assert!(
        result.mean_polariton_log_negativity >= 0.850,
        "Mean polariton logarithmic negativity must be >= 0.850, got {:.4}",
        result.mean_polariton_log_negativity
    );
    assert!(
        result.min_polariton_log_negativity >= 0.850,
        "Worst-case polariton logarithmic negativity must be >= 0.850, got {:.4}",
        result.min_polariton_log_negativity
    );

    // Assert comb teeth count >= 40
    assert!(
        result.mean_comb_teeth_count >= 40.0,
        "Mean comb teeth count must be >= 40, got {:.1}",
        result.mean_comb_teeth_count
    );
    assert!(
        result.min_comb_teeth_count >= 40,
        "Worst-case comb teeth count must be >= 40, got {}",
        result.min_comb_teeth_count
    );

    assert!(
        result.throughput_sweeps_per_sec > 0.0,
        "Throughput must be positive, got {:.2} sweeps/sec",
        result.throughput_sweeps_per_sec
    );
}
