//! Integration tests for multi-threaded Rayon benchmark of Quantum LIDAR & Telemetry across 10,000 pulses.

use phonon_solver::snspd::QuantumLidarBenchmarkRunner;

#[test]
fn test_parallel_10k_pulses_quantum_lidar_ranging() {
    let runner = QuantumLidarBenchmarkRunner::standard();
    let report = runner.run();

    assert_eq!(report.total_pulses, 10_000);
    assert!(
        report.detected_pulses >= 9500,
        "Should detect at least 95% of pulses, got {}",
        report.detected_pulses
    );
    assert!(
        report.detection_efficiency >= 0.95,
        "Detection efficiency should be >= 95%, got {}",
        report.detection_efficiency
    );

    // Sub-50 ps timing jitter
    assert!(
        report.measured_jitter_rms_ps < 50.0,
        "Timing jitter RMS should be < 50 ps, got {} ps",
        report.measured_jitter_rms_ps
    );
    assert!(
        report.theoretical_jitter_ps < 50.0,
        "Theoretical jitter should be < 50 ps, got {} ps",
        report.theoretical_jitter_ps
    );

    // Dark count rate < 1.0 cps
    assert!(
        report.dark_count_rate_cps < 1.0,
        "Dark count rate should be < 1.0 cps, got {} cps",
        report.dark_count_rate_cps
    );

    // Sub-millimeter ranging precision: ranging_error_mm < 1.0 mm
    assert!(
        report.ranging_error_mm < 1.0,
        "Ranging error must be sub-millimeter (< 1.0 mm), got {} mm",
        report.ranging_error_mm
    );

    // Performance and throughput
    assert!(
        report.throughput_pulses_per_sec > 10_000.0,
        "Throughput should be > 10,000 pulses/sec, got {}",
        report.throughput_pulses_per_sec
    );
    assert!(
        report.passed_criteria,
        "All quantum LIDAR benchmark criteria should pass"
    );
}
