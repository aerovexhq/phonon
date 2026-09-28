//! Integration Test Suite:
//! Magic-Wavelength Optical Lattice Clocks, Relativistic Redshift & Chronometric Geodesy.

use phonon_core::constants::SPEED_OF_LIGHT;
use phonon_models::quantum::{
    AtomicSpecies, OpticalLatticeClock, SR88_CLOCK_LINEWIDTH_RAD_S, SR88_CLOCK_WAVELENGTH_METERS,
    SR88_MAGIC_WAVELENGTH_METERS, STANDARD_GRAVITY_M_S2,
};

#[test]
fn test_strontium88_magic_wavelength_and_differential_polarizability_cancellation() {
    let clock = OpticalLatticeClock::standard_sr88_lattice_clock();

    assert_eq!(clock.species, AtomicSpecies::Strontium88);
    assert_eq!(clock.clock_wavelength, SR88_CLOCK_WAVELENGTH_METERS);
    assert_eq!(clock.magic_wavelength, SR88_MAGIC_WAVELENGTH_METERS);

    // 1. Clock center frequency
    let nu_0 = clock.clock_frequency_hz();
    let expected_nu = SPEED_OF_LIGHT / SR88_CLOCK_WAVELENGTH_METERS;
    assert!((nu_0 - expected_nu).abs() < 1.0);
    assert!((nu_0 - 4.29228e14).abs() < 0.01e14); // ~429.23 THz

    // 2. Clock transition natural quality factor Q = nu_0 / DeltaNu_nat
    let natural_delta_nu = SR88_CLOCK_LINEWIDTH_RAD_S / (2.0 * std::f64::consts::PI);
    let expected_q = nu_0 / natural_delta_nu;
    assert!((clock.quality_factor - expected_q).abs() < 1e6);
    assert!(clock.quality_factor > 4.0e17);

    // 3. Exact cancellation at magic wavelength: Delta alpha(lambda_magic) = 0.0
    let d_alpha_magic = clock.differential_polarizability(clock.magic_wavelength);
    assert_eq!(d_alpha_magic, 0.0);

    // Off magic wavelength: differential Stark shift emerges
    let d_alpha_plus_1nm = clock.differential_polarizability(clock.magic_wavelength + 1.0e-9);
    assert!((d_alpha_plus_1nm - clock.differential_polarizability_slope).abs() < 1e-12);
    assert!(d_alpha_plus_1nm > 0.0);

    let d_alpha_minus_1nm = clock.differential_polarizability(clock.magic_wavelength - 1.0e-9);
    assert!((d_alpha_minus_1nm + clock.differential_polarizability_slope).abs() < 1e-12);
    assert!(d_alpha_minus_1nm < 0.0);
}

#[test]
fn test_clock_fractional_frequency_instability_and_quantum_projection_noise() {
    let clock = OpticalLatticeClock::standard_sr88_lattice_clock();

    // 1. Single-second instability under Quantum Projection Noise (QPN)
    let sigma_1s = clock.fractional_frequency_instability(1.0);
    assert!(sigma_1s <= 1.0e-18);
    assert!(sigma_1s > 1.0e-22);

    // 2. Allan deviation scaling: sigma_y(tau) proportional to 1 / sqrt(tau)
    let sigma_4s = clock.fractional_frequency_instability(4.0);
    assert!((sigma_4s - 0.5 * sigma_1s).abs() < 1e-22);

    let sigma_100s = clock.fractional_frequency_instability(100.0);
    assert!((sigma_100s - 0.1 * sigma_1s).abs() < 1e-22);
    assert!(sigma_100s < 1.0e-19);

    let sigma_10000s = clock.fractional_frequency_instability(10000.0);
    assert!((sigma_10000s - 0.01 * sigma_1s).abs() < 1e-22);
    assert!(sigma_10000s < 1.0e-20);

    // 3. Increasing atom count improves instability as 1 / sqrt(N)
    let mut high_n_clock = clock;
    high_n_clock.atom_count = 200_000.0; // 4x atom count
    let sigma_high_n = high_n_clock.fractional_frequency_instability(1.0);
    assert!((sigma_high_n - 0.5 * sigma_1s).abs() < 1e-22);
}

