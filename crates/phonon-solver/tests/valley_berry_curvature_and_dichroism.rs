use phonon_models::valleytronics::{TmdMaterialParams, ValleyLattice};

#[test]
fn test_valley_dispersion_and_berry_curvature_reversal() {
    let params = TmdMaterialParams::wse2_monolayer();
    let lattice = ValleyLattice::new(params);

    // Evaluate at small k near Dirac point
    let state_k = lattice.evaluate_state(1e7, 1e7, 1);
    let state_kp = lattice.evaluate_state(1e7, 1e7, -1);

    // Conduction band energy should be positive and equal in both valleys
    assert!(
        state_k.energy_conduction_ev > 0.8,
        "Conduction band minimum should be ~ Eg/2: got {}",
        state_k.energy_conduction_ev
    );
    assert!(
        (state_k.energy_conduction_ev - state_kp.energy_conduction_ev).abs() < 1e-6,
        "Valley energies must be degenerate under TRS at k = 0"
    );

    // Berry curvature must reverse sign between K and K'
    assert!(
        state_k.berry_curvature_m2 < 0.0,
        "K valley Berry curvature should be negative"
    );
    assert!(
        state_kp.berry_curvature_m2 > 0.0,
        "K' valley Berry curvature should be positive"
    );
    assert!(
        (state_k.berry_curvature_m2 + state_kp.berry_curvature_m2).abs() < 1e-25,
        "Berry curvature must be odd under time reversal: Omega_K + Omega_K' = 0"
    );

    // Orbital magnetic moment must also reverse sign
    assert!(
        state_k.orbital_moment_mu_b * state_kp.orbital_moment_mu_b < 0.0,
        "Orbital moments must have opposite signs between valleys"
    );
}

#[test]
fn test_valley_selective_circular_dichroism() {
    let params = TmdMaterialParams::mos2_monolayer();
    let lattice = ValleyLattice::new(params);

    // Exactly at Dirac point k = 0, dichroism should be 100% (+1 for K, -1 for K')
    let state_k = lattice.evaluate_state(0.0, 0.0, 1);
    let state_kp = lattice.evaluate_state(0.0, 0.0, -1);

    assert!(
        (state_k.circular_dichroism - 1.0).abs() < 1e-6,
        "K valley must exhibit 100% right-circular dichroism at band edge: got {}",
        state_k.circular_dichroism
    );
    assert!(
        (state_kp.circular_dichroism - (-1.0)).abs() < 1e-6,
        "K' valley must exhibit 100% left-circular dichroism at band edge: got {}",
        state_kp.circular_dichroism
    );
}
