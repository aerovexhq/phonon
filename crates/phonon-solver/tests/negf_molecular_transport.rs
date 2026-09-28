//! Integration tests for NEGF molecular transport, Kondo resonance,
//! and spin-dependent tunneling.

use phonon_models::stno::NegfMolecularJunctionParams;
use phonon_solver::stno::NegfMolecularSolver;

#[test]
fn test_negf_molecular_junction_parameters() {
    let adatom = NegfMolecularJunctionParams::standard_magnetic_adatom();
    assert_eq!(adatom.molecular_level_ev, -0.18);
    assert_eq!(adatom.on_site_coulomb_u_ev, 0.8);

    let tk = adatom.kondo_temperature_k();
    assert!(
        tk > 5.0 && tk < 100.0,
        "Kondo temperature should be ~15-50 K, got {}",
        tk
    );

    let tmr = adatom.tunnel_magnetoresistance_ratio();
    assert!(
        (tmr - 0.3809).abs() < 0.01,
        "TMR ratio for P=0.4 should be ~38%, got {}",
        tmr
    );
}

#[test]
fn test_kondo_zeeman_splitting_under_magnetic_field() {
    let adatom = NegfMolecularJunctionParams::standard_magnetic_adatom();

    // B = 0: single zero-bias Kondo peak
    let res_b0 = NegfMolecularSolver::solve(&adatom, 0.001, 0.0);
    assert!(res_b0.zero_bias_conductance_2e2_over_h > 0.5);
    assert_eq!(res_b0.zeeman_splitting_ev, 0.0);

    // B = 2 Tesla: Zeeman splitting of Kondo peak = 2 g mu_B B ~ 0.46 meV
    let res_b2 = NegfMolecularSolver::solve(&adatom, 0.001, 2.0);
    let delta_z_mev = res_b2.zeeman_splitting_ev * 1.0e3;
    assert!(
        (delta_z_mev - 0.4637).abs() < 0.02,
        "Zeeman splitting at 2 T should be ~0.464 meV, got {} meV",
        delta_z_mev
    );
    assert!(res_b2.current_microamps.abs() > 0.0);
}
