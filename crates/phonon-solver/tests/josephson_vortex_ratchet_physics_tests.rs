//! Automated unit and physical validation tests for quantum acoustoelectric
//! Josephson vortex ratchets and topological soliton transport.

use phonon_models::josephson_vortex_ratchet::JosephsonVortexRatchetParams;
use phonon_solver::josephson_vortex_ratchet::JosephsonVortexRatchetSolver;

#[test]
fn test_fluxon_ratchet_rectification_efficiency() {
    let params = JosephsonVortexRatchetParams::default();
    let solver = JosephsonVortexRatchetSolver::new(params);
    let eta = solver.compute_ratchet_rectification_efficiency();

    // Target ratchet efficiency >= 92.0%
    assert!(
        eta >= 0.920,
        "Ratchet rectification efficiency must be >= 92.0%, got {:.2}%",
        eta * 100.0
    );
    assert!(
        eta <= 1.0,
        "Rectification efficiency cannot exceed unity, got {:.4}",
        eta
    );
}

#[test]
fn test_normalized_soliton_velocity() {
    let params = JosephsonVortexRatchetParams::default();
    let solver = JosephsonVortexRatchetSolver::new(params);
    let v_norm = solver.compute_normalized_soliton_velocity();

    // Target soliton velocity >= 0.850 c_sw
    assert!(
        v_norm >= 0.850,
        "Normalized soliton velocity must be >= 0.850, got {:.4}",
        v_norm
    );
    assert!(
        v_norm < 1.0,
        "Soliton velocity cannot exceed Swihart speed of light, got {:.4}",
        v_norm
    );
}

#[test]
fn test_acoustic_depinning_threshold_power() {
    let params = JosephsonVortexRatchetParams::default();
    let solver = JosephsonVortexRatchetSolver::new(params);
    let p_th = solver.compute_acoustic_threshold_power_uw();

    // Target acoustic depinning threshold power <= 0.50 uW
    assert!(
        p_th <= 0.50,
        "Acoustic threshold power must be <= 0.50 uW, got {:.4} uW",
        p_th
    );
    assert!(
        p_th > 0.0,
        "Threshold power must be positive, got {:.4} uW",
        p_th
    );
}

#[test]
fn test_phase_slip_shapiro_locking_precision() {
    let params = JosephsonVortexRatchetParams::default();
    let solver = JosephsonVortexRatchetSolver::new(params);
    let prec = solver.compute_phase_slip_locking_precision();

    // Target fractional locking precision <= 1.0e-9
    assert!(
        prec <= 1.0e-9,
        "Phase-slip locking precision must be <= 1.0e-9, got {:.3e}",
        prec
    );
    assert!(
        prec > 0.0,
        "Locking precision must be positive, got {:.3e}",
        prec
    );
}

#[test]
fn test_voltage_noise_spectral_density() {
    let params = JosephsonVortexRatchetParams::default();
    let solver = JosephsonVortexRatchetSolver::new(params);
    let s_v = solver.compute_voltage_noise_spectral_density_v2_hz();

    // Target voltage noise <= 1.0e-22 V^2/Hz
    assert!(
        s_v <= 1.0e-22,
        "Voltage noise spectral density must be <= 1.0e-22 V^2/Hz, got {:.3e}",
        s_v
    );
}

#[test]
fn test_full_ratchet_metrics_physical_compliance() {
    let params = JosephsonVortexRatchetParams::default();
    let solver = JosephsonVortexRatchetSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.is_physically_compliant,
        "Default Josephson vortex ratchet system must satisfy all physical compliance thresholds: {:?}",
        metrics
    );
    assert!(metrics.ratchet_rectification_efficiency >= 0.920);
    assert!(metrics.normalized_soliton_velocity >= 0.850);
    assert!(metrics.acoustic_threshold_power_uw <= 0.50);
    assert!(metrics.phase_slip_locking_precision <= 1.0e-9);
    assert!(metrics.voltage_noise_spectral_density_v2_hz <= 1.0e-22);
}
