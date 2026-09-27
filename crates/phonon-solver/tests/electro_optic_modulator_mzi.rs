//! Integration tests for electro-optic modulators (MZI and EAM).
//!
//! Validates:
//! - Mach-Zehnder Interferometer (MZI) transfer curve, V_pi, and extinction ratio (ER).
//! - Soref-Bennett carrier plasma dispersion model in silicon.
//! - Small-signal high-frequency 3-dB modulation bandwidth roll-off.
//! - Dynamic optical signal propagation under voltage modulation.

use phonon_core::{OpticalSignal, T_REF};
use phonon_models::photonic::ElectroOpticModulatorModel;

#[test]
fn test_mzi_modulation_curve_and_extinction() {
    let arm_len = 1.5e-3; // 1.5 mm arm length
    let v_pi = 3.5; // 3.5 V switching voltage
    let mzi = ElectroOpticModulatorModel::silicon_mzi_quadrature(arm_len, v_pi);

    let t_max = 10.0_f64.powf(-mzi.insertion_loss_db / 10.0);
    let er_linear = 10.0_f64.powf(mzi.extinction_ratio_db / 10.0);
    let t_min = t_max / er_linear;

    // At quadrature (V=0, phi0=pi/2): transmission is midpoint
    let t_quad = mzi.transmission_factor(0.0, T_REF);
    assert!((t_quad - 0.5 * (t_max + t_min)).abs() < 1e-4);

    // At V = -V_pi / 2: constructive interference -> peak transmission
    let t_on = mzi.transmission_factor(-v_pi / 2.0, T_REF);
    assert!((t_on - t_max).abs() < 1e-4);

    // At V = +V_pi / 2: destructive interference -> dark state
    let t_off = mzi.transmission_factor(v_pi / 2.0, T_REF);
    assert!((t_off - t_min).abs() < 1e-5);

    // Dynamic extinction ratio: ER_meas = 10 * log10(T_on / T_off)
    let er_meas = 10.0 * (t_on / t_off).log10();
    assert!((er_meas - mzi.extinction_ratio_db).abs() < 0.1);
}

#[test]
fn test_soref_bennett_plasma_dispersion_equations() {
    // Inject carrier plasma: 5e17 cm^-3 electrons, 5e17 cm^-3 holes
    let n_e = 5.0e17;
    let n_h = 5.0e17;
    let (dn, dalpha_per_m) = ElectroOpticModulatorModel::soref_bennett_plasma_dispersion(n_e, n_h);

    // Delta n must be negative (plasma effect reduces index):
    assert!(dn < 0.0);
    let expected_dn = -8.8e-22 * n_e - 8.5e-18 * n_h.powf(0.8);
    assert!((dn - expected_dn).abs() < 1e-7);

    // Free carrier absorption must increase attenuation:
    assert!(dalpha_per_m > 0.0);
    let expected_alpha_cm = 8.5e-18 * n_e + 6.0e-18 * n_h;
    let expected_alpha_m = expected_alpha_cm * 100.0;
    assert!((dalpha_per_m - expected_alpha_m).abs() < 1e-4);
}

#[test]
fn test_modulator_frequency_response_cutoff() {
    let mzi = ElectroOpticModulatorModel::silicon_mzi_quadrature(1.0e-3, 3.0);
    assert_eq!(mzi.bandwidth_3db_hz, 40.0e9);

    // DC response (f = 0): |H(0)| = 1.0
    let resp_dc = mzi.frequency_response(0.0);
    assert!((resp_dc - 1.0).abs() < 1e-6);

    // At f_3dB = 40 GHz: |H(f_3dB)| = 1 / sqrt(2) ~ 0.7071 (-3 dB)
    let resp_3db = mzi.frequency_response(40.0e9);
    assert!((resp_3db - 1.0 / std::f64::consts::SQRT_2).abs() < 1e-4);

    // At f = 80 GHz (2x f_3dB): |H(2 f_3dB)| = 1 / sqrt(1 + 4) = 1 / sqrt(5) ~ 0.4472
    let resp_80ghz = mzi.frequency_response(80.0e9);
    assert!((resp_80ghz - 1.0 / 5.0_f64.sqrt()).abs() < 1e-4);
}

#[test]
fn test_modulator_signal_propagation() {
    let mzi = ElectroOpticModulatorModel::silicon_mzi_quadrature(1.0e-3, 3.0);
    let sig_in = OpticalSignal::new(1.55e-6, 5.0e-3, 0.0); // 5 mW input CW laser

    // Drive with high state (-1.5 V) and low state (+1.5 V)
    let sig_high = mzi.propagate(&sig_in, -1.5, T_REF);
    let sig_low = mzi.propagate(&sig_in, 1.5, T_REF);

    assert!(sig_high.power_watts > sig_low.power_watts * 50.0);
    assert_eq!(sig_high.wavelength_m, 1.55e-6);
    assert_eq!(sig_low.wavelength_m, 1.55e-6);
}
