//! Integration tests for Non-Equilibrium Green's Function (NEGF) quantum transport,
//! WKB dielectric tunneling, Kane band-to-band tunneling, and GAA nanosheet confinement.

use phonon_core::{ELEMENTARY_CHARGE, PLANCK_CONSTANT};
use phonon_models::quantum::{
    BandToBandTunnelingModel, DielectricTunnelingModel, GaaNanowireModel, QuantumChannel1D,
};

#[test]
fn test_negf_ballistic_transmission_and_landauer_current() {
    // 1D quantum channel of 10 nm length with 20 discretization grid points, m* = 0.26 m0
    let channel = QuantumChannel1D::new(20, 10e-9, 0.26);
    let t0 = channel.hopping_energy_ev();

    assert!(t0 > 0.0);

    // Evaluate transmission across energy spectrum
    // For energies below the subband bottom, transmission should be evanescent
    // For energies above, transmission opens up
    let t_below = channel.transmission(-0.5, 0.0);
    let t_above = channel.transmission(0.5, 0.0);

    assert!(
        t_below < 1e-3,
        "Transmission below band edge must be evanescent: {}",
        t_below
    );
    assert!(
        t_above > 0.1,
        "Transmission above band edge must be propagating: {}",
        t_above
    );

    // Compute Landauer terminal current at 300K under V_ds = 0.1 V
    let ids = channel.calculate_current(0.1, 300.0, 100);
    assert!(
        ids > 0.0,
        "Current must be positive under positive drain bias"
    );

    // Theoretical ballistic conductance quantum limit: G_0 * V_ds = (2e^2 / h) * 0.1 V
    let g0 = (2.0 * ELEMENTARY_CHARGE * ELEMENTARY_CHARGE) / PLANCK_CONSTANT;
    let i_max_single_mode = g0 * 0.1;
    assert!(
        ids <= i_max_single_mode * 2.0,
        "Ballistic current {} exceeds single-mode quantum upper bound {}",
        ids,
        i_max_single_mode
    );
}

#[test]
fn test_wkb_gate_dielectric_tunneling() {
    let model = DielectricTunnelingModel {
        tox: 1.0e-9,
        ..Default::default()
    };

    // Direct tunneling current at low voltage
    let j_direct = model.gate_leakage_current_density(0.5);
    assert!(j_direct > 0.0);

    // Compare with thinner oxide (0.8 nm) -> exponential increase in tunneling
    let mut thin_model = model;
    thin_model.tox = 0.8e-9;
    let j_thin = thin_model.gate_leakage_current_density(0.5);
    assert!(
        j_thin > j_direct * 5.0,
        "Thinner dielectric must exhibit exponentially higher direct tunneling: {} vs {}",
        j_thin,
        j_direct
    );

    // High electric field -> Fowler-Nordheim regime
    let t_fn = model.wkb_direct_transmission(3.5); // Greater than barrier height (3.15 eV)
    assert!(t_fn > 0.0);
}

#[test]
fn test_kane_band_to_band_tunneling() {
    let btbt = BandToBandTunnelingModel::default();

    let gen_low_field = btbt.generation_rate(1e6); // 10 kV/cm
    let gen_high_field = btbt.generation_rate(1.5e8); // 1.5 MV/cm

    assert!(gen_low_field < 1.0);
    assert!(
        gen_high_field > 1e15,
        "High field should induce massive BTBT carrier generation: {}",
        gen_high_field
    );
}

#[test]
fn test_gaa_nanosheet_quantum_confinement_subbands() {
    let nanosheet = GaaNanowireModel::default();
    let subbands = nanosheet.subband_energies_ev();

    // Verify subbands are non-empty and sorted monotonically
    assert!(!subbands.is_empty());
    for i in 0..(subbands.len() - 1) {
        assert!(
            subbands[i] <= subbands[i + 1],
            "Subband energies must be monotonically ascending: {} vs {}",
            subbands[i],
            subbands[i + 1]
        );
    }

    // Lowest subband E_11 should be tens to hundreds of meV due to quantum confinement
    let e_ground = subbands[0];
    assert!(
        e_ground > 0.02 && e_ground < 0.5,
        "Ground state subband energy {} eV is outside physical range (0.02 - 0.5 eV)",
        e_ground
    );

    // Multi-subband ballistic current increases with gate overdrive
    let i_sub_low = nanosheet.evaluate_current(0.2, 0.1, 300.0);
    let i_sub_high = nanosheet.evaluate_current(0.6, 0.1, 300.0);

    assert!(i_sub_high > i_sub_low * 2.0);
}
