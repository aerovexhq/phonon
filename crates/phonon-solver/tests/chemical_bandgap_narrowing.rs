//! Integration Test: Heavy-Doping Bandgap Narrowing (Slotboom Model).
//!
//! Validates:
//! - Slotboom bandgap narrowing Delta E_g(N) across 1e15 cm^-3 to 1e20 cm^-3
//! - Bandgap reduction at heavy doping levels in Silicon
//! - Exponential expansion of effective intrinsic carrier density n_ie(T, N).

use phonon_core::constants::T_REF;
use phonon_models::chemistry::bandstructure::Bandstructure;

#[test]
fn test_slotboom_bandgap_narrowing() {
    let band = Bandstructure::silicon();
    let eg_300 = band.bandgap_ev(T_REF);
    let ni_300 = band.intrinsic_carrier_density(T_REF);

    println!(
        "Silicon at 300K: Intrinsic E_g = {:.4} eV, n_i = {:.3e} m^-3",
        eg_300, ni_300
    );
    assert!(
        (eg_300 - 1.124).abs() < 0.01,
        "Silicon bandgap at 300K must be ~1.124 eV"
    );

    // 1. Lightly doped: N = 1e15 cm^-3 (1e21 m^-3) -> Delta E_g ~ 0
    let n_light = 1.0e21;
    let bgn_light = band.bandgap_narrowing_ev(n_light);
    let nie_light = band.effective_intrinsic_carrier_density(T_REF, n_light);
    println!(
        "Light Doping (1e15 cm^-3): Delta E_g = {:.2} meV, n_ie = {:.3e} m^-3",
        bgn_light * 1000.0,
        nie_light
    );
    assert!(
        bgn_light < 1.0e-3,
        "Bandgap narrowing must be negligible (< 1 meV) at 1e15 cm^-3"
    );
    assert!(
        (nie_light - ni_300).abs() / ni_300 < 0.05,
        "n_ie must match n_i at light doping"
    );

    // 2. Moderately doped: N = 1e18 cm^-3 (1e24 m^-3) -> Delta E_g ~ 25 meV
    let n_med = 1.0e24;
    let bgn_med = band.bandgap_narrowing_ev(n_med);
    let nie_med = band.effective_intrinsic_carrier_density(T_REF, n_med);
    println!(
        "Moderate Doping (1e18 cm^-3): Delta E_g = {:.2} meV, n_ie = {:.3e} m^-3",
        bgn_med * 1000.0,
        nie_med
    );
    assert!(
        bgn_med > 0.015 && bgn_med < 0.050,
        "Delta E_g at 1e18 cm^-3 must be in 15-50 meV range"
    );
    assert!(nie_med > ni_300 * 1.2, "n_ie must be noticeably elevated");

    // 3. Heavily doped (Degenerate): N = 1e20 cm^-3 (1e26 m^-3) -> Delta E_g ~ 80 meV
    let n_heavy = 1.0e26;
    let bgn_heavy = band.bandgap_narrowing_ev(n_heavy);
    let nie_heavy = band.effective_intrinsic_carrier_density(T_REF, n_heavy);
    println!(
        "Heavy Doping (1e20 cm^-3): Delta E_g = {:.2} meV, n_ie = {:.3e} m^-3",
        bgn_heavy * 1000.0,
        nie_heavy
    );
    assert!(
        bgn_heavy > 0.060 && bgn_heavy < 0.120,
        "Delta E_g at 1e20 cm^-3 must be in 60-120 meV range"
    );
    assert!(
        nie_heavy > ni_300 * 3.0,
        "Effective intrinsic concentration must expand multiple times"
    );
}
