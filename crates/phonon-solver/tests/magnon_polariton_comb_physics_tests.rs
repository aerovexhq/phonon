//! Automated unit and physical validation tests for cavity quantum magnon-polariton
//! frequency combs and non-linear squeezed halometry.

use phonon_models::cavity_magnon_polariton_comb::MagnonPolaritonCombParams;
use phonon_solver::cavity_magnon_polariton_comb::CavityMagnonPolaritonCombSolver;

#[test]
fn test_comb_generation_threshold_power() {
    let params = MagnonPolaritonCombParams::default();
    let solver = CavityMagnonPolaritonCombSolver::new(params);
    let p_th = solver.compute_comb_threshold_power_mw();

    // Target threshold pump power P_th <= 1.00 mW
    assert!(
        p_th <= 1.00,
        "Comb threshold power must be <= 1.00 mW, got {:.4} mW",
        p_th
    );
    assert!(
        p_th > 0.0,
        "Comb threshold power must be positive, got {:.4} mW",
        p_th
    );
}

#[test]
fn test_comb_octave_span() {
    let params = MagnonPolaritonCombParams::default();
    let solver = CavityMagnonPolaritonCombSolver::new(params);
    let span = solver.compute_comb_octave_span();

    // Target comb octave span >= 1.50 octaves
    assert!(
        span >= 1.50,
        "Comb octave span must be >= 1.50 octaves, got {:.3}",
        span
    );
}

#[test]
fn test_sub_shot_noise_halometer_improvement() {
    let params = MagnonPolaritonCombParams::default();
    let solver = CavityMagnonPolaritonCombSolver::new(params);
    let ssn_db = solver.compute_sub_shot_noise_improvement_db();

    // Target sub-shot-noise improvement >= 6.00 dB
    assert!(
        ssn_db >= 6.00,
        "Sub-shot-noise improvement must be >= 6.00 dB, got {:.2} dB",
        ssn_db
    );
}

#[test]
fn test_polariton_logarithmic_negativity_entanglement() {
    let params = MagnonPolaritonCombParams::default();
    let solver = CavityMagnonPolaritonCombSolver::new(params);
    let e_n = solver.compute_polariton_log_negativity();

    // Target continuous-variable polariton entanglement E_N >= 0.850
    assert!(
        e_n >= 0.850,
        "Polariton logarithmic negativity must be >= 0.850, got {:.4}",
        e_n
    );
}

#[test]
fn test_comb_teeth_count() {
    let params = MagnonPolaritonCombParams::default();
    let solver = CavityMagnonPolaritonCombSolver::new(params);
    let teeth = solver.compute_comb_teeth_count();

    // Target comb teeth count >= 40
    assert!(
        teeth >= 40,
        "Comb teeth count must be >= 40, got {}",
        teeth
    );
}

#[test]
fn test_full_comb_physical_compliance() {
    let params = MagnonPolaritonCombParams::default();
    let solver = CavityMagnonPolaritonCombSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.is_physically_compliant,
        "Default magnon-polariton comb system must satisfy all physical compliance thresholds: {:?}",
        metrics
    );
    assert!(metrics.comb_threshold_power_mw <= 1.00);
    assert!(metrics.comb_octave_span >= 1.50);
    assert!(metrics.sub_shot_noise_improvement_db >= 6.00);
    assert!(metrics.polariton_log_negativity >= 0.850);
    assert!(metrics.comb_teeth_count >= 40);
}
