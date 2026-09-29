//! Integration tests for 10,000-sweep parallel Rayon KITWPA benchmark.

use phonon_solver::kitwpa::{KitwpaBenchmarkConfig, KitwpaBenchmarkRunner};

#[test]
fn test_kitwpa_parallel_benchmark_execution() {
    let runner = KitwpaBenchmarkRunner::new();
    let config = KitwpaBenchmarkConfig {
        total_sweeps: 10_000,
        ..Default::default()
    };

    let report = runner.run_benchmark(&config);

    assert_eq!(report.total_sweeps, 10_000);
    assert!(report.elapsed_ms > 0.0);
    assert!(
        report.sweeps_per_sec > 10_000.0,
        "Throughput should be > 10k sweeps/sec, got {:.0}",
        report.sweeps_per_sec
    );

    // Physics assertions
    assert!(
        report.mean_gain_db >= 20.0,
        "Mean gain must be >= 20 dB, got {:.2} dB",
        report.mean_gain_db
    );
    assert!(
        report.min_gain_db >= 19.5,
        "Min gain must be >= 19.5 dB, got {:.2} dB",
        report.min_gain_db
    );
    assert!(
        report.mean_bandwidth_ghz >= 4.0,
        "Mean bandwidth must be >= 4.0 GHz, got {:.2} GHz",
        report.mean_bandwidth_ghz
    );
    assert!(
        report.mean_saturation_power_dbm > -50.0,
        "Saturation power must exceed -50 dBm, got {:.2} dBm",
        report.mean_saturation_power_dbm
    );
    assert!(
        report.mean_added_noise_quanta <= 0.505,
        "Mean added noise quanta must be <= 0.505, got {:.4}",
        report.mean_added_noise_quanta
    );
    assert!(
        report.mean_scan_speedup > 100.0,
        "Mean scan speedup must exceed 100x, got {:.1}x",
        report.mean_scan_speedup
    );
    assert!(
        report.max_manley_rowe_error < 1e-6,
        "Max Manley-Rowe error must be < 1e-6, got {:e}",
        report.max_manley_rowe_error
    );
}
