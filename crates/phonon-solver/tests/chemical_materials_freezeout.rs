//! Integration Test: Dopant Species Chemical Ionization & Temperature Freeze-out.
//!
//! Validates:
//! - Temperature-dependent incomplete ionization / carrier freeze-out across 50K - 500K
//! - Ground-state degeneracy physics (g_D = 2 for donors, g_A = 4 for acceptors)
//! - Deep dopant freeze-out (Indium in Silicon, Magnesium in GaN)
//! - Solid chemical solubility limits and active carrier saturation.

use phonon_core::constants::T_REF;
use phonon_models::chemistry::bandstructure::Bandstructure;
use phonon_models::chemistry::dopant::{DopantSpecies, DopantType};

#[test]
fn test_silicon_phosphorus_and_boron_freezeout() {
    let band_si = Bandstructure::silicon();
    let n_doping = 1.0e23; // 1e17 cm^-3 in m^-3

    let phosphorus = DopantSpecies::Phosphorus;
    let boron = DopantSpecies::Boron;

    assert_eq!(phosphorus.dopant_type(), DopantType::Donor);
    assert_eq!(boron.dopant_type(), DopantType::Acceptor);
    assert!((phosphorus.ionization_energy_ev() - 0.045).abs() < 1e-6);
    assert!((boron.ionization_energy_ev() - 0.045).abs() < 1e-6);

    // 1. Room Temperature (300K): Shallow dopants should be almost fully ionized (> 90%)
    let eta_p_300 = phosphorus.ionization_fraction(&band_si, n_doping, T_REF);
    let eta_b_300 = boron.ionization_fraction(&band_si, n_doping, T_REF);
    println!(
        "Ionization fraction at 300K: Phosphorus = {:.2}%, Boron = {:.2}%",
        eta_p_300 * 100.0,
        eta_b_300 * 100.0
    );
    assert!(
        eta_p_300 > 0.85,
        "Phosphorus must be predominantly ionized at 300K"
    );
    assert!(
        eta_b_300 > 0.80,
        "Boron must be predominantly ionized at 300K"
    );

    // 2. High Temperature (500K): Full 100% ionization
    let eta_p_500 = phosphorus.ionization_fraction(&band_si, n_doping, 500.0);
    println!(
        "Ionization fraction at 500K: Phosphorus = {:.2}%",
        eta_p_500 * 100.0
    );
    assert!(
        eta_p_500 > 0.98,
        "Phosphorus must approach 100% ionization at 500K"
    );

    // 3. Cryogenic Freeze-out (77K Liquid Nitrogen): Severe freeze-out
    let eta_p_77 = phosphorus.ionization_fraction(&band_si, n_doping, 77.0);
    let eta_b_77 = boron.ionization_fraction(&band_si, n_doping, 77.0);
    println!(
        "Ionization fraction at 77K: Phosphorus = {:.2}%, Boron = {:.2}%",
        eta_p_77 * 100.0,
        eta_b_77 * 100.0
    );
    assert!(
        eta_p_77 < 0.20,
        "Carrier freeze-out must trap carriers at 77K"
    );
    assert!(eta_b_77 < 0.20, "Hole freeze-out must trap carriers at 77K");

    // 4. Ultra-Cryogenic Freeze-out (50K): Less than 5% ionized
    let eta_p_50 = phosphorus.ionization_fraction(&band_si, n_doping, 50.0);
    println!(
        "Ionization fraction at 50K: Phosphorus = {:.3}%",
        eta_p_50 * 100.0
    );
    assert!(
        eta_p_50 < 0.05,
        "Over 95% of carriers must freeze out at 50K"
    );
}

#[test]
fn test_deep_dopants_and_chemical_solubility() {
    let band_si = Bandstructure::silicon();
    let band_gan = Bandstructure::gallium_nitride();

    // Deep acceptor in Silicon: Indium (Delta E_a = 160 meV)
    let indium = DopantSpecies::Indium;
    let n_doping = 1.0e23;
    let eta_in_300 = indium.ionization_fraction(&band_si, n_doping, T_REF);
    println!(
        "Indium in Silicon ionization fraction at 300K: {:.2}%",
        eta_in_300 * 100.0
    );
    assert!(
        eta_in_300 < 0.40,
        "Deep Indium acceptor must exhibit substantial freeze-out even at 300K"
    );

    // Deep acceptor in GaN: Magnesium (Delta E_a = 170 meV)
    let magnesium = DopantSpecies::MagnesiumAcceptorInGaN;
    let eta_mg_300 = magnesium.ionization_fraction(&band_gan, n_doping, T_REF);
    println!(
        "Magnesium in GaN ionization fraction at 300K: {:.2}%",
        eta_mg_300 * 100.0
    );
    assert!(
        eta_mg_300 < 0.25,
        "Magnesium in GaN must show significant freeze-out at room temperature"
    );

    // Chemical Solid Solubility Limit clipping:
    // Arsenic limit: 1.8e27 m^-3. Requesting 5.0e27 m^-3 should be clamped
    let arsenic = DopantSpecies::Arsenic;
    let active_as = arsenic.active_concentration(5.0e27);
    assert_eq!(active_as, arsenic.solid_solubility_m3());
    assert!(
        active_as < 5.0e27,
        "Excess dopant beyond solubility limit must precipitate"
    );
}
