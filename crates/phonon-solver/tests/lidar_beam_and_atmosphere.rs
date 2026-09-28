//! Integration Test: LiDAR Laser Beam Physics, Gaussian Divergence & Atmospheric Mie Scattering

use phonon_core::constants::SPEED_OF_LIGHT;
use phonon_models::lidar::{AtmosphericCondition, FogType, LaserPulseConfig, WAVELENGTH_905_NM};

#[test]
fn test_laser_pulse_peak_power_and_gaussian_beam_divergence() {
    let laser_905 = LaserPulseConfig::new_automotive_905nm();
    // 15 uJ in 5 ns => P_peak = 15e-6 / 5e-9 = 3000 W
    assert_eq!(laser_905.peak_power_watts(), 3000.0);

    // Initial beam waist at R = 0 m:
    let w_0 = laser_905.beam_radius_at_range(0.0);
    assert!(
        (w_0 - 0.0015).abs() < 1e-6,
        "Waist radius at exit should be 1.5 mm, got {}",
        w_0
    );

    // Beam expansion at range R = 100 m:
    // With 1.5 mrad divergence, theta_half = 0.75 mrad => expansion ~ 100 * tan(0.00075) ~ 0.075 m = 7.5 cm
    let w_100 = laser_905.beam_radius_at_range(100.0);
    assert!(
        w_100 > 0.070 && w_100 < 0.080,
        "Expected ~7.5 cm beam radius at 100 m, got {}",
        w_100
    );

    // Spot area at 100 m:
    let spot_area_100 = laser_905.beam_spot_area_at_range(100.0);
    assert!(
        spot_area_100 > 0.015 && spot_area_100 < 0.020,
        "Expected ~0.0177 m^2 spot area, got {}",
        spot_area_100
    );
}

#[test]
fn test_lidar_range_equation_and_surface_reflectivity() {
    let laser = LaserPulseConfig::new_automotive_905nm();
    let alpha_clear = 0.000046; // Clear air

    // 1. Returned power at 10 m vs 20 m (approx 1 / R^2 scaling):
    let p_10 = laser.returned_optical_power(10.0, 0.80, 0.0, alpha_clear);
    let p_20 = laser.returned_optical_power(20.0, 0.80, 0.0, alpha_clear);
    let ratio = p_10 / p_20;
    assert!(
        (ratio - 4.0).abs() < 0.1,
        "Power ratio for 2x distance should be ~4.0 (inverse square law), got {}",
        ratio
    );

    // 2. Surface albedo effect:
    let p_retro = laser.returned_optical_power(10.0, 0.90, 0.0, alpha_clear); // Retroreflective sign
    let p_asphalt = laser.returned_optical_power(10.0, 0.10, 0.0, alpha_clear); // Dark asphalt
    assert!(
        (p_retro / p_asphalt - 9.0).abs() < 0.05,
        "Power should scale linearly with albedo: 0.9 / 0.1 = 9.0, got {}",
        p_retro / p_asphalt
    );

    // 3. Oblique incidence angle (Lambertian cos(theta)):
    let p_normal = laser.returned_optical_power(10.0, 0.50, 0.0, alpha_clear);
    let p_60deg = laser.returned_optical_power(10.0, 0.50, std::f64::consts::PI / 3.0, alpha_clear); // cos(60) = 0.5
    assert!(
        (p_normal / p_60deg - 2.0).abs() < 0.05,
        "Normal power should be 2x that of 60 deg incidence, got {}",
        p_normal / p_60deg
    );
}

#[test]
fn test_atmospheric_mie_scattering_extinction_and_fog() {
    let cond_clear = AtmosphericCondition::ClearAir;
    let cond_haze = AtmosphericCondition::Haze {
        visibility_m: 3000.0,
    };
    let cond_fog = AtmosphericCondition::Fog {
        visibility_m: 100.0,
        fog_type: FogType::RadiationFog,
    };

    let alpha_clear = cond_clear.extinction_coefficient_per_m(WAVELENGTH_905_NM);
    let alpha_haze = cond_haze.extinction_coefficient_per_m(WAVELENGTH_905_NM);
    let alpha_fog = cond_fog.extinction_coefficient_per_m(WAVELENGTH_905_NM);

    assert!(
        alpha_haze > alpha_clear * 10.0,
        "Haze extinction should exceed clear air by >10x"
    );
    assert!(
        alpha_fog > alpha_haze * 20.0,
        "Dense fog extinction should exceed haze by >20x"
    );

    // Two-way transmittance through 50 m in dense fog:
    // T = exp(-2 * alpha_fog * 50)
    let t_fog_50m = cond_fog.two_way_transmittance(50.0, WAVELENGTH_905_NM);
    assert!(
        t_fog_50m < 0.05,
        "Dense fog should severely attenuate two-way optical transmission: {}",
        t_fog_50m
    );

    // Rain extinction:
    let cond_rain = AtmosphericCondition::Rain { rate_mm_hr: 25.0 }; // Heavy rain 25 mm/h
    let alpha_rain = cond_rain.extinction_coefficient_per_m(WAVELENGTH_905_NM);
    assert!(
        alpha_rain > alpha_clear * 5.0,
        "Heavy rain should cause noticeable optical extinction"
    );
}

#[test]
fn test_wavelength_comparison_and_backscatter_clutter() {
    let laser_905 = LaserPulseConfig::new_automotive_905nm();
    let laser_1550 = LaserPulseConfig::new_aerospace_1550nm();

    // 1550 nm allows much higher pulse energy (80 uJ vs 15 uJ):
    assert!(laser_1550.peak_power_watts() > laser_905.peak_power_watts() * 5.0);

    // Fog backscatter power at 4 meters:
    let cond_fog = AtmosphericCondition::Fog {
        visibility_m: 50.0,
        fog_type: FogType::RadiationFog,
    };
    let bs_power = cond_fog.atmospheric_backscatter_power(&laser_905, 4.0);
    assert!(
        bs_power > 0.0,
        "Backscatter power in dense fog should be positive"
    );
    assert!(
        bs_power > laser_905.noise_equivalent_power_w,
        "Backscatter clutter in dense fog should exceed detector NEP: bs={}, nep={}",
        bs_power,
        laser_905.noise_equivalent_power_w
    );

    // Clear air should produce near-zero backscatter clutter:
    let cond_clear = AtmosphericCondition::ClearAir;
    let bs_clear = cond_clear.atmospheric_backscatter_power(&laser_905, 4.0);
    assert!(
        bs_clear < bs_power * 0.01,
        "Clear air backscatter should be negligible compared to fog"
    );
}

#[test]
fn test_time_of_flight_range_conversion() {
    let range = 45.0; // 45 meters
    let tof = LaserPulseConfig::time_of_flight_seconds(range);
    let expected_tof = (2.0 * 45.0) / SPEED_OF_LIGHT;
    assert!((tof - expected_tof).abs() < 1e-12);

    let converted_range = LaserPulseConfig::range_from_time_of_flight(tof);
    assert!((converted_range - range).abs() < 1e-9);
}
