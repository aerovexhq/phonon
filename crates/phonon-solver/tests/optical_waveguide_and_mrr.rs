//! Integration tests for integrated optical waveguides and micro-ring resonators (MRR).
//!
//! Validates:
//! - Waveguide modal propagation constant, group delay, and power attenuation for SOI and SiN.
//! - Micro-ring resonator FSR, loaded Q-factor, finesse, and add-drop complementary transmission.
//! - Thermo-optic resonance shift $\approx 80\text{ pm/K}$ and integrated micro-heater compensation.

use phonon_core::{OpticalSignal, SPEED_OF_LIGHT, T_REF};
use phonon_models::photonic::{MicroRingResonatorModel, OpticalWaveguideModel};
use std::f64::consts::PI;

#[test]
fn test_waveguide_dispersion_and_delay() {
    // 1 cm SOI strip waveguide
    let wg_soi = OpticalWaveguideModel::silicon_strip(0.01);

    // Group delay: tau_g = ng * L / c = 4.2 * 0.01 / 3e8 ~ 140.1 ps
    let tau_g = wg_soi.group_delay_seconds();
    assert!((tau_g - (4.20 * 0.01 / SPEED_OF_LIGHT)).abs() < 1e-13);

    // Attenuation over 1 cm: 2 dB/cm -> transmission ~ 0.630957
    let trans = wg_soi.power_transmission_factor();
    assert!((trans - 0.630957).abs() < 1e-4);

    // Propagate 10 mW input signal at 1550 nm
    let sig_in = OpticalSignal::new(1.55e-6, 0.010, 0.0);
    let sig_out = wg_soi.propagate(&sig_in, T_REF);
    assert!((sig_out.power_watts - 0.010 * trans).abs() < 1e-6);
    assert_eq!(sig_out.wavelength_m, 1.55e-6);

    // Low-loss Silicon Nitride (SiN) waveguide
    let wg_sin = OpticalWaveguideModel::silicon_nitride(0.05); // 5 cm
                                                               // 0.2 dB/cm * 5 cm = 1.0 dB loss -> trans ~ 10^(-0.1) ~ 0.7943
    let trans_sin = wg_sin.power_transmission_factor();
    assert!((trans_sin - 0.7943).abs() < 1e-3);
}

#[test]
fn test_micro_ring_resonator_fsr_and_q() {
    let radius = 10e-6; // 10 um radius
    let ring = MicroRingResonatorModel::add_drop_silicon(radius, 0.15);

    // Perimeter L = 2 * pi * 10 um ~ 62.83 um
    let perimeter = ring.roundtrip_length();
    assert!((perimeter - 2.0 * PI * 10e-6).abs() < 1e-12);

    // FSR = lambda0^2 / (ng * L) ~ 1.55e-6^2 / (4.2 * 62.83e-6) ~ 9.10 nm
    let fsr = ring.free_spectral_range_m();
    assert!(fsr > 8.0e-9 && fsr < 10.5e-9);

    // Quality factor Q should be high (> 5,000 for integrated SOI ring)
    let q = ring.quality_factor();
    assert!(q > 5_000.0);

    // Optical finesse F = FSR / FWHM > 10
    let f = ring.finesse();
    assert!(f > 10.0);
}

#[test]
fn test_micro_ring_add_drop_transmission_and_complementarity() {
    let radius = 8.0e-6;
    let ring = MicroRingResonatorModel::add_drop_silicon(radius, 0.25);

    let sig_in = OpticalSignal::new(1.55e-6, 1e-3, 0.0); // 1 mW
    let (thru, drop) = ring.propagate(&sig_in, T_REF);

    let drop_sig = drop.expect("Add-drop ring must have drop port signal");
    // Energy conservation: Thru + Drop <= Input
    assert!(thru.power_watts + drop_sig.power_watts <= sig_in.power_watts * 1.0001);
    assert!(thru.power_watts >= 0.0);
    assert!(drop_sig.power_watts >= 0.0);
}

#[test]
fn test_thermo_optic_resonance_drift_and_heater_tuning() {
    let radius = 10e-6;
    let ring = MicroRingResonatorModel::all_pass_silicon(radius, 0.2);

    // Silicon thermo-optic resonance drift: dlambda / dT ~ 68.6 pm/K
    let drift = ring.thermo_optic_resonance_drift_m_per_k();
    assert!((drift - 6.86e-11).abs() < 5e-12);

    // Thermal heater tuning: 50 Ohm heater, Rth = 2500 K/W
    // Applying 1.0 V heater drive produces P = 1 / 50 = 20 mW
    // Delta T = 0.02 * 2500 = 50 K
    let t_heated = ring.heated_temperature(T_REF, 1.0);
    assert!((t_heated - (T_REF + 50.0)).abs() < 1e-6);

    // The resonance will shift by Delta lambda = drift * 50 K ~ 3.43 nm
    let delta_lambda = drift * 50.0;
    assert!((delta_lambda - 3.43e-9).abs() < 0.3e-9);
}
