//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for cavity acoustomagnonic haloscopes across multi-threaded Rayon workers.

use phonon_solver::cavity_acoustomagnonic::AcoustomagnonicBenchmarkRunner;

#[test]
fn test_10k_acoustomagnonic_parallel_sweep() {
    let cycles = 10_000;
    let result = AcoustomagnonicBenchmarkRunner::run_benchmark(cycles);

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

    // Assert conversion gain target >= 22.0 dB
    assert!(
        result.mean_conversion_gain_db >= 22.0,
        "Mean conversion gain must be >= 22.0 dB, got {:.2} dB",
        result.mean_conversion_gain_db
    );
    assert!(
        result.min_conversion_gain_db >= 22.0,
        "Worst-case conversion gain must be >= 22.0 dB, got {:.2} dB",
        result.min_conversion_gain_db
    );

    // Assert readout SNR target >= 28.0 dB
    assert!(
        result.mean_snr_db >= 28.0,
        "Mean haloscope SNR must be >= 28.0 dB, got {:.2} dB",
        result.mean_snr_db
    );
    assert!(
        result.min_snr_db >= 28.0,
        "Worst-case haloscope SNR must be >= 28.0 dB, got {:.2} dB",
        result.min_snr_db
    );

    // Assert exclusion scan rate >= 1.0 GHz/day
    assert!(
        result.mean_scan_rate_ghz_per_day >= 1.0,
        "Mean exclusion scan rate must be >= 1.0 GHz/day, got {:.3} GHz/day",
        result.mean_scan_rate_ghz_per_day
    );
    assert!(
        result.min_scan_rate_ghz_per_day >= 1.0,
        "Worst-case exclusion scan rate must be >= 1.0 GHz/day, got {:.3} GHz/day",
        result.min_scan_rate_ghz_per_day
    );

    // Assert cooperativity C_ma >= 150.0
    assert!(
        result.mean_cooperativity >= 150.0,
        "Mean cooperativity must be >= 150.0, got {:.2}",
        result.mean_cooperativity
    );
    assert!(
        result.min_cooperativity >= 150.0,
        "Worst-case cooperativity must be >= 150.0, got {:.2}",
        result.min_cooperativity
    );
}
