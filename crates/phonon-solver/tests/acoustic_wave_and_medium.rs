//! Integration Tests for Acoustic Wave Propagation, Medium Impedance & Doppler Shift

use phonon_models::acoustic::{
    compute_acoustic_doppler, evaluate_acoustic_field, speed_of_sound_in_air, AcousticMedium,
    AcousticObserver, AcousticSource, AcousticWall, MediumType,
};
use phonon_models::em::Vector3D;

#[test]
fn test_temperature_and_humidity_speed_of_sound() {
    // 0 deg C, dry air
    let c_0c = speed_of_sound_in_air(0.0, 0.0);
    assert!(
        (c_0c - 331.3).abs() < 0.5,
        "Expected ~331.3 m/s at 0 C, got {:.2}",
        c_0c
    );

    // 20 deg C, 50% RH
    let c_20c = speed_of_sound_in_air(20.0, 0.50);
    assert!(
        (c_20c - 344.0).abs() < 2.0,
        "Expected ~343-345 m/s at 20 C, got {:.2}",
        c_20c
    );

    // 100 deg C, dry air
    let c_100c = speed_of_sound_in_air(100.0, 0.0);
    assert!(
        (c_100c - 386.5).abs() < 2.0,
        "Expected ~386.5 m/s at 100 C, got {:.2}",
        c_100c
    );

    // Density check: ideal gas at 20 C, 1 atm
    let air = AcousticMedium::standard_air();
    assert!(
        (air.density - 1.204).abs() < 0.05,
        "Expected air density ~1.20 kg/m^3, got {:.3}",
        air.density
    );
}

#[test]
fn test_boundary_reflection_and_transmission_air_concrete() {
    let air = AcousticMedium::standard_air();
    let concrete = AcousticMedium::concrete();

    let (r_p, t_p, r_i, t_i) = air.boundary_coefficients(&concrete);

    // Concrete impedance Z ~ 7.82e6, Air Z ~ 413 => R_p ~ +0.99989, R_I ~ 0.99979, T_I ~ 0.00021
    assert!(r_p > 0.999, "Expected R_p > 0.999, got {:.5}", r_p);
    assert!(r_i > 0.999, "Expected R_I > 0.999, got {:.5}", r_i);
    assert!(t_i < 0.001, "Expected T_I < 0.001, got {:.5}", t_i);
    assert!(
        (r_i + t_i - 1.0).abs() < 1e-9,
        "Energy conservation R_I + T_I = 1 failed"
    );
    assert!(
        t_p > 1.99,
        "Expected boundary pressure doubling T_p ~ 2.0, got {:.4}",
        t_p
    );
}

#[test]
fn test_structural_wall_transmission_loss_mass_law() {
    let concrete = AcousticMedium::concrete();
    let wall = AcousticWall::new(
        Vector3D::new(5.0, 0.0, 1.5),
        Vector3D::new(1.0, 0.0, 0.0),
        0.15, // 15 cm concrete
        concrete,
        10.0,
        3.0,
    );

    // Mass density = 2300 kg/m^3 * 0.15 m = 345 kg/m^2
    assert!((wall.surface_mass_density - 345.0).abs() < 1.0);

    let tl_125 = wall.transmission_loss_db(125.0);
    let tl_500 = wall.transmission_loss_db(500.0);
    let tl_1000 = wall.transmission_loss_db(1000.0);
    let tl_4000 = wall.transmission_loss_db(4000.0);

    // Verify monotonic frequency dependence: 6 dB per octave increase
    assert!(tl_500 > tl_125);
    assert!(tl_1000 > tl_500);
    assert!(tl_4000 > tl_1000);

    // TL at 1000 Hz: 20 * log10(1000 * 345) - 47 ~ 110.7 - 47 = 63.7 dB
    assert!(
        (tl_1000 - 63.7).abs() < 2.0,
        "Expected ~63.7 dB TL at 1 kHz, got {:.2}",
        tl_1000
    );

    let amp_factor = wall.transmission_amplitude_factor(1000.0);
    assert!(
        amp_factor < 0.001,
        "Amplitude factor should be < 0.001, got {:.6}",
        amp_factor
    );
}

#[test]
fn test_acoustic_doppler_frequency_shift() {
    let air = AcousticMedium::standard_air();
    let f0 = 1000.0; // 1 kHz source

    let pos_s = Vector3D::new(0.0, 0.0, 0.0);
    let pos_o = Vector3D::new(100.0, 0.0, 0.0);
    let obs = AcousticObserver::new(pos_o, Vector3D::ZERO);

    // Stationary source:
    let src_stat = AcousticSource::new(pos_s, Vector3D::ZERO, f0, 1.0);
    let dop_stat = compute_acoustic_doppler(&src_stat, &obs, &air);
    assert!((dop_stat.observed_frequency_hz - f0).abs() < 1e-4);

    // Approaching source: v_s = +34.32 m/s (0.1 cs) towards observer
    let cs = air.speed_of_sound;
    let vs_app = Vector3D::new(0.1 * cs, 0.0, 0.0);
    let src_app = AcousticSource::new(pos_s, vs_app, f0, 1.0);
    let dop_app = compute_acoustic_doppler(&src_app, &obs, &air);
    // f_obs = f0 * cs / (cs - 0.1 cs) = f0 / 0.9 ~ 1.1111 f0
    let expected_app = f0 / 0.9;
    assert!(
        (dop_app.observed_frequency_hz - expected_app).abs() < 1.0,
        "Expected ~{:.1} Hz, got {:.1}",
        expected_app,
        dop_app.observed_frequency_hz
    );
    assert!(!dop_app.is_supersonic);

    // Receding source: v_s = -34.32 m/s away from observer
    let vs_rec = Vector3D::new(-0.1 * cs, 0.0, 0.0);
    let src_rec = AcousticSource::new(pos_s, vs_rec, f0, 1.0);
    let dop_rec = compute_acoustic_doppler(&src_rec, &obs, &air);
    // f_obs = f0 * cs / (cs + 0.1 cs) = f0 / 1.1 ~ 0.9091 f0
    let expected_rec = f0 / 1.1;
    assert!(
        (dop_rec.observed_frequency_hz - expected_rec).abs() < 1.0,
        "Expected ~{:.1} Hz, got {:.1}",
        expected_rec,
        dop_rec.observed_frequency_hz
    );
}

#[test]
fn test_vacuum_isolation_zero_acoustic_transmission() {
    let vac = AcousticMedium::vacuum();
    assert_eq!(vac.medium_type, MediumType::Vacuum);
    assert_eq!(vac.speed_of_sound, 0.0);
    assert_eq!(vac.acoustic_impedance, 0.0);

    let src = AcousticSource::new(Vector3D::new(0.0, 0.0, 0.0), Vector3D::ZERO, 1000.0, 100.0);
    let obs = AcousticObserver::new(Vector3D::new(5.0, 0.0, 0.0), Vector3D::ZERO);

    let field = evaluate_acoustic_field(&src, &obs, &vac, 0.05, 1.0);
    assert_eq!(field.instantaneous_pressure_pa, 0.0);
    assert_eq!(field.rms_pressure_pa, 0.0);
    assert_eq!(field.intensity_w_per_m2, 0.0);
    assert_eq!(field.spl_db, -100.0);
}
