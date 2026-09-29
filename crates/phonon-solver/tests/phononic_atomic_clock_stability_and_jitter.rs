//! Integration tests for ultra-low jitter quantum phononic clocks,
//! beat-note SNR, and fractional frequency stability.

use phonon_models::topological_soliton_comb::TopologicalSolitonCombParams;
use phonon_solver::topological_soliton_comb::TopologicalSolitonCombSolver;

#[test]
fn test_jitter_and_stability_bounds() {
    let params = TopologicalSolitonCombParams::default();
    let solver = TopologicalSolitonCombSolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.timing_jitter_fs <= 10.0,
        "Timing jitter {} fs must be <= 10.0 fs",
        metrics.timing_jitter_fs
    );
    assert!(
        metrics.beat_note_snr_db >= 30.0,
        "Beat note SNR {} dB must be >= 30.0 dB",
        metrics.beat_note_snr_db
    );
    assert!(
        metrics.allan_deviation_floor <= 1.0e-12,
        "Allan deviation floor {} must be <= 1.0e-12",
        metrics.allan_deviation_floor
    );
}

#[test]
fn test_q_factor_jitter_reduction() {
    let p_low_q = TopologicalSolitonCombParams {
        loaded_q_factor: 5.0e5,
        ..Default::default()
    };
    let p_high_q = TopologicalSolitonCombParams {
        loaded_q_factor: 3.5e6,
        ..Default::default()
    };

    let solver_low = TopologicalSolitonCombSolver::new(p_low_q);
    let solver_high = TopologicalSolitonCombSolver::new(p_high_q);

    let jitter_low = solver_low.compute_timing_jitter_fs();
    let jitter_high = solver_high.compute_timing_jitter_fs();

    assert!(
        jitter_high < jitter_low,
        "Higher cavity Q must suppress quantum timing jitter (high_q={} vs low_q={})",
        jitter_high,
        jitter_low
    );
    assert!(jitter_high <= 10.0);
    assert!(jitter_low <= 10.0);
}

#[test]
fn test_q_factor_allan_deviation_scaling() {
    let p_low_q = TopologicalSolitonCombParams {
        loaded_q_factor: 6.0e5,
        ..Default::default()
    };
    let p_high_q = TopologicalSolitonCombParams {
        loaded_q_factor: 4.0e6,
        ..Default::default()
    };

    let solver_low = TopologicalSolitonCombSolver::new(p_low_q);
    let solver_high = TopologicalSolitonCombSolver::new(p_high_q);

    let allan_low = solver_low.compute_allan_deviation_floor();
    let allan_high = solver_high.compute_allan_deviation_floor();

    assert!(
        allan_high < allan_low,
        "Higher Q factor must improve fractional frequency stability floor"
    );
    assert!(allan_low <= 1.0e-12);
    assert!(allan_high <= 1.0e-12);
}
