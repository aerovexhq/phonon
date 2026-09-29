//! Integration tests for quantum topological soliton microcavities,
//! octave-spanning comb spans, and dissipative acoustic pulse durations.

use phonon_models::topological_soliton_comb::TopologicalSolitonCombParams;
use phonon_solver::topological_soliton_comb::TopologicalSolitonCombSolver;

#[test]
fn test_default_soliton_comb_metrics() {
    let params = TopologicalSolitonCombParams::default();
    let solver = TopologicalSolitonCombSolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.comb_span_octaves >= 2.0,
        "Comb span {} octaves must be >= 2.0 octaves",
        metrics.comb_span_octaves
    );
    assert!(
        metrics.soliton_pulse_duration_ps <= 15.0,
        "Soliton duration {} ps must be <= 15.0 ps",
        metrics.soliton_pulse_duration_ps
    );
    assert!(
        metrics.comb_lines_count >= 100,
        "Comb lines {} must be >= 100",
        metrics.comb_lines_count
    );
}

#[test]
fn test_pump_power_span_scaling() {
    let p_low = TopologicalSolitonCombParams {
        pump_power_mw: 1.5,
        ..Default::default()
    };
    let p_high = TopologicalSolitonCombParams {
        pump_power_mw: 8.0,
        ..Default::default()
    };

    let solver_low = TopologicalSolitonCombSolver::new(p_low);
    let solver_high = TopologicalSolitonCombSolver::new(p_high);

    let span_low = solver_low.compute_comb_span_octaves();
    let span_high = solver_high.compute_comb_span_octaves();

    assert!(
        span_high > span_low,
        "Higher acoustic pump power should broaden frequency comb span (high={} vs low={})",
        span_high,
        span_low
    );
    assert!(span_low >= 2.0);
    assert!(span_high >= 2.0);
}

#[test]
fn test_dispersion_pulse_duration_scaling() {
    let p_low_d2 = TopologicalSolitonCombParams {
        dispersion_d2_khz: 20.0,
        ..Default::default()
    };
    let p_high_d2 = TopologicalSolitonCombParams {
        dispersion_d2_khz: 70.0,
        ..Default::default()
    };

    let solver_low = TopologicalSolitonCombSolver::new(p_low_d2);
    let solver_high = TopologicalSolitonCombSolver::new(p_high_d2);

    let tau_low = solver_low.compute_soliton_pulse_duration_ps();
    let tau_high = solver_high.compute_soliton_pulse_duration_ps();

    assert!(
        tau_high > tau_low,
        "Higher anomalous dispersion leads to wider soliton pulse durations"
    );
    assert!(tau_low <= 15.0);
    assert!(tau_high <= 15.0);
}
