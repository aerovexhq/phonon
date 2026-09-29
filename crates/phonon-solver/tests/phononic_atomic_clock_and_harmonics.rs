//! Integration tests for phononic atomic clocks, soliton timing jitter (<= 100 fs),
//! switching extinction contrast (>= 20.0 dB), and Allan deviation stability.

use phonon_models::phononic_microcomb::PhononicMicrocombParams;
use phonon_solver::phononic_microcomb::PhononicClockSolver;

#[test]
fn test_phononic_clock_timing_jitter_and_contrast() {
    let params = PhononicMicrocombParams {
        center_frequency_ghz: 3.5,
        free_spectral_range_mhz: 45.0,
        dispersion_d2_khz: 30.0,
        total_damping_khz: 40.0,
        kerr_nonlinearity_hz: 20.0,
        drive_power_mw: 10.0,
        normalized_detuning: 3.0,
    };
    let solver = PhononicClockSolver::new(params);
    let metrics = solver.solve_soliton_metrics();
    let microcomb_metrics = params.evaluate_metrics();

    // Soliton timing jitter must be <= 100 fs
    assert!(
        microcomb_metrics.timing_jitter_fs <= 100.0,
        "Timing jitter {:.2} fs must be <= 100 fs",
        microcomb_metrics.timing_jitter_fs
    );

    // Soliton switching extinction contrast must be >= 20.0 dB
    assert!(
        metrics.soliton_contrast_db >= 20.0,
        "Soliton contrast {:.2} dB must be >= 20.0 dB",
        metrics.soliton_contrast_db
    );

    // Total comb line count must be >= 50
    assert!(
        metrics.comb_line_count >= 50,
        "Comb line count {} must be >= 50",
        metrics.comb_line_count
    );

    // Fractional frequency instability floor must be <= 1e-11
    assert!(
        metrics.fractional_instability <= 1.0e-11,
        "Fractional instability {:.2e} must be <= 1e-11",
        metrics.fractional_instability
    );

    // Allan deviation scaling with tau: sigma(10s) < sigma(1s)
    let allan_1s = solver.solve_allan_deviation(1.0);
    let allan_10s = solver.solve_allan_deviation(10.0);
    assert!(
        allan_10s < allan_1s,
        "Allan deviation at 10s {:.2e} must be lower than at 1s {:.2e}",
        allan_10s,
        allan_1s
    );
}
