use phonon_models::moire::{CorrelatedInsulatorModel, MoireLattice, SuperconductingDomeModel};

#[test]
fn test_coulomb_interaction_and_mott_criterion() {
    let lattice = MoireLattice::magic_angle();
    let bandwidth_w_ev = 0.004; // 4 meV
    let epsilon_r = 5.0; // hBN encapsulation

    let model = CorrelatedInsulatorModel::new(lattice.moire_period_nm, epsilon_r, bandwidth_w_ev);

    // U = 1.44 eV*nm / (5.0 * 13.05 nm) \u{2248} 0.022 eV = 22 meV
    let u_mev = model.coulomb_u_ev * 1000.0;
    assert!(
        u_mev > 18.0 && u_mev < 26.0,
        "Expected U \u{2248} 22 meV, got {:.2} meV",
        u_mev
    );

    // Strongly correlated regime: U / W > 3.0
    assert!(
        model.correlation_ratio > 3.0,
        "Expected U/W > 3.0, got {:.2}",
        model.correlation_ratio
    );
}

#[test]
fn test_correlated_insulator_gaps_and_transport() {
    let lattice = MoireLattice::magic_angle();
    let bandwidth_w_ev = 0.0035; // 3.5 meV
    let model = CorrelatedInsulatorModel::new(lattice.moire_period_nm, 5.0, bandwidth_w_ev);

    // Correlated Mott gap at half-filling \u{03bd} = -2 (and +2):
    let gap_half_filling_mev = model.correlated_gap_ev(-2.0) * 1000.0;
    assert!(
        gap_half_filling_mev > 3.0 && gap_half_filling_mev < 8.0,
        "Expected \u{0394}_Mott \u{2248} 5 meV, got {:.2} meV",
        gap_half_filling_mev
    );

    // Charge neutrality \u{03bd} = 0 has zero Mott gap:
    assert_eq!(model.correlated_gap_ev(0.0), 0.0);

    // Arrhenius activation transport test:
    let r0 = 100.0; // ohms
    let r_cryo = model.evaluate_resistance_ohms(-2.0, 1.5, r0); // at 1.5 K
    let r_warm = model.evaluate_resistance_ohms(-2.0, 30.0, r0); // at 30 K

    // In the activated insulating regime, low-temperature resistance is exponentially higher:
    assert!(
        r_cryo > 10.0 * r_warm,
        "Expected insulating activation: R(1.5K)={:.1} \u{03a9} vs R(30K)={:.1} \u{03a9}",
        r_cryo,
        r_warm
    );
}

#[test]
fn test_superconducting_dome_and_critical_fields() {
    let sc_model = SuperconductingDomeModel::new_hole_doped_tbg();

    // Optimal doping at \u{03bd} = -2.25:
    let tc_optimal = sc_model.critical_temperature_k(-2.25);
    assert!(
        (tc_optimal - sc_model.tc_max_k).abs() < 0.05,
        "Expected peak Tc \u{2248} 1.7 K, got {:.3} K",
        tc_optimal
    );

    // Edge of the dome (\u{03bd} = -2.02 or -2.60) should have Tc = 0:
    assert_eq!(sc_model.critical_temperature_k(-2.02), 0.0);
    assert_eq!(sc_model.critical_temperature_k(-2.60), 0.0);

    // BCS Superconducting gap:
    let gap_sc_mev = sc_model.superconducting_gap_0_ev(-2.25) * 1000.0;
    assert!(
        gap_sc_mev > 0.20 && gap_sc_mev < 0.35,
        "Expected \u{0394}_SC \u{2248} 0.26 meV, got {:.3} meV",
        gap_sc_mev
    );

    // Upper critical field B_c2,\u{22a5} \u{2248} \u{03a6}_0 / (2\u{03c0} \u{03be}^2)
    let bc2_perp = sc_model.upper_critical_field_perp_tesla(-2.25);
    assert!(
        bc2_perp > 0.15 && bc2_perp < 0.40,
        "Expected B_c2,\u{22a5} \u{2248} 0.24 T, got {:.3} T",
        bc2_perp
    );

    // Pauli limit:
    let b_pauli = sc_model.pauli_limit_tesla(-2.25);
    assert!(b_pauli > 2.5 && b_pauli < 3.5);
}
