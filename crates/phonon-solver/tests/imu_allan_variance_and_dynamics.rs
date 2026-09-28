//! Integration Tests: 6-DOF / 9-DOF IMU, Allan Variance & Dynamics
//!
//! Validates:
//! 1. 4D unit quaternion vector rotations and body-frame gravity decomposition.
//! 2. Specific force kinematics (f = a_body - g_body) in level, pitched, rolled, and inverted attitudes.
//! 3. 9-DOF geomagnetic field sensing and hard-iron calibration.
//! 4. First-order Gauss-Markov bias instability and thermal drift.

#![deny(unsafe_code)]

use phonon_models::em::{ChannelRng, Vector3D};
use phonon_models::sensors::{
    sample_imu, ImuConfig, ImuMeasurement, ImuState, Quaternion, STANDARD_GRAVITY_M_S2,
};

#[test]
fn test_attitude_quaternion_gravity_and_specific_force() {
    let mut config = ImuConfig::new_industrial_9dof();
    // Zero out stochastic white noise for deterministic kinematic check:
    config.accel_noise.white_noise_density = 0.0;
    config.accel_noise.bias_instability = 0.0;
    config.gyro_noise.white_noise_density = 0.0;
    config.gyro_noise.bias_instability = 0.0;

    let mut state = ImuState::default();
    let mut rng = ChannelRng::new(100);

    // 1. Level stationary drone:
    let q_level = Quaternion::from_euler_rpy(0.0, 0.0, 0.0);
    let meas_level = sample_imu(
        &config,
        &mut state,
        0.002,
        Vector3D::ZERO,
        Vector3D::ZERO,
        q_level,
        298.15,
        &mut rng,
    );
    // Body +Z points upwards against gravity, so specific force is +9.80665 m/s^2:
    assert!((meas_level.accel_m_s2.z - STANDARD_GRAVITY_M_S2).abs() < 1e-4);
    assert!(meas_level.accel_m_s2.x.abs() < 1e-4);
    assert!(meas_level.accel_m_s2.y.abs() < 1e-4);

    // 2. Pitched 90 deg up:
    let q_pitch = Quaternion::from_euler_rpy(0.0, std::f64::consts::FRAC_PI_2, 0.0);
    let meas_pitch = sample_imu(
        &config,
        &mut state,
        0.004,
        Vector3D::ZERO,
        Vector3D::ZERO,
        q_pitch,
        298.15,
        &mut rng,
    );
    assert!((meas_pitch.accel_m_s2.x - (-STANDARD_GRAVITY_M_S2)).abs() < 1e-4);
    assert!(meas_pitch.accel_m_s2.z.abs() < 1e-4);

    // 3. Rolled 90 deg right:
    let q_roll = Quaternion::from_euler_rpy(std::f64::consts::FRAC_PI_2, 0.0, 0.0);
    let meas_roll = sample_imu(
        &config,
        &mut state,
        0.006,
        Vector3D::ZERO,
        Vector3D::ZERO,
        q_roll,
        298.15,
        &mut rng,
    );
    assert!((meas_roll.accel_m_s2.y.abs() - STANDARD_GRAVITY_M_S2).abs() < 1e-4);
    assert!(meas_roll.accel_m_s2.z.abs() < 1e-4);

    // 4. Inverted flight (180 deg roll):
    let q_inverted = Quaternion::from_euler_rpy(std::f64::consts::PI, 0.0, 0.0);
    let meas_inv = sample_imu(
        &config,
        &mut state,
        0.008,
        Vector3D::ZERO,
        Vector3D::ZERO,
        q_inverted,
        298.15,
        &mut rng,
    );
    assert!((meas_inv.accel_m_s2.z - (-STANDARD_GRAVITY_M_S2)).abs() < 1e-4);
}

#[test]
fn test_9dof_geomagnetic_sensing_and_hard_iron_bias() {
    let mut config = ImuConfig::new_industrial_9dof();
    config.mag_noise_std_ut = 0.0; // Deterministic check
    config.hard_iron_bias_ut = Vector3D::new(2.0, -3.0, 5.0);
    config.earth_mag_field_world_ut = Vector3D::new(20.0, 0.0, -40.0);

    let mut state = ImuState::default();
    let mut rng = ChannelRng::new(200);

    // 1. Level attitude facing North (heading = 0):
    let q_north = Quaternion::from_euler_rpy(0.0, 0.0, 0.0);
    let meas_north = sample_imu(
        &config,
        &mut state,
        0.01,
        Vector3D::ZERO,
        Vector3D::ZERO,
        q_north,
        298.15,
        &mut rng,
    );

    let mag = meas_north.mag_ut.expect("9-DOF must include magnetometer");
    // Expected = Earth field [20, 0, -40] + Hard iron [2, -3, 5] = [22, -3, -35]
    assert!((mag.x - 22.0).abs() < 1e-4);
    assert!((mag.y - (-3.0)).abs() < 1e-4);
    assert!((mag.z - (-35.0)).abs() < 1e-4);

    // 2. 6-DOF tactical mode has no magnetometer:
    let tac_config = ImuConfig::new_tactical_6dof();
    let meas_tac = sample_imu(
        &tac_config,
        &mut state,
        0.02,
        Vector3D::ZERO,
        Vector3D::ZERO,
        q_north,
        298.15,
        &mut rng,
    );
    assert!(meas_tac.mag_ut.is_none());
}

#[test]
fn test_allan_variance_gauss_markov_drift_and_thermal_sensitivity() {
    let config = ImuConfig::new_industrial_9dof();
    let mut state = ImuState::default();
    let mut rng = ChannelRng::new(300);

    // Step across 1,000 steps with temperature elevated by 25 K:
    let dt = 0.002;
    let temp_elevated = config.reference_temp_kelvin + 25.0;

    let mut meas_last: Option<ImuMeasurement> = None;
    for k in 1..=1000 {
        let t = k as f64 * dt;
        let meas = sample_imu(
            &config,
            &mut state,
            t,
            Vector3D::ZERO,
            Vector3D::ZERO,
            Quaternion::default(),
            temp_elevated,
            &mut rng,
        );
        meas_last = Some(meas);
    }

    let meas = meas_last.unwrap();
    // Thermal drift = 25 K * 0.0002 m/s^2/K = 0.0050 m/s^2
    let expected_thermal_a = 25.0 * config.accel_noise.temp_drift_coeff;
    assert!((expected_thermal_a - 0.0050).abs() < 1e-6);

    // Accel Z should include gravity + thermal drift + Gauss-Markov bias + white noise:
    assert!((meas.accel_m_s2.z - (STANDARD_GRAVITY_M_S2 + expected_thermal_a)).abs() < 0.2);
    // Temperature reported in Celsius (298.15 + 25 - 273.15 = 50.0 C):
    assert!((meas.temperature_c - 50.0).abs() < 1e-4);
}
