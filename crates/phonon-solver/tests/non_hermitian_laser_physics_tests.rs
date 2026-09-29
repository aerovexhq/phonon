//! Automated unit and physical validation tests for non-Hermitian Floquet
//! topological acoustic lasers and skin-effect metamaterials.

use phonon_models::non_hermitian_acoustic_laser::NonHermitianLaserParams;
use phonon_solver::non_hermitian_acoustic_laser::NonHermitianAcousticLaserSolver;

#[test]
fn test_skin_mode_localization_ratio() {
    let params = NonHermitianLaserParams::default();
    let solver = NonHermitianAcousticLaserSolver::new(params);
    let skin = solver.compute_skin_mode_localization_ratio();

    // Target skin mode localization >= 92.0%
    assert!(
        skin >= 0.920,
        "Skin mode localization ratio must be >= 92.0%, got {:.2}%",
        skin * 100.0
    );
    assert!(
        skin <= 1.0,
        "Skin mode localization ratio cannot exceed unity, got {:.4}",
        skin
    );
}

#[test]
fn test_laser_single_mode_suppression_ratio() {
    let params = NonHermitianLaserParams::default();
    let solver = NonHermitianAcousticLaserSolver::new(params);
    let smsr = solver.compute_laser_smsr_db();

    // Target SMSR >= 35.0 dB
    assert!(
        smsr >= 35.0,
        "Laser single-mode suppression ratio must be >= 35.0 dB, got {:.2} dB",
        smsr
    );
}

#[test]
fn test_laser_output_power() {
    let params = NonHermitianLaserParams::default();
    let solver = NonHermitianAcousticLaserSolver::new(params);
    let power = solver.compute_laser_output_power_mw();

    // Target laser output power >= 15.0 mW
    assert!(
        power >= 15.0,
        "Laser output power must be >= 15.0 mW, got {:.2} mW",
        power
    );
}

#[test]
fn test_non_reciprocal_acoustic_isolation() {
    let params = NonHermitianLaserParams::default();
    let solver = NonHermitianAcousticLaserSolver::new(params);
    let iso = solver.compute_non_reciprocal_isolation_db();

    // Target non-reciprocal isolation >= 30.0 dB
    assert!(
        iso >= 30.0,
        "Non-reciprocal isolation must be >= 30.0 dB, got {:.2} dB",
        iso
    );
}

#[test]
fn test_topological_corner_mode_fidelity() {
    let params = NonHermitianLaserParams::default();
    let solver = NonHermitianAcousticLaserSolver::new(params);
    let fidelity = solver.compute_corner_mode_fidelity();

    // Target corner mode fidelity >= 0.950
    assert!(
        fidelity >= 0.950,
        "Corner mode fidelity must be >= 0.950, got {:.4}",
        fidelity
    );
    assert!(
        fidelity <= 1.0,
        "Fidelity cannot exceed unity, got {:.4}",
        fidelity
    );
}

#[test]
fn test_full_laser_metrics_physical_compliance() {
    let params = NonHermitianLaserParams::default();
    let solver = NonHermitianAcousticLaserSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.is_physically_compliant,
        "Default non-Hermitian acoustic laser system must satisfy all physical compliance thresholds: {:?}",
        metrics
    );
    assert!(metrics.skin_mode_localization_ratio >= 0.920);
    assert!(metrics.laser_smsr_db >= 35.0);
    assert!(metrics.laser_output_power_mw >= 15.0);
    assert!(metrics.non_reciprocal_isolation_db >= 30.0);
    assert!(metrics.corner_mode_fidelity >= 0.950);
}
