//! Integration tests for JTWPA broadband amplification and parallel Rayon benchmarks.

use phonon_models::jtwpa::ParametricProcessParams;
use phonon_solver::jtwpa::{CoupledModeSolver, JtwpaBenchmarkRunner};

#[test]
fn test_gain_bandwidth_profile() {
    let params = ParametricProcessParams::standard_3wm_amplifier();
    let (max_gain_db, bw_hz, _f_low) =
        CoupledModeSolver::compute_gain_bandwidth(&params, 4.0e9, 8.0e9, 30);

    assert!(
        max_gain_db > 18.0,
        "Peak gain should exceed 18 dB, got {} dB",
        max_gain_db
    );

    let bw_ghz = bw_hz * 1.0e-9;
    assert!(
        bw_ghz >= 2.5,
        "3-dB bandwidth should be broad (>= 2.5 GHz), got {} GHz",
        bw_ghz
    );
}

#[test]
fn test_jtwpa_parallel_10k_pulses_benchmark() {
    let report = JtwpaBenchmarkRunner::run_benchmark(10_000);

    assert!(
        report.peak_gain_db > 18.0,
        "Peak gain must exceed 18 dB, got {} dB",
        report.peak_gain_db
    );

    assert!(
        report.gain_bandwidth_ghz >= 2.5,
        "Bandwidth must be >= 2.5 GHz, got {} GHz",
        report.gain_bandwidth_ghz
    );

    assert!(
        report.added_noise_quanta <= 0.505,
        "Added noise quanta must be <= 0.505 (quantum-limited), got {}",
        report.added_noise_quanta
    );

    assert!(
        report.saturation_power_1db_dbm > -90.0,
        "1-dB saturation power must exceed -90 dBm, got {} dBm",
        report.saturation_power_1db_dbm
    );

    assert!(
        report.max_manley_rowe_error < 1.0e-6,
        "Max Manley-Rowe error must be < 1e-6, got {}",
        report.max_manley_rowe_error
    );

    assert_eq!(report.total_pulses_evaluated, 10_000);

    assert!(
        report.throughput_pulses_per_sec > 40_000.0,
        "Throughput must exceed 40,000 pulses/sec, got {}",
        report.throughput_pulses_per_sec
    );
}
