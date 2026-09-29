//! Integration tests for non-reciprocal Josephson diode arrays,
//! second-harmonic current-phase relation, and nematic elastoresistance divergence.

use phonon_models::interfacial_superconductivity::{
    InterfacialScParams, JosephsonDiodeParams, NematicOrderParams,
};
use phonon_solver::interfacial_superconductivity::{
    InterfacialBdgSolver, JosephsonDiodeArraySolver,
};
use std::f64::consts::PI;

#[test]
fn test_josephson_diode_efficiency_exceeds_threshold() {
    // Second harmonic ratio r2 = 0.35
    let params = JosephsonDiodeParams::new(1.0e-5, 0.35, 0.1, 4);
    let solver = JosephsonDiodeArraySolver::new(params);
    let metrics = solver.solve_diode_metrics();

    assert!(
        metrics.diode_efficiency >= 0.20,
        "Diode efficiency must exceed 0.20 (20%), got {:.4}",
        metrics.diode_efficiency
    );
    assert!(
        metrics.diode_efficiency <= 0.60,
        "Diode efficiency must stay within realistic theoretical bound (<= 0.60), got {:.4}",
        metrics.diode_efficiency
    );
    assert!(
        metrics.rectification_ratio_db >= 3.0,
        "Rectification ratio must exceed 3.0 dB, got {:.2} dB",
        metrics.rectification_ratio_db
    );
    assert!(
        metrics.forward_critical_current_a > metrics.reverse_critical_current_a,
        "Forward critical current ({:.3e} A) must strictly exceed reverse ({:.3e} A)",
        metrics.forward_critical_current_a,
        metrics.reverse_critical_current_a
    );
}

#[test]
fn test_josephson_array_junction_scaling() {
    let params_1 = JosephsonDiodeParams::new(1.0e-5, 0.35, 0.1, 1);
    let params_4 = JosephsonDiodeParams::new(1.0e-5, 0.35, 0.1, 4);

    let solver_1 = JosephsonDiodeArraySolver::new(params_1);
    let solver_4 = JosephsonDiodeArraySolver::new(params_4);

    let m1 = solver_1.solve_diode_metrics();
    let m4 = solver_4.solve_diode_metrics();

    // Critical supercurrent scales with junction count N
    let ic_ratio = m4.forward_critical_current_a / m1.forward_critical_current_a;
    assert!(
        (ic_ratio - 4.0).abs() < 1e-4,
        "Forward critical current should scale by 4x for 4 junctions, got {:.4}",
        ic_ratio
    );

    // Diode efficiency is an intrinsic property of CPR harmonic distortion
    assert!(
        (m1.diode_efficiency - m4.diode_efficiency).abs() < 1e-5,
        "Diode efficiency must remain invariant under symmetric series scaling"
    );
    assert!(
        (m1.rectification_ratio_db - m4.rectification_ratio_db).abs() < 1e-4,
        "Rectification ratio in dB must remain invariant under symmetric series scaling"
    );
}

#[test]
fn test_nematic_curie_weiss_elastoresistance_divergence() {
    let nem_params = NematicOrderParams::new(0.40, 70.0, 0.35);
    let sc_params = InterfacialScParams::default();
    let solver = InterfacialBdgSolver::new(sc_params, nem_params);

    // Near nematic transition temperature T = 70 K, 2m66 diverges
    let m66_at_tnem = solver.solve_elastoresistance(70.0);
    let m66_at_100k = solver.solve_elastoresistance(100.0);
    let m66_at_150k = solver.solve_elastoresistance(150.0);

    assert!(
        m66_at_tnem > m66_at_100k,
        "Elastoresistance at T_nem ({:.1}) must exceed that at 100 K ({:.1})",
        m66_at_tnem,
        m66_at_100k
    );
    assert!(
        m66_at_100k > m66_at_150k,
        "Elastoresistance at 100 K ({:.1}) must exceed that at 150 K ({:.1})",
        m66_at_100k,
        m66_at_150k
    );
    assert!(
        m66_at_tnem > 100.0,
        "Peak nematic elastoresistance must exceed 100, got {:.1}",
        m66_at_tnem
    );
}

#[test]
fn test_nematic_anisotropic_superconducting_gap() {
    let nem_params = NematicOrderParams::new(0.50, 70.0, 0.40);
    let sc_params = InterfacialScParams::default();
    let solver = InterfacialBdgSolver::new(sc_params, nem_params);

    // Delta(theta) = Delta_0 [1 + eta * psi * cos(2 theta)]
    // At theta = 0, cos(2*0) = 1 (maximum)
    // At theta = pi/2, cos(pi) = -1 (minimum)
    let gap_0 = solver.solve_anisotropic_gap(0.0);
    let gap_pi_2 = solver.solve_anisotropic_gap(PI / 2.0);
    let gap_pi_4 = solver.solve_anisotropic_gap(PI / 4.0);
    let gap_base = solver.solve_zero_temp_gap_mev();

    assert!(
        gap_0 > gap_base,
        "Gap at theta=0 ({:.2} meV) must exceed isotropic base gap ({:.2} meV)",
        gap_0,
        gap_base
    );
    assert!(
        gap_pi_2 < gap_base,
        "Gap at theta=pi/2 ({:.2} meV) must be lower than isotropic base gap ({:.2} meV)",
        gap_pi_2,
        gap_base
    );
    assert!(
        (gap_pi_4 - gap_base).abs() < 1e-4,
        "Gap at theta=pi/4 ({:.2} meV) must match base gap ({:.2} meV) as cos(pi/2)=0",
        gap_pi_4,
        gap_base
    );

    let anisotropy_ratio = nem_params.gap_anisotropy_ratio();
    assert!(
        (gap_0 / gap_pi_2 - anisotropy_ratio).abs() < 1e-4,
        "Direct ratio {:.4} must match analytical gap anisotropy ratio {:.4}",
        gap_0 / gap_pi_2,
        anisotropy_ratio
    );
}
