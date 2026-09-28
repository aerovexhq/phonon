//! Integration tests for STNO/SHNO auto-oscillation, Adler injection locking,
//! and parallel Rayon benchmarks.

use phonon_models::stno::{InjectionLockingParams, StnoParams};
use phonon_solver::stno::StnoBenchmarkRunner;

#[test]
fn test_stno_threshold_current_and_fmr() {
    let stno = StnoParams::standard_cofeb_stno();
    let h_ext = 1.6e5; // ~0.2 T

    let ith = stno.threshold_current_amperes(h_ext);
    assert!(
        ith > 1.0e-5 && ith < 5.0e-3,
        "Threshold current should be in sub-mA to mA range, got {} A",
        ith
    );

    let f_fmr = stno.kittel_fmr_frequency_rad_per_s(h_ext) / (2.0 * std::f64::consts::PI);
    assert!(
        f_fmr > 1.0e9 && f_fmr < 25.0e9,
        "FMR frequency should be in microwave GHz range (1-25 GHz), got {} Hz",
        f_fmr
    );
}

#[test]
fn test_adler_injection_locking() {
    let locking = InjectionLockingParams::new(
        3.0e9,   // 3.0 GHz oscillator
        3.005e9, // 3.005 GHz injection signal (5 MHz detuning)
        1.0e-6,  // 1 uW injection power
        10.0e-6, // 10 uW oscillator power
        50.0,    // loaded Q
    );

    let bandwidth = locking.locking_bandwidth_hz();
    assert!(
        bandwidth > 1.0e6 && bandwidth < 50.0e6,
        "Locking bandwidth should be several MHz, got {} Hz",
        bandwidth
    );

    assert!(locking.is_locked());
    let phase = locking.steady_state_phase_offset_rad();
    assert!(phase.is_some());
}

#[test]
fn test_stno_parallel_benchmark() {
    // Run 5 oscillators for 200 cycles each = 1,000 cycles
    let report = StnoBenchmarkRunner::run_benchmark(5, 200);

    assert!(report.fe8_qtm_splitting_k > 0.0);
    assert!(report.kondo_zero_bias_conductance > 0.0);
    assert!(report.kondo_zeeman_splitting_mev > 0.0);
    assert!(
        report.stno_frequency_ghz > 0.5 && report.stno_frequency_ghz < 40.0,
        "STNO frequency should be in microwave range (0.5-40 GHz), got {} GHz",
        report.stno_frequency_ghz
    );
    assert!(report.injection_locking_verified);
    assert!(
        report.throughput_steps_per_sec > 100_000.0,
        "Throughput must exceed 100,000 steps/sec, got {}",
        report.throughput_steps_per_sec
    );
}
