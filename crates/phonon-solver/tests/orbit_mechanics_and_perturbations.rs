#![allow(clippy::needless_range_loop)]
//! Integration tests for Phase 45: Spacecraft Orbital Mechanics & Perturbation Dynamics.

use phonon_models::em::Vector3D;
use phonon_models::space::orbit::{
    KeplerianElements, OrbitalPerturbationSolver, SpacecraftPhysicalProperties, ASTRONOMICAL_UNIT,
    MU_MOON, MU_SUN, R_EARTH,
};

#[test]
fn test_keplerian_to_cartesian_and_roundtrip() {
    let kep_initial = KeplerianElements {
        semi_major_axis_m: R_EARTH + 600_000.0,
        eccentricity: 0.05,
        inclination_rad: 51.6 * std::f64::consts::PI / 180.0,
        raan_rad: 45.0 * std::f64::consts::PI / 180.0,
        arg_perigee_rad: 30.0 * std::f64::consts::PI / 180.0,
        true_anomaly_rad: 60.0 * std::f64::consts::PI / 180.0,
    };

    let (r, v) = kep_initial.to_cartesian();
    assert!(r.norm() > R_EARTH, "Position must be above Earth surface");
    assert!(
        v.norm() > 7000.0 && v.norm() < 8500.0,
        "LEO velocity should be ~7.5 km/s"
    );

    let kep_recovered = KeplerianElements::from_cartesian(r, v);

    assert!(
        (kep_initial.semi_major_axis_m - kep_recovered.semi_major_axis_m).abs() < 1.0,
        "Semi-major axis mismatch: {} vs {}",
        kep_initial.semi_major_axis_m,
        kep_recovered.semi_major_axis_m
    );
    assert!(
        (kep_initial.eccentricity - kep_recovered.eccentricity).abs() < 1e-5,
        "Eccentricity mismatch: {} vs {}",
        kep_initial.eccentricity,
        kep_recovered.eccentricity
    );
    assert!(
        (kep_initial.inclination_rad - kep_recovered.inclination_rad).abs() < 1e-5,
        "Inclination mismatch: {} vs {}",
        kep_initial.inclination_rad,
        kep_recovered.inclination_rad
    );
    assert!(
        (kep_initial.raan_rad - kep_recovered.raan_rad).abs() < 1e-4,
        "RAAN mismatch"
    );
    assert!(
        (kep_initial.true_anomaly_rad - kep_recovered.true_anomaly_rad).abs() < 1e-4,
        "True anomaly mismatch"
    );
}

#[test]
fn test_geopotential_j2_j4_oblateness_acceleration() {
    let r_equatorial = Vector3D::new(R_EARTH + 500_000.0, 0.0, 0.0);
    let r_polar = Vector3D::new(0.0, 0.0, R_EARTH + 500_000.0);

    let a_two_body = OrbitalPerturbationSolver::two_body_accel(r_equatorial);
    let a_j2_j4_eq = OrbitalPerturbationSolver::geopotential_j2_j4_accel(r_equatorial);
    let a_j2_j4_pol = OrbitalPerturbationSolver::geopotential_j2_j4_accel(r_polar);

    assert!(a_two_body.norm() > 8.0, "LEO gravity should be ~8.4 m/s^2");
    assert!(
        a_j2_j4_eq.norm() > 0.005 && a_j2_j4_eq.norm() < 0.05,
        "J2 acceleration magnitude should be ~1e-2 m/s^2, got {}",
        a_j2_j4_eq.norm()
    );

    // J2 equatorial acceleration points radially inward; polar points radially outward due to oblateness:
    assert!(
        a_j2_j4_eq.x < 0.0,
        "J2 equatorial acceleration should be negative on X"
    );
    assert!(
        a_j2_j4_pol.z > 0.0,
        "J2 polar acceleration should be positive on Z"
    );
}

#[test]
fn test_third_body_lunar_and_solar_perturbation() {
    let r_sc = Vector3D::new(R_EARTH + 500_000.0, 0.0, 0.0);
    let r_moon = Vector3D::new(3.844e8, 0.0, 0.0);
    let r_sun = Vector3D::new(ASTRONOMICAL_UNIT, 0.0, 0.0);

    let a_moon = OrbitalPerturbationSolver::third_body_accel(r_sc, r_moon, MU_MOON);
    let a_sun = OrbitalPerturbationSolver::third_body_accel(r_sc, r_sun, MU_SUN);

    // Lunar and solar tide accelerations in LEO are ~ 1e-6 to 1e-7 m/s^2:
    assert!(
        a_moon.norm() > 1e-8 && a_moon.norm() < 1e-5,
        "Moon tidal acceleration in LEO: got {:e} m/s^2",
        a_moon.norm()
    );
    assert!(
        a_sun.norm() > 1e-8 && a_sun.norm() < 1e-5,
        "Sun tidal acceleration in LEO: got {:e} m/s^2",
        a_sun.norm()
    );
}

#[test]
fn test_atmospheric_drag_and_scale_height() {
    let props = SpacecraftPhysicalProperties::default();

    let r_200km = Vector3D::new(R_EARTH + 200_000.0, 0.0, 0.0);
    let r_500km = Vector3D::new(R_EARTH + 500_000.0, 0.0, 0.0);
    let v_orb = Vector3D::new(0.0, 7600.0, 0.0);

    let a_drag_200 = OrbitalPerturbationSolver::atmospheric_drag_accel(r_200km, v_orb, &props);
    let a_drag_500 = OrbitalPerturbationSolver::atmospheric_drag_accel(r_500km, v_orb, &props);

    // Drag deceleration must oppose velocity vector (negative Y):
    assert!(a_drag_200.y < 0.0, "Drag must oppose velocity");
    assert!(a_drag_500.y < 0.0, "Drag must oppose velocity");

    // Atmospheric density drops exponentially with altitude:
    assert!(
        a_drag_200.norm() > 100.0 * a_drag_500.norm(),
        "Drag at 200 km must vastly exceed drag at 500 km"
    );
}

#[test]
fn test_solar_radiation_pressure_and_cylindrical_eclipse() {
    let props = SpacecraftPhysicalProperties::default();
    let r_sun = Vector3D::new(ASTRONOMICAL_UNIT, 0.0, 0.0);

    // Dayside spacecraft (sunlit):
    let r_dayside = Vector3D::new(R_EARTH + 500_000.0, 0.0, 0.0);
    let nu_day = OrbitalPerturbationSolver::shadow_factor(r_dayside, r_sun);
    assert_eq!(nu_day, 1.0, "Dayside must have shadow factor 1.0 (sunlit)");

    let a_srp_day = OrbitalPerturbationSolver::solar_radiation_accel(r_dayside, r_sun, &props);
    assert!(
        a_srp_day.norm() > 1e-9 && a_srp_day.norm() < 1e-6,
        "SRP acceleration magnitude should be ~1e-7 m/s^2, got {:e}",
        a_srp_day.norm()
    );

    // Nightside spacecraft in umbra eclipse:
    let r_nightside = Vector3D::new(-(R_EARTH + 500_000.0), 0.0, 0.0);
    let nu_night = OrbitalPerturbationSolver::shadow_factor(r_nightside, r_sun);
    assert_eq!(nu_night, 0.0, "Nightside umbra must have shadow factor 0.0");

    let a_srp_night = OrbitalPerturbationSolver::solar_radiation_accel(r_nightside, r_sun, &props);
    assert_eq!(a_srp_night.norm(), 0.0, "SRP in eclipse must be zero");
}
