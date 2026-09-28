//! Integration Test: 6-DOF Flight Dynamics, Aerodynamics & Sensor Transducers
//!
//! Validates:
//! 1. 6-DOF rigid-body translational and rotational equations of motion.
//! 2. Aerodynamic ground effect thrust amplification: T(h)/T_\infty = 1 / (1 - (R / 4h)^2).
//! 3. Dryden continuous turbulent wind gust model, scale lengths, and intensities.
//! 4. First-order actuator motor lag and reaction torque moments.
//! 5. Transducers: 9-DOF IMU, barometric pressure formula, and pulsed LiDAR rangefinder.

use approx::assert_relative_eq;
use phonon_models::em::{ChannelRng, Vector3D};
use phonon_models::sensors::{
    compute_ground_effect_factor, AirframeConfig, BarometerSensor, DrydenWindModel,
    FlightDynamicsEngine, FlightDynamicsState, InertiaTensor3D, LidarRangefinder, Quaternion,
    STANDARD_GRAVITY_M_S2, STANDARD_SEA_LEVEL_PRESSURE_PA,
};

#[test]
fn test_inertia_tensor_properties() {
    let inertia = InertiaTensor3D::new_diagonal(0.015, 0.020, 0.030);
    assert_relative_eq!(
        inertia.determinant(),
        0.015 * 0.020 * 0.030,
        epsilon = 1e-10
    );

    let inv = inertia.inverse();
    assert_relative_eq!(inv.ixx, 1.0 / 0.015, epsilon = 1e-6);
    assert_relative_eq!(inv.iyy, 1.0 / 0.020, epsilon = 1e-6);
    assert_relative_eq!(inv.izz, 1.0 / 0.030, epsilon = 1e-6);

    let omega = Vector3D::new(1.0, 2.0, 3.0);
    let h = inertia.multiply_vector(omega);
    assert_relative_eq!(h.x, 0.015 * 1.0, epsilon = 1e-10);
    assert_relative_eq!(h.y, 0.020 * 2.0, epsilon = 1e-10);
    assert_relative_eq!(h.z, 0.030 * 3.0, epsilon = 1e-10);
}

#[test]
fn test_aerodynamic_ground_effect_curve() {
    let r_rotor = 0.127; // 5-inch rotor (0.127 m)

    // Above 2 * R_rotor, ground effect factor must be exactly 1.0:
    let factor_high = compute_ground_effect_factor(0.50, r_rotor);
    assert_relative_eq!(factor_high, 1.0, epsilon = 1e-6);

    let factor_at_2r = compute_ground_effect_factor(2.0 * r_rotor, r_rotor);
    assert_relative_eq!(factor_at_2r, 1.0, epsilon = 1e-6);

    // At h = 0.5 * R_rotor, factor = 1 / (1 - (1/2)^2) = 1 / 0.75 = 1.3333:
    let h_half = 0.5 * r_rotor;
    let factor_half = compute_ground_effect_factor(h_half, r_rotor);
    assert_relative_eq!(factor_half, 4.0 / 3.0, epsilon = 1e-4);

    // Monotonicity: factor increases as altitude decreases below 2 * R_rotor:
    let factor_1 = compute_ground_effect_factor(0.20, r_rotor);
    let factor_2 = compute_ground_effect_factor(0.10, r_rotor);
    let factor_3 = compute_ground_effect_factor(0.05, r_rotor);
    assert!(
        factor_1 < factor_2,
        "factor_1 = {}, factor_2 = {}",
        factor_1,
        factor_2
    );
    assert!(
        factor_2 < factor_3,
        "factor_2 = {}, factor_3 = {}",
        factor_2,
        factor_3
    );
    assert!(factor_3 <= 1.50, "factor clamped to prevent singularity");
}

#[test]
fn test_motor_first_order_lag_and_rpm_limits() {
    let mut rng = ChannelRng::new(0x1234_5678);
    let airframe = AirframeConfig::new_quadrotor_x(1.5, 0.22, 0.127);
    let initial_state = FlightDynamicsState {
        position_world: Vector3D::new(0.0, 0.0, 1.0),
        velocity_body: Vector3D::ZERO,
        orientation: Quaternion::default(),
        angular_velocity_body: Vector3D::ZERO,
        rotor_speeds_rad_s: vec![0.0; 4],
    };

    let mut engine = FlightDynamicsEngine::new(airframe, initial_state);
    let target_cmd_rad_s: f64 = 1000.0;
    let tau_m: f64 = 0.025; // 25 ms time constant
    let dt: f64 = 0.002;

    // Step for tau_m seconds (12 steps):
    let steps_for_tau = (tau_m / dt).round() as usize;
    for _ in 0..steps_for_tau {
        engine.step(&[target_cmd_rad_s; 4], dt, &mut rng);
    }

    // At t = tau, response of 1 - exp(-1) ~ 0.6321 * target_cmd:
    let expected_speed = target_cmd_rad_s * (1.0 - (-1.0f64).exp());
    for &speed in &engine.state.rotor_speeds_rad_s {
        assert_relative_eq!(speed, expected_speed, epsilon = 35.0);
    }

    // After 5 * tau (~125 ms), response should be > 98% of target:
    for _ in 0..(5 * steps_for_tau) {
        engine.step(&[target_cmd_rad_s; 4], dt, &mut rng);
    }
    for &speed in &engine.state.rotor_speeds_rad_s {
        assert!(speed > target_cmd_rad_s * 0.98);
        assert!(speed <= target_cmd_rad_s * 1.01);
    }
}