#[test]
fn test_relativistic_gravitational_redshift_elevation_mapping() {
    let clock = OpticalLatticeClock::standard_sr88_lattice_clock();
    let local_g = STANDARD_GRAVITY_M_S2;

    // 1. Exactly 1.0 cm height difference -> ~1.091e-18 fractional shift
    let delta_h_1cm = 0.01; // 1 cm in meters
    let frac_shift_1cm = clock.gravitational_redshift_fractional_shift(delta_h_1cm, local_g);
    let expected_frac = (local_g * delta_h_1cm) / (SPEED_OF_LIGHT * SPEED_OF_LIGHT);
    assert!((frac_shift_1cm - expected_frac).abs() < 1e-25);
    // Mandatory specification: 1.09 x 10^-18 per cm
    assert!((frac_shift_1cm - 1.091e-18).abs() < 1e-20);

    // 2. Frequency shift in absolute Hertz
    let delta_nu_hz = clock.gravitational_frequency_shift_hz(delta_h_1cm, local_g);
    let expected_hz = clock.clock_frequency_hz() * frac_shift_1cm;
    assert!((delta_nu_hz - expected_hz).abs() < 1e-12);
    // Delta nu ~ 4.29e14 * 1.091e-18 ~ 4.68e-4 Hz = 0.468 mHz
    assert!((delta_nu_hz - 4.68e-4).abs() < 0.1e-4);

    // 3. Geopotential difference Delta W = c^2 * (Delta nu / nu_0) = g * Delta h
    let delta_w = clock.geopotential_difference(frac_shift_1cm);
    let expected_w = local_g * delta_h_1cm;
    assert!((delta_w - expected_w).abs() < 1e-10);

    // 4. Exact height inversion across various elevations
    let heights_to_test = [0.005, 0.01, 0.10, 1.0, 50.0, 1000.0];
    for &h in &heights_to_test {
        let f_shift = clock.gravitational_redshift_fractional_shift(h, local_g);
        let h_recon = clock.height_from_fractional_shift(f_shift, local_g);
        assert!((h_recon - h).abs() < 1e-7);
    }

    // 5. Geodetic height resolution within 1 second of clock averaging
    let res_1s = clock.elevation_resolution_meters(1.0, local_g);
    // delta h < 1 cm (0.01 m)
    assert!(res_1s < 0.01);
    assert!(res_1s > 0.0);
}

#[test]
fn test_two_site_chronometric_leveling_comparison() {
    let clock_base = OpticalLatticeClock::standard_sr88_lattice_clock();
    let local_g = STANDARD_GRAVITY_M_S2;

    // Mountain station 1250.50 meters above base station
    let true_delta_h = 1250.50;
    let frac_shift = clock_base.gravitational_redshift_fractional_shift(true_delta_h, local_g);

    // Fractional frequency shift for 1250.50 m is ~1.364e-13
    assert!((frac_shift - 1.364e-13).abs() < 1e-15);

    // Optical frequency shift ~ 58.56 Hz
    let beat_note_hz = clock_base.gravitational_frequency_shift_hz(true_delta_h, local_g);
    assert!((beat_note_hz - 58.56).abs() < 0.1);

    // Reconstruct elevation from measured optical beat note
    let reconstructed_h = clock_base.height_from_fractional_shift(frac_shift, local_g);
    assert!((reconstructed_h - true_delta_h).abs() < 1e-6);

    // Geopotential number C = Delta W / g_0
    let delta_w = clock_base.geopotential_difference(frac_shift);
    let geopotential_number = delta_w / local_g;
    assert!((geopotential_number - true_delta_h).abs() < 1e-6);
}
