//! Integration tests for PT-symmetric ultrasensitive acoustic sensors and non-reciprocal circulators.

use phonon_models::non_hermitian_pt_symmetry::PtSymmetryParams;
use phonon_solver::non_hermitian_pt_symmetry::PtSymmetrySolver;

#[test]
fn test_ep_sensitivity_enhancement_and_power() {
    let params = PtSymmetryParams::default();
    let solver = PtSymmetrySolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.sensitivity_enhancement_db >= 35.0,
        "Sensitivity enhancement {} dB must be >= 35.0 dB",
        metrics.sensitivity_enhancement_db
    );
    assert!(
        metrics.threshold_power_mw <= 2.0,
        "Threshold power {} mW must be <= 2.0 mW",
        metrics.threshold_power_mw
    );
    assert!(
        metrics.reverse_isolation_db >= 25.0,
        "Reverse isolation {} dB must be >= 25.0 dB",
        metrics.reverse_isolation_db
    );
}

#[test]
fn test_perturbation_scaling() {
    let perturbations_khz = [0.5, 2.0, 10.0, 30.0];
    for &df in &perturbations_khz {
        let params = PtSymmetryParams {
            perturbation_detuning_khz: df,
            ..Default::default()
        };
        let solver = PtSymmetrySolver::new(params);
        let enh = solver.compute_sensitivity_enhancement_db();

        assert!(
            enh >= 35.0,
            "Enhancement for df={} kHz was {} dB < 35.0 dB",
            df,
            enh
        );
    }
}
