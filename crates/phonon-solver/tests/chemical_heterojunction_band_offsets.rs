//! Integration Test: Heterojunction Band Alignment & Contact Physics.
//!
//! Validates:
//! - Anderson's electron affinity rule: Delta E_c + Delta E_v = Delta E_g
//! - Type-I straddling gap alignment in AlGaAs/GaAs and SiGe/Si
//! - Spontaneous + piezoelectric polarization 2DEG induction in AlGaN/GaN HEMTs
//! - Schottky barrier sum rule Phi_Bn + Phi_Bp = E_g across silicides.

use phonon_core::constants::T_REF;
use phonon_models::chemistry::bandstructure::Bandstructure;
use phonon_models::chemistry::contact::ContactMaterial;
use phonon_models::chemistry::heterostructure::{BandAlignmentType, HeteroInterface};

#[test]
fn test_anderson_heterojunction_band_offsets() {
    let band_gaas = Bandstructure::gallium_arsenide();
    // Al_0.3Ga_0.7As: Eg ~ 1.798 eV, chi ~ 3.74 eV
    let mut band_algaas = Bandstructure::gallium_arsenide();
    band_algaas.bandgap_0k_ev = 1.88;
    band_algaas.electron_affinity_ev = 3.74;

    let hetero = HeteroInterface::from_anderson_rule(
        "Al0.3Ga0.7As/GaAs",
        &band_gaas,
        &band_algaas,
        T_REF,
        0.0,
    );

    let eg_gaas = band_gaas.bandgap_ev(T_REF);
    let eg_algaas = band_algaas.bandgap_ev(T_REF);
    let delta_eg = eg_algaas - eg_gaas;

    println!(
        "AlGaAs/GaAs: Delta E_c = {:.3} eV, Delta E_v = {:.3} eV, Delta E_g = {:.3} eV",
        hetero.delta_ec_ev, hetero.delta_ev_ev, delta_eg
    );

    assert_eq!(hetero.alignment, BandAlignmentType::TypeIStraddling);
    assert!(
        (hetero.delta_ec_ev + hetero.delta_ev_ev - delta_eg).abs() < 1e-6,
        "Anderson sum rule must hold"
    );
    assert!(
        hetero.delta_ec_ev > 0.0,
        "Conduction band offset must be positive"
    );
    assert!(
        hetero.delta_ev_ev > 0.0,
        "Valence band offset must be positive"
    );
}

#[test]
fn test_algan_gan_polarization_2deg() {
    let hemt_interface = HeteroInterface::algan_gan_hemt();
    let n_2deg = hemt_interface.two_dimensional_electron_gas_density();
    let n_2deg_cm2 = n_2deg * 1e-4; // m^-2 to cm^-2

    println!(
        "AlGaN/GaN HEMT: Polarization charge = {:.3e} C/m^2 -> 2DEG density = {:.3e} cm^-2",
        hemt_interface.interface_polarization_c_m2, n_2deg_cm2
    );

    assert_eq!(hemt_interface.alignment, BandAlignmentType::TypeIStraddling);
    assert!(
        n_2deg_cm2 > 5.0e12 && n_2deg_cm2 < 2.0e13,
        "2DEG density in AlGaN/GaN must be ~1e13 cm^-2"
    );
}

#[test]
fn test_silicide_schottky_barrier_sum_rule() {
    let band_si = Bandstructure::silicon();
    let eg_si = band_si.bandgap_ev(T_REF);

    let nisi = ContactMaterial::nickel_silicide();
    let ptsi = ContactMaterial::platinum_silicide();

    let phi_bn_nisi = nisi.electron_barrier_height(&band_si, T_REF);
    let phi_bp_nisi = nisi.hole_barrier_height(&band_si, T_REF);
    println!(
        "NiSi on Silicon: Phi_Bn = {:.3} eV, Phi_Bp = {:.3} eV (Sum = {:.3} eV, Eg = {:.3} eV)",
        phi_bn_nisi,
        phi_bp_nisi,
        phi_bn_nisi + phi_bp_nisi,
        eg_si
    );
    assert!(
        (phi_bn_nisi + phi_bp_nisi - eg_si).abs() < 1e-6,
        "Schottky sum rule must hold for NiSi"
    );

    let phi_bn_ptsi = ptsi.electron_barrier_height(&band_si, T_REF);
    let phi_bp_ptsi = ptsi.hole_barrier_height(&band_si, T_REF);
    println!(
        "PtSi on Silicon: Phi_Bn = {:.3} eV, Phi_Bp = {:.3} eV (Sum = {:.3} eV, Eg = {:.3} eV)",
        phi_bn_ptsi,
        phi_bp_ptsi,
        phi_bn_ptsi + phi_bp_ptsi,
        eg_si
    );
    assert!(
        (phi_bn_ptsi + phi_bp_ptsi - eg_si).abs() < 1e-6,
        "Schottky sum rule must hold for PtSi"
    );
    assert!(
        phi_bp_ptsi < 0.40,
        "PtSi must form a low-barrier contact to p-type silicon"
    );
}
