//! Integration Tests for WGS-84 Geodetic Coordinates, Space Channels & Multi-Source RF Noise.

use approx::assert_relative_eq;
use phonon_models::em::{
    EarthHorizon, GeodeticCoord, RfNoiseModel, SpaceNode, Vector3D, STANDARD_K_FACTOR,
    WGS84_A_METERS, WGS84_B_METERS,
};

#[test]
fn test_wgs84_geodetic_ecef_roundtrip() {
    // 1. Equator on Prime Meridian: (0, 0, 0m) -> ECEF (a, 0, 0)
    let eq = GeodeticCoord::new(0.0, 0.0, 0.0);
    let ecef_eq = eq.to_ecef();
    assert_relative_eq!(ecef_eq.x, WGS84_A_METERS, epsilon = 1e-3);
    assert_relative_eq!(ecef_eq.y, 0.0, epsilon = 1e-3);
    assert_relative_eq!(ecef_eq.z, 0.0, epsilon = 1e-3);

    let round_eq = ecef_eq.to_geodetic();
    assert_relative_eq!(round_eq.lat_deg, 0.0, epsilon = 1e-7);
    assert_relative_eq!(round_eq.lon_deg, 0.0, epsilon = 1e-7);
    assert_relative_eq!(round_eq.alt_m, 0.0, epsilon = 1e-3);

    // 2. North Pole: (90, 0, 100m) -> ECEF (0, 0, b + 100)
    let pole = GeodeticCoord::new(90.0, 0.0, 100.0);
    let ecef_pole = pole.to_ecef();
    assert_relative_eq!(ecef_pole.x, 0.0, epsilon = 1e-3);
    assert_relative_eq!(ecef_pole.y, 0.0, epsilon = 1e-3);
    assert_relative_eq!(ecef_pole.z, WGS84_B_METERS + 100.0, epsilon = 1e-3);

    let round_pole = ecef_pole.to_geodetic();
    assert_relative_eq!(round_pole.lat_deg, 90.0, epsilon = 1e-6);
    assert_relative_eq!(round_pole.alt_m, 100.0, epsilon = 1e-3);

    // 3. Arbitrary Station (London, UK): (51.5074 N, -0.1278 E, 50m)
    let london = GeodeticCoord::new(51.5074, -0.1278, 50.0);
    let ecef_london = london.to_ecef();
    let round_london = ecef_london.to_geodetic();
    assert_relative_eq!(round_london.lat_deg, london.lat_deg, epsilon = 1e-6);
    assert_relative_eq!(round_london.lon_deg, london.lon_deg, epsilon = 1e-6);
    assert_relative_eq!(round_london.alt_m, london.alt_m, epsilon = 1e-3);
}

#[test]
fn test_earth_curvature_horizon_and_los() {
    // 30m tower radio horizon with standard 4/3 refraction:
    // d_h = sqrt(2 * (4/3) * 6371000 * 30) ~ 22.576 km
    let d_h30 = EarthHorizon::radio_horizon_distance(30.0, STANDARD_K_FACTOR);
    assert_relative_eq!(d_h30 / 1000.0, 22.576, epsilon = 0.05);

    // Max LOS between 30m tower and 1.5m mobile receiver:
    // d_max = d_h(30m) + d_h(1.5m) ~ 22.58 + 5.05 ~ 27.62 km
    let d_max = EarthHorizon::max_line_of_sight_distance(30.0, 1.5, STANDARD_K_FACTOR);
    assert_relative_eq!(d_max / 1000.0, 27.62, epsilon = 0.1);

    // Two stations on opposite sides of Earth (Equator vs Antimeridian)
    let st1 = GeodeticCoord::new(0.0, 0.0, 10.0).to_ecef();
    let st2 = GeodeticCoord::new(0.0, 180.0, 10.0).to_ecef();
    assert!(!EarthHorizon::has_ellipsoid_line_of_sight(
        &st1,
        &st2,
        STANDARD_K_FACTOR
    ));

    // Two nearby stations 5 km apart
    let near1 = GeodeticCoord::new(0.0, 0.0, 10.0).to_ecef();
    let near2 = GeodeticCoord::new(0.045, 0.0, 10.0).to_ecef(); // ~5 km north
    assert!(EarthHorizon::has_ellipsoid_line_of_sight(
        &near1,
        &near2,
        STANDARD_K_FACTOR
    ));
}

