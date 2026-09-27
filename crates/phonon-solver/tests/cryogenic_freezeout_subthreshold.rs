//! Integration tests for cryogenic dopant freeze-out, Poole-Frenkel ionization,
//! and Cryo-CMOS steep subthreshold swing.

use phonon_models::cryogenic::{
    fermi_dirac_half, inverse_fermi_dirac_half, CryoMosfetModel, CryogenicFreezeoutModel,
};

#[test]
fn test_fermi_dirac_round_trip_and_degeneracy() {
    // Non-degenerate limit (eta << 0)
    let eta_neg = -3.0;
    let f_neg = fermi_dirac_half(eta_neg);
    let inv_eta_neg = inverse_fermi_dirac_half(f_neg);
    assert!(
        (eta_neg - inv_eta_neg).abs() < 1e-3,
        "Round trip failed for eta = {}: got {}",
        eta_neg,
        inv_eta_neg
    );

    // Highly degenerate limit (eta >> 0, Sommerfeld expansion)
    let eta_pos = 4.0;
    let f_pos = fermi_dirac_half(eta_pos);
    let inv_eta_pos = inverse_fermi_dirac_half(f_pos);
    assert!(
        (eta_pos - inv_eta_pos).abs() < 1e-3,
        "Round trip failed for eta = {}: got {}",
        eta_pos,
        inv_eta_pos
    );

    // Deep degenerate limit
    let eta_deep = 10.0;
    let f_deep = fermi_dirac_half(eta_deep);
    let inv_eta_deep = inverse_fermi_dirac_half(f_deep);
    assert!(
        (eta_deep - inv_eta_deep).abs() < 1e-3,
        "Round trip failed for eta = {}: got {}",
        eta_deep,
        inv_eta_deep
    );
}

#[test]
fn test_dopant_freezeout_temperature_dependence() {
    let model = CryogenicFreezeoutModel::default();

    // Effective density of states Nc(T) = Nc(300K) * (T / 300)^1.5
    let nc_300 = 2.8e25; // m^-3
    let nc_77 = nc_300 * (77.0 / 300.0f64).powf(1.5);
    let nc_4 = nc_300 * (4.2 / 300.0f64).powf(1.5);

    let (_, n_d_plus_300) = model.solve_equilibrium_electron_density(300.0, nc_300);
    let (_, n_d_plus_77) = model.solve_equilibrium_electron_density(77.0, nc_77);
    let (_, n_d_plus_4) = model.solve_equilibrium_electron_density(4.2, nc_4);

    let frac_300 = n_d_plus_300 / model.donor_concentration;
    let frac_77 = n_d_plus_77 / model.donor_concentration;
    let frac_4 = n_d_plus_4 / model.donor_concentration;

    // At 300K, almost fully ionized
    assert!(
        frac_300 > 0.85,
        "Ionization at 300K should be high: {}",
        frac_300
    );
    // At 77K, moderate freezeout
    assert!(
        frac_77 < frac_300,
        "77K ionization {} should be less than 300K {}",
        frac_77,
        frac_300
    );
    // At 4.2K, deep freezeout (virtually zero unassisted ionization)
    assert!(
        frac_4 < 0.05,
        "Deep freeze-out at 4.2K expected (< 5%): {}",
        frac_4
    );
}

#[test]
fn test_poole_frenkel_cryogenic_field_ionization() {
    let model = CryogenicFreezeoutModel::default();

    // At 4.2K with zero electric field -> deep freeze-out
    let frac_zero_field = model.ionized_donor_fraction(4.2, -5.0, 0.0);
    assert!(frac_zero_field < 1e-4);

    // Poole-Frenkel barrier lowering at 10^7 V/m (100 kV/cm)
    let delta_pf = model.poole_frenkel_barrier_lowering_ev(1e7);
    assert!(
        delta_pf > 0.03,
        "Poole-Frenkel barrier lowering {} eV should be significant",
        delta_pf
    );

    // Under high electric field (1.5 x 10^7 V/m), barrier is pulled down, liberating frozen carriers
    let frac_high_field = model.ionized_donor_fraction(4.2, -5.0, 1.5e7);
    assert!(
        frac_high_field > frac_zero_field * 1000.0,
        "High electric field must dramatically increase ionization via Poole-Frenkel effect: {} vs {}",
        frac_high_field,
        frac_zero_field
    );
}

#[test]
fn test_cryo_cmos_subthreshold_swing_and_threshold_shift() {
    let mosfet = CryoMosfetModel::default();

    // Subthreshold swing S(T)
    let s_300 = mosfet.subthreshold_swing_mv_dec(300.0);
    let s_77 = mosfet.subthreshold_swing_mv_dec(77.0);
    let s_4 = mosfet.subthreshold_swing_mv_dec(4.2);

    // Theoretical proportionality: S(77) / S(300) ~= 77 / 300 = 0.256
    let ratio = s_77 / s_300;
    assert!(
        (ratio - (77.0 / 300.0)).abs() < 0.05,
        "Subthreshold swing must scale linearly with T: ratio = {}",
        ratio
    );

    // At 4.2K, swing hits the interface trap floor S_min
    assert_eq!(s_4, mosfet.min_swing_mv_dec);

    // Threshold voltage increases monotonically as temperature decreases
    let vth_300 = mosfet.threshold_voltage(300.0);
    let vth_77 = mosfet.threshold_voltage(77.0);
    let vth_4 = mosfet.threshold_voltage(4.2);

    assert!(
        vth_77 > vth_300,
        "Vth at 77K {} should be higher than 300K {}",
        vth_77,
        vth_300
    );
    assert!(
        vth_4 > vth_77,
        "Vth at 4.2K {} should be higher than 77K {}",
        vth_4,
        vth_77
    );
    assert!((vth_4 - vth_300 - 0.2958).abs() < 0.01);
}
