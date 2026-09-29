//! Integration tests for interfacial high-Tc superconductivity in monolayer FeSe/STO:
//! forward-scattering electron-phonon enhancement, linear coupling scaling,
//! and strong-coupling gap ratio.

use phonon_models::interfacial_superconductivity::{InterfacialScParams, NematicOrderParams};
use phonon_solver::interfacial_superconductivity::InterfacialBdgSolver;

#[test]
fn test_interfacial_tc_exceeds_65_kelvin() {
    // STO optical phonon ~ 100 meV, forward-scattering q0 ~ 0.15 A^-1, lambda ~ 0.55, FeSe bulk Tc ~ 8 K
    let params = InterfacialScParams::new(100.0, 0.15, 0.55, 8.0);
    let tc = params.critical_temperature_kelvin();

    assert!(
        tc > 65.0,
        "Interfacial FeSe/STO critical temperature must exceed 65 K, got {:.2} K",
        tc
    );
    // Typical experimental values range between 65 K and 100 K
    assert!(
        tc < 120.0,
        "Critical temperature must remain physically reasonable (< 120 K), got {:.2} K",
        tc
    );
}

#[test]
fn test_forward_scattering_linear_lambda_scaling() {
    let base_params = InterfacialScParams::new(100.0, 0.15, 0.40, 8.0);
    let enhanced_params = InterfacialScParams::new(100.0, 0.15, 0.60, 8.0);

    let tc_base = base_params.critical_temperature_kelvin();
    let tc_enhanced = enhanced_params.critical_temperature_kelvin();

    assert!(
        tc_enhanced > tc_base,
        "Enhanced electron-phonon coupling must increase Tc: base={:.2} K, enhanced={:.2} K",
        tc_base,
        tc_enhanced
    );

    // Delta Tc scaling with Delta lambda: (tc_enhanced - 8) / (tc_base - 8) should be 0.60 / 0.40 = 1.5
    let delta_tc_base = tc_base - 8.0;
    let delta_tc_enhanced = tc_enhanced - 8.0;
    let ratio = delta_tc_enhanced / delta_tc_base;

    assert!(
        (ratio - 1.5).abs() < 1e-4,
        "Forward-scattering enhancement must scale linearly with lambda, expected 1.5, got {:.5}",
        ratio
    );
}

#[test]
fn test_strong_coupling_bcs_gap_ratio() {
    let params = InterfacialScParams::new(100.0, 0.15, 0.55, 8.0);
    let ratio = params.strong_coupling_ratio();

    assert!(
        ratio >= 3.8,
        "Strong-coupling gap ratio 2*Delta_0 / (k_B * Tc) must exceed weak-coupling BCS (3.53) and threshold 3.8, got {:.3}",
        ratio
    );
    assert!(
        (ratio - 4.3).abs() < 1e-2,
        "Strong-coupling ratio must match experimental value ~ 4.3, got {:.3}",
        ratio
    );

    let delta_0 = params.zero_temperature_gap_mev();
    assert!(
        delta_0 > 10.0 && delta_0 < 30.0,
        "Zero-temperature gap must be physically consistent (10-30 meV), got {:.2} meV",
        delta_0
    );
}

#[test]
fn test_bdg_solver_temperature_dependent_gap() {
    let sc_params = InterfacialScParams::new(100.0, 0.15, 0.55, 8.0);
    let nem_params = NematicOrderParams::default();
    let solver = InterfacialBdgSolver::new(sc_params, nem_params);

    let tc = solver.solve_critical_temperature();
    let gap_zero = solver.solve_temp_dependent_gap_mev(0.01);
    let gap_half_tc = solver.solve_temp_dependent_gap_mev(tc * 0.5);
    let gap_near_tc = solver.solve_temp_dependent_gap_mev(tc * 0.99);
    let gap_above_tc = solver.solve_temp_dependent_gap_mev(tc * 1.05);

    assert!(
        (gap_zero - solver.solve_zero_temp_gap_mev()).abs() < 1e-2,
        "Gap at near zero temp must match Delta_0"
    );
    assert!(
        gap_half_tc < gap_zero,
        "Gap must decrease with temperature: Delta(0)={:.2}, Delta(0.5 Tc)={:.2}",
        gap_zero,
        gap_half_tc
    );
    assert!(
        gap_near_tc < gap_half_tc,
        "Gap must sharply decrease near Tc: Delta(0.5 Tc)={:.2}, Delta(0.99 Tc)={:.2}",
        gap_half_tc,
        gap_near_tc
    );
    assert_eq!(
        gap_above_tc, 0.0,
        "Superconducting gap must strictly vanish for T >= Tc"
    );
}