#[test]
fn test_space_relativistic_doppler_and_ionosphere() {
    let gs_ecef = GeodeticCoord::new(0.0, 0.0, 0.0).to_ecef();
    let gs = SpaceNode::new("GroundStation", gs_ecef, Vector3D::ZERO);

    // LEO satellite at 500 km altitude directly approaching at 7500 m/s
    let sat_ecef = GeodeticCoord::new(0.0, 5.0, 500_000.0).to_ecef();
    let disp = (gs_ecef.to_vector3d() - sat_ecef.to_vector3d()).normalize();
    let vel_approach = disp.scale(7500.0); // Exactly approaching along LOS

    let sat = SpaceNode::new("SatApproaching", sat_ecef, vel_approach);

    let tx_freq = 10.0e9; // 10 GHz
    let doppler = sat.evaluate_doppler(&gs, tx_freq);

    // Approaching velocity -> positive blueshift:
    // Delta f ~ f * (v / c) = 10^10 * (7500 / 299792458) ~ +250.17 kHz
    assert!(doppler.doppler_shift_hz > 0.0);
    assert_relative_eq!(doppler.doppler_shift_hz / 1e3, 250.17, epsilon = 0.5);

    // Ionospheric group delay for 50 TECU at GPS L1 (1.57542 GHz)
    let delay_ns = SpaceNode::ionospheric_delay_ns(50.0, 1.57542e9);
    // Delta tau = (40.3 * 50 * 10^16) / (3e8 * (1.57542e9)^2) ~ 27.05 ns (~8.1 m)
    assert_relative_eq!(delay_ns, 27.05, epsilon = 0.2);
}

#[test]
fn test_rf_noise_and_solar_burst_outage() {
    let mut noise = RfNoiseModel::new(20.0e6, 4.0); // 20 MHz, 4 dB NF
    noise.solar_flux_index_sfu = 80.0; // Quiet Sun

    // Receiver noise temp: T_rx = 290 * (10^0.4 - 1) ~ 438.45 K
    let t_rx = noise.receiver_noise_temperature();
    assert_relative_eq!(t_rx, 438.45, epsilon = 1.0);

    // Thermal noise floor with quiet Sun at 2.4 GHz (~ -99.0 dBm for 20 MHz and 4 dB NF)
    let p_n_dbm = noise.thermal_noise_power_dbm(30.0, 2.4e9);
    assert_relative_eq!(p_n_dbm, -99.03, epsilon = 0.5);

    // Active solar burst: F10.7 surges to 1500 sfu with antenna pointing near Sun
    noise.solar_flux_index_sfu = 1500.0;
    noise.sun_offset_angle_deg = 2.0;
    noise.antenna_beamwidth_deg = 15.0;

    let t_sun_contrib = noise.solar_noise_temperature_contribution(2.4e9);
    assert!(
        t_sun_contrib > 100.0,
        "Solar burst should contribute significant antenna temperature, got {t_sun_contrib} K"
    );

    let p_n_burst_dbm = noise.thermal_noise_power_dbm(30.0, 2.4e9);
    assert!(
        p_n_burst_dbm > p_n_dbm,
        "Noise floor should increase during solar burst"
    );
}

#[test]
fn test_itur_gaseous_and_rain_attenuation() {
    // 60 GHz oxygen absorption resonance should be very high (> 5 dB/km)
    let att_60ghz = RfNoiseModel::itu_r_p676_gaseous_attenuation_db(60.0e9, 1.0);
    assert!(
        att_60ghz > 10.0,
        "60 GHz oxygen absorption should exceed 10 dB/km, got {att_60ghz}"
    );

    // 2.4 GHz attenuation over 1 km is nearly negligible (< 0.05 dB/km)
    let att_2ghz = RfNoiseModel::itu_r_p676_gaseous_attenuation_db(2.4e9, 1.0);
    assert!(att_2ghz < 0.05);

    // Rain attenuation at 30 GHz under heavy rain (25 mm/hr) over 2 km
    let noise = RfNoiseModel {
        rain_rate_mm_hr: 25.0,
        rain_path_length_km: 2.0,
        ..Default::default()
    };

    let rain_loss = noise.itu_r_p838_rain_attenuation_db(30.0e9);
    assert!(
        rain_loss > 1.0,
        "Rain loss at 30 GHz should be noticeable, got {rain_loss} dB"
    );
}