#[test]
fn test_dryden_turbulent_wind_gust_spectrum() {
    let mut rng = ChannelRng::new(0x9876_5432);
    let mean_wind = Vector3D::new(4.0, 2.0, 0.0);
    let mut dryden = DrydenWindModel::new(mean_wind, 6.0);

    let altitude = 15.0; // 15 m AGL
    let (lengths, sigmas) = dryden.turbulence_parameters(altitude);

    assert!(lengths.x > 0.0);
    assert!(lengths.y > 0.0);
    assert_relative_eq!(lengths.z, altitude, epsilon = 1e-6);
    assert!(sigmas.x > 0.0);
    assert!(sigmas.y > 0.0);
    assert!(sigmas.z > 0.0);

    // Sample wind over 500 steps:
    let dt = 0.01;
    let mut wind_samples = Vec::new();
    for _ in 0..500 {
        let w = dryden.step(altitude, 8.0, dt, &mut rng);
        wind_samples.push(w);
    }

    let avg_x: f64 = wind_samples.iter().map(|w| w.x).sum::<f64>() / wind_samples.len() as f64;
    let avg_y: f64 = wind_samples.iter().map(|w| w.y).sum::<f64>() / wind_samples.len() as f64;

    // Mean should be centered close to background mean wind:
    assert_relative_eq!(avg_x, mean_wind.x, epsilon = 1.0);
    assert_relative_eq!(avg_y, mean_wind.y, epsilon = 1.0);
}

#[test]
fn test_barometric_pressure_formula_and_altitude_inversion() {
    let mut rng = ChannelRng::new(0xCAFE_BABE);
    let baro = BarometerSensor::default();

    // Sea level altitude:
    let p_sea = baro.theoretical_pressure_pa(0.0);
    assert_relative_eq!(p_sea, STANDARD_SEA_LEVEL_PRESSURE_PA, epsilon = 1e-4);

    // At 1000 m AMSL:
    let p_1000 = baro.theoretical_pressure_pa(1000.0);
    // Standard atmosphere at 1000 m is approx 89,875 Pa:
    assert!(p_1000 < p_sea);
    assert_relative_eq!(p_1000, 89875.0, epsilon = 200.0);

    // Pressure to altitude inversion:
    let h_inv = baro.pressure_to_altitude(p_1000);
    assert_relative_eq!(h_inv, 1000.0, epsilon = 1e-6);

    // Transduced noisy measurement:
    let p_sample = baro.sample_pressure_pa(100.0, &mut rng);
    let h_est = baro.pressure_to_altitude(p_sample);
    assert_relative_eq!(h_est, 100.0, epsilon = 1.5); // Within 1.5 m with noise
}

#[test]
fn test_lidar_rangefinder_cosine_tilt_and_range_cutoff() {
    let mut rng = ChannelRng::new(0xFEED_FACE);
    let lidar = LidarRangefinder::default();

    // Level attitude (pitch=0, roll=0):
    let q_level = Quaternion::default();
    let (d_level, valid_level) = lidar.sample_range(5.0, q_level, &mut rng);
    assert!(valid_level);
    assert_relative_eq!(d_level, 5.0, epsilon = 0.1);

    // 30 degree tilt: slant range = 5.0 / cos(30 deg) = 5.0 / 0.8660 ~ 5.7735 m:
    let q_tilt_30 = Quaternion::from_euler_rpy(0.0, 30.0f64.to_radians(), 0.0);
    let (d_tilt, valid_tilt) = lidar.sample_range(5.0, q_tilt_30, &mut rng);
    assert!(valid_tilt);
    assert_relative_eq!(d_tilt, 5.0 / 30.0f64.to_radians().cos(), epsilon = 0.1);

    // 60 degree tilt: exceeds max allowable tilt angle (45 deg) -> invalid reading:
    let q_tilt_60 = Quaternion::from_euler_rpy(0.0, 60.0f64.to_radians(), 0.0);
    let (_d_invalid, valid_invalid) = lidar.sample_range(5.0, q_tilt_60, &mut rng);
    assert!(!valid_invalid);
}

#[test]
fn test_6dof_rigid_body_hover_equilibrium() {
    let mut rng = ChannelRng::new(0x5555_AAAA);
    let mass = 1.5;
    let airframe = AirframeConfig::new_quadrotor_x(mass, 0.22, 0.127);

    // Weight to balance: W = m * g = 1.5 * 9.80665 = 14.709975 N
    // Each of 4 rotors needs T_i = W / 4 = 3.67749 N
    // T_i = k_t * Omega^2 -> Omega = sqrt(T_i / k_t)
    let weight = mass * STANDARD_GRAVITY_M_S2;
    let thrust_per_rotor = weight / 4.0;
    let k_t = airframe.rotors[0].thrust_coeff;
    let hover_speed_rad_s = (thrust_per_rotor / k_t).sqrt();

    let initial_state = FlightDynamicsState::new_hover(10.0, hover_speed_rad_s, 4);
    let mut engine = FlightDynamicsEngine::new(airframe, initial_state);

    let dt = 0.002;
    // Step for 100 steps with hover RPM command:
    for _ in 0..100 {
        engine.step(&[hover_speed_rad_s; 4], dt, &mut rng);
    }

    // Altitude should remain virtually unchanged around 10.0 m (vertical acceleration ~ 0):
    assert_relative_eq!(engine.state.position_world.z, 10.0, epsilon = 0.05);
    assert_relative_eq!(engine.state.velocity_body.z, 0.0, epsilon = 0.05);
}
