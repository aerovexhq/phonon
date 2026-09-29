use phonon_models::fractional_chern::{
    AnyonQudit, FractionalFilling, MoireLatticeParams, ELECTRON_CHARGE_C,
};
use phonon_solver::fractional_chern::{AnyonicTeleportationSolver, ManyBodyFciSolver};

#[test]
fn test_fractional_quasiparticle_charge_and_degeneracy() {
    let params = MoireLatticeParams::twisted_mote2_125();

    // 1/3 Laughlin State
    let solver_one_third = ManyBodyFciSolver::new(params, FractionalFilling::OneThird, 1);
    let res_13 = solver_one_third.solve();

    assert_eq!(res_13.ground_state_degeneracy, 3);
    assert!((res_13.filling_fraction - 1.0 / 3.0).abs() < 1e-6);
    assert!((res_13.many_body_chern_number - 1.0 / 3.0).abs() < 1e-6);

    let e_star_13 = solver_one_third.fci_state.fractional_charge_c;
    assert!((e_star_13 - ELECTRON_CHARGE_C / 3.0).abs() < 1e-30);

    // Topological gap must be robust (> 1.0 meV)
    assert!(
        res_13.spectral_gap_ev >= 0.001,
        "Spectral gap must exceed 1.0 meV: got {} meV",
        res_13.spectral_gap_ev * 1e3
    );

    // Torus splitting must be exponentially smaller than the gap
    assert!(
        res_13.ground_state_splitting_ev < 0.1 * res_13.spectral_gap_ev,
        "Ground-state manifold splitting must be exponentially suppressed"
    );

    // 1/5 Higher-Order Laughlin State
    let solver_one_fifth = ManyBodyFciSolver::new(params, FractionalFilling::OneFifth, 1);
    let res_15 = solver_one_fifth.solve();

    assert_eq!(res_15.ground_state_degeneracy, 5);
    assert!((res_15.filling_fraction - 1.0 / 5.0).abs() < 1e-6);
    assert!((res_15.many_body_chern_number - 1.0 / 5.0).abs() < 1e-6);

    let e_star_15 = solver_one_fifth.fci_state.fractional_charge_c;
    assert!((e_star_15 - ELECTRON_CHARGE_C / 5.0).abs() < 1e-30);
    assert!(res_15.spectral_gap_ev >= 0.0005);
}

#[test]
fn test_anyonic_state_teleportation_fidelity() {
    let params = MoireLatticeParams::twisted_mote2_125();

    // Teleportation of a 3-level anyonic qutrit (nu = 1/3)
    let solver = AnyonicTeleportationSolver::new(
        params,
        FractionalFilling::OneThird,
        350.0, // 350 nm channel
        0.05,  // 50 mK
    );

    // Prepare arbitrary superposition: 1/sqrt(6) (|0> + 2|1> + i|2>)
    let state_raw = vec![(1.0, 0.0), (2.0, 0.0), (0.0, 1.0)];
    let input_state = AnyonQudit::new(state_raw);

    // Test across various Bell measurement outcomes (m, n)
    for m in 0..3 {
        for n in 0..3 {
            let res = solver.execute_teleportation(&input_state, m, n);
            assert!(
                res.state_fidelity >= 0.99,
                "Anyonic teleportation fidelity must be >= 99%: got {} for BSM ({}, {})",
                res.state_fidelity,
                m,
                n
            );
        }
    }
}
