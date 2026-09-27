//! Integration Tests for Physical Antenna Transduction & Electrodynamic Radiation

use approx::assert_relative_eq;
use phonon_core::constants::SPEED_OF_LIGHT;
use phonon_models::em::{DiscreteTransmitter, PhysicalAntenna, Vector3D};
use phonon_solver::em::AntennaElectrodynamicSolver;
use std::f64::consts::PI;

#[test]
fn test_half_wave_dipole_impedance_and_efficiency() {
    let freq_hz = 1.0e9; // 1 GHz (lambda = 0.3 m)
    let dipole = PhysicalAntenna::half_wave_dipole("Dipole_1GHz", freq_hz);

    // Resonant radiation resistance should be ~73.13 Ohms
    let r_rad = dipole.radiation_resistance_ohms();
    assert_relative_eq!(r_rad, 73.13, epsilon = 0.1);

    // Copper skin-effect ohmic loss resistance should be small (< 0.5 Ohms)
    let r_loss = dipole.loss_resistance_ohms();
    assert!(r_loss > 0.0 && r_loss < 0.5, "Ohmic loss was {r_loss} Ohms");

    // Radiation efficiency should exceed 99%
    let eff = dipole.radiation_efficiency();
    assert!(eff > 0.99 && eff <= 1.0, "Efficiency was {eff}");

    // Maximum directivity should be ~1.643 (2.156 dBi)
    assert_relative_eq!(dipole.max_directivity_linear(), 1.643, epsilon = 0.01);
    assert_relative_eq!(dipole.max_directivity_dbi(), 2.156, epsilon = 0.05);

    // Broadside (theta = pi/2) normalized pattern should be 1.0
    let f_broadside = dipole.normalized_power_pattern(PI * 0.5, 0.0);
    assert_relative_eq!(f_broadside, 1.0, epsilon = 1e-4);

    // Null at poles (theta = 0, pi)
    let f_pole = dipole.normalized_power_pattern(0.0, 0.0);
    assert_relative_eq!(f_pole, 0.0, epsilon = 1e-4);
}

#[test]
fn test_quarter_wave_monopole_characteristics() {
    let freq_hz = 433.92e6;
    let monopole = PhysicalAntenna::quarter_wave_monopole("Monopole_433MHz", freq_hz);

    // Radiation resistance over ground plane is half of dipole: ~36.56 Ohms
    let r_rad = monopole.radiation_resistance_ohms();
    assert_relative_eq!(r_rad, 36.56, epsilon = 0.1);

    // Maximum directivity is twice that of dipole: ~3.286 (5.166 dBi)
    assert_relative_eq!(monopole.max_directivity_linear(), 3.286, epsilon = 0.01);
    assert_relative_eq!(monopole.max_directivity_dbi(), 5.166, epsilon = 0.05);

    // Lower hemisphere (theta > pi/2) should be completely shielded by ground plane
    let f_ground = monopole.normalized_power_pattern(PI * 0.75, 0.0);
    assert_relative_eq!(f_ground, 0.0, epsilon = 1e-6);
}

#[test]
fn test_microstrip_patch_and_apertures() {
    let freq_hz = 2.45e9;
    let patch = PhysicalAntenna::microstrip_patch("Patch_2.45GHz", freq_hz, 4.4, 0.0016);

    // Patch should have directivity between 6.0 and 9.0 dBi
    let dbi = patch.max_directivity_dbi();
    assert!(
        (6.0..=9.0).contains(&dbi),
        "Patch directivity was {dbi} dBi"
    );

    // Broadside (+Z, theta = 0) radiation pattern is maximum
    let f_zenith = patch.normalized_power_pattern(0.0, 0.0);
    assert_relative_eq!(f_zenith, 1.0, epsilon = 1e-3);

    // Back hemisphere is zero
    let f_back = patch.normalized_power_pattern(PI * 0.8, 0.0);
    assert_relative_eq!(f_back, 0.0, epsilon = 1e-6);

    // Effective aperture area A_e = lambda^2 / (4 * pi) * G
    let lambda = SPEED_OF_LIGHT / freq_hz;
    let ae = patch.effective_aperture_m2();
    let expected_ae = (lambda * lambda / (4.0 * PI)) * patch.max_gain_linear();
    assert_relative_eq!(ae, expected_ae, epsilon = 1e-6);
}

#[test]
fn test_phased_array_beam_steering() {
    let freq_hz = 28.0e9; // 28 GHz mmWave
    let steer_theta = 30.0_f64.to_radians(); // Steer 30 deg off broadside
    let steer_phi = 0.0;

    let array = PhysicalAntenna::phased_array("Array_8x8", freq_hz, 8, 8, steer_theta, steer_phi);

    // Peak directivity for 64 elements should exceed 20 dBi
    let max_dbi = array.max_directivity_dbi();
    assert!(
        max_dbi > 20.0,
        "64-element array directivity was {max_dbi} dBi"
    );

    // Normalized pattern at steered angle should be maximum (~1.0 * cos(30 deg) = 0.866)
    let p_steered = array.normalized_power_pattern(steer_theta, steer_phi);
    assert!(p_steered > 0.8, "Steered pattern was {p_steered}");

    // At broadside (theta = 0), array pattern should be significantly lower due to phase steering
    let p_broadside = array.normalized_power_pattern(0.0, 0.0);
    assert!(
        p_broadside < 0.1,
        "Off-steer pattern at broadside was {p_broadside}"
    );
}

#[test]
fn test_electrodynamic_radiation_sphere_energy_conservation() {
    let transmitter = DiscreteTransmitter::wifi_2_4ghz_patch("WiFi_TX", Vector3D::ZERO);

    let solver = AntennaElectrodynamicSolver::new();
    let sphere_radius = 5.0; // 5 meters (far-field)

    // Evaluate over spherical grid with 36 polar steps and 72 azimuth steps
    let res = solver.evaluate_radiation_sphere(&transmitter, sphere_radius, 36, 72);

    assert_eq!(res.total_points, 36 * 72);
    assert!(res.theoretical_radiated_power_watts > 0.0);

    // Numerical integration of Poynting vector flux over closed sphere
    // should conserve power within numerical grid discretization tolerance (< 5%)
    assert!(
        res.power_conservation_error < 0.05,
        "Power conservation error was {:.2}%, integrated={:.4} W, theoretical={:.4} W",
        res.power_conservation_error * 100.0,
        res.integrated_power_watts,
        res.theoretical_radiated_power_watts,
    );

    // Far-field flags should all be true at 5 meters
    assert!(res.observation_points.iter().all(|pt| pt.is_far_field));
}
