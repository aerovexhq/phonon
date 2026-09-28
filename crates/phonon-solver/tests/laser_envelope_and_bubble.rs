//! Integration tests for laser envelope dynamics and relativistic blowout bubble regime.

use phonon_models::wakefield::{BubbleRegime, LaserPulseParams, PlasmaChannelParams};

#[test]
fn test_laser_pulse_parameters() {
    let laser = LaserPulseParams::standard_tisapphire();
    assert_eq!(laser.a0, 3.0);
    assert!(laser.wavelength_m > 0.0);

    let omega0 = laser.angular_frequency_rad_per_s();
    assert!(omega0 > 2.0e15 && omega0 < 2.5e15);

    let e0 = laser.peak_electric_field_v_per_m();
    assert!(
        e0 > 1.0e12,
        "Peak electric field should exceed 1 TV/m, got {}",
        e0
    );

    let i0 = laser.peak_intensity_w_per_cm2();
    assert!(
        i0 > 1.0e19,
        "Peak intensity should exceed 10^19 W/cm^2, got {}",
        i0
    );

    let energy = laser.pulse_energy_joules();
    assert!(
        energy > 0.1 && energy < 100.0,
        "Laser pulse energy should be in Joule range, got {}",
        energy
    );
}

#[test]
fn test_plasma_channel_parameters() {
    let plasma = PlasmaChannelParams::standard_underdense();
    let omegap = plasma.plasma_frequency_rad_per_s();
    assert!(
        omegap > 5.0e13 && omegap < 1.0e14,
        "Plasma frequency should be ~7e13 rad/s, got {}",
        omegap
    );

    let lambda_p = plasma.plasma_wavelength_m();
    assert!(
        lambda_p > 20.0e-6 && lambda_p < 40.0e-6,
        "Plasma wavelength should be ~27 um, got {}",
        lambda_p
    );

    let kp = plasma.plasma_wavenumber_m1();
    assert!(kp > 2.0e5 && kp < 3.0e5);

    let e_wb = plasma.wavebreaking_field_v_per_m();
    assert!(
        e_wb > 1.0e11,
        "Wavebreaking field should exceed 100 GV/m, got {}",
        e_wb
    );

    let matched_depth = plasma.matched_guiding_depth_m3();
    assert!(matched_depth > 1.0e23);
}

#[test]
fn test_bubble_regime_cavitation_and_acceleration() {
    let laser = LaserPulseParams::standard_tisapphire();
    let plasma = PlasmaChannelParams::standard_underdense();
    let bubble = BubbleRegime::new(laser, plasma);

    assert!(
        bubble.is_bubble_regime(),
        "a0 = 3.0 must trigger bubble regime"
    );

    let rb = bubble.bubble_radius_m();
    assert!(
        rb > 10.0e-6 && rb < 25.0e-6,
        "Bubble radius should be ~15 um, got {}",
        rb
    );

    let e_peak = bubble.peak_accelerating_field_v_per_m();
    assert!(
        e_peak > 1.0e11,
        "Peak accelerating field should exceed 100 GV/m, got {}",
        e_peak
    );

    // Test longitudinal field profile
    let ez_front = bubble.longitudinal_field_v_per_m(0.0);
    assert_eq!(ez_front, 0.0);

    let ez_mid = bubble.longitudinal_field_v_per_m(-0.5 * rb);
    assert!(
        ez_mid < 0.0,
        "Longitudinal field in trailing bubble must be negative to accelerate electrons"
    );

    let ld = bubble.dephasing_length_m();
    assert!(
        ld > 0.5e-3,
        "Dephasing length should exceed 0.5 mm, got {}",
        ld
    );

    let max_energy = bubble.max_energy_gain_mev();
    assert!(
        max_energy > 100.0,
        "Max energy gain should exceed 100 MeV, got {}",
        max_energy
    );
}
