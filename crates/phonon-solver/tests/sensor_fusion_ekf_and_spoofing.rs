//! Integration Test: Multi-Rate ESKF Sensor Fusion & GPS Spoofing Rejection
//!
//! Validates:
//! 1. 15-state strapdown inertial navigation propagation at 500 Hz.
//! 2. Multi-rate asynchronous updates (Barometer 50 Hz, Magnetometer 50 Hz, GPS 10 Hz).
//! 3. Covariance convergence across filter cycles.
//! 4. Innovation Mahalanobis chi-square gating:
//!    - Clean GPS position updates are accepted.
//!    - Adversarial GPS spoofing teleportation is autonomously rejected.
//!    - State estimation remains uncorrupted during spoofing attack.
//! 5. Sensor dropout tolerance and dead-reckoning continuity.

use approx::assert_relative_eq;
use phonon_models::em::Vector3D;
use phonon_models::sensors::{
    BarometerSensor, GpsFixType, GpsMeasurement, ImuMeasurement, Quaternion, STANDARD_GRAVITY_M_S2,
};
use phonon_solver::sensors::sensor_fusion::{EskfConfig, EskfNominalState, MultiRateEskf};

#[test]
fn test_eskf_strapdown_propagation_and_covariance_growth() {
    let config = EskfConfig::default();
    let init_state = EskfNominalState {
        position: Vector3D::new(0.0, 0.0, 10.0),
        velocity: Vector3D::new(1.0, 0.0, 0.0),
        orientation: Quaternion::default(),
        accel_bias: Vector3D::ZERO,
        gyro_bias: Vector3D::ZERO,
    };

    let mut eskf = MultiRateEskf::new(config, init_state);
    let p_init_pos_var = eskf.covariance.data[0][0];

    // Accelerometer measures exact gravity reaction (stationary or constant velocity):
    // Specific force f_body = a_body - g_body = (0, 0, 0) - (0, 0, -g) = (0, 0, +g)
    let imu = ImuMeasurement {
        timestamp_s: 0.002,
        accel_m_s2: Vector3D::new(0.0, 0.0, STANDARD_GRAVITY_M_S2),
        gyro_rad_s: Vector3D::ZERO,
        mag_ut: None,
        temperature_c: 25.0,
    };

    let dt = 0.002;
    // Propagate for 500 steps (1.0 second):
    for i in 1..=500 {
        let mut step_imu = imu;
        step_imu.timestamp_s = i as f64 * dt;
        eskf.propagate_imu(&step_imu, dt);
    }

    // After 1.0 second at 1.0 m/s in X direction:
    assert_relative_eq!(eskf.nominal.position.x, 1.0, epsilon = 0.02);
    assert_relative_eq!(eskf.nominal.position.z, 10.0, epsilon = 0.02);

    // Process noise should cause covariance to grow in the absence of updates:
    assert!(eskf.covariance.data[0][0] > p_init_pos_var);
}

#[test]
fn test_barometer_and_magnetometer_asynchronous_updates() {
    let config = EskfConfig::default();
    let init_state = EskfNominalState {
        position: Vector3D::new(0.0, 0.0, 5.0),
        velocity: Vector3D::ZERO,
        orientation: Quaternion::default(),
        accel_bias: Vector3D::ZERO,
        gyro_bias: Vector3D::ZERO,
    };

    let mut eskf = MultiRateEskf::new(config, init_state);
    let baro = BarometerSensor::default();

    // Prior altitude variance:
    let p_z_before = eskf.covariance.data[2][2];

    // Barometer update at true altitude 5.0 m:
    let p_baro = baro.theoretical_pressure_pa(5.0);
    let baro_ok = eskf.update_barometer_pressure(p_baro, &baro);
    assert!(baro_ok);

    // Altitude variance must decrease after update:
    let p_z_after = eskf.covariance.data[2][2];
    assert!(p_z_after < p_z_before);

    // Magnetometer update with reference field:
    let p_att_before = eskf.covariance.data[8][8]; // Yaw variance
    let mag_field = eskf.config.earth_mag_world_ut;
    let mag_ok = eskf.update_magnetometer(mag_field);
    assert!(mag_ok);

    let p_att_after = eskf.covariance.data[8][8];
    assert!(p_att_after < p_att_before);
}

#[test]
fn test_gps_spoofing_attack_mahalanobis_rejection() {
    let config = EskfConfig::default();
    let init_state = EskfNominalState {
        position: Vector3D::new(10.0, 20.0, 30.0),
        velocity: Vector3D::new(2.0, 1.0, 0.0),
        orientation: Quaternion::default(),
        accel_bias: Vector3D::ZERO,
        gyro_bias: Vector3D::ZERO,
    };

    let mut eskf = MultiRateEskf::new(config, init_state);

    // 1. Clean GPS measurement close to true state:
    let clean_gps = GpsMeasurement {
        timestamp_s: 1.0,
        position_world_m: Vector3D::new(10.15, 19.90, 30.05), // Small 15-20 cm noise
        velocity_world_m_s: Vector3D::new(2.02, 0.98, 0.01),
        hdop: 0.8,
        vdop: 1.1,
        fix_type: GpsFixType::Fix3D,
        satellites_visible: 16,
        is_spoofed: false,
    };

    let (clean_pos_ok, clean_vel_ok) = eskf.update_gps(&clean_gps);
    assert!(
        clean_pos_ok,
        "Clean GPS position should pass Mahalanobis gate"
    );
    assert!(
        clean_vel_ok,
        "Clean GPS velocity should pass Mahalanobis gate"
    );
    assert_eq!(eskf.gps_pos_accepted_count, 1);
    assert_eq!(eskf.gps_pos_rejected_count, 0);

    // 2. Adversarial GPS Spoofing Attack: Teleportation of +50 meters:
    let spoofed_gps = GpsMeasurement {
        timestamp_s: 1.1,
        position_world_m: Vector3D::new(60.0, -30.0, 80.0), // Massive 50+ m spoofed jump
        velocity_world_m_s: Vector3D::new(2.02, 0.98, 0.01),
        hdop: 0.8,
        vdop: 1.1,
        fix_type: GpsFixType::Fix3D,
        satellites_visible: 16,
        is_spoofed: true,
    };

    let pos_before_spoof = eskf.nominal.position;
    let (spoof_pos_ok, _spoof_vel_ok) = eskf.update_gps(&spoofed_gps);

    // Spoofed position measurement MUST be rejected by chi-square gating:
    assert!(
        !spoof_pos_ok,
        "Spoofed GPS position must be rejected by Chi-square gate"
    );
    assert_eq!(eskf.gps_pos_rejected_count, 1);

    // State position must remain close to pre-spoof position, completely ignoring false teleportation:
    assert_relative_eq!(eskf.nominal.position.x, pos_before_spoof.x, epsilon = 1.0);
    assert_relative_eq!(eskf.nominal.position.y, pos_before_spoof.y, epsilon = 1.0);
    assert_relative_eq!(eskf.nominal.position.z, pos_before_spoof.z, epsilon = 1.0);
}

#[test]
fn test_sensor_dropout_graceful_dead_reckoning() {
    let config = EskfConfig::default();
    let init_state = EskfNominalState {
        position: Vector3D::new(0.0, 0.0, 2.0),
        velocity: Vector3D::new(0.5, 0.0, 0.0),
        orientation: Quaternion::default(),
        accel_bias: Vector3D::ZERO,
        gyro_bias: Vector3D::ZERO,
    };

    let mut eskf = MultiRateEskf::new(config, init_state);
    let baro = BarometerSensor::default();

    let dt = 0.002;
    // Simulate 2.0 seconds with dead reckoning: IMU running at 500 Hz, Baro running at 50 Hz, but NO GPS:
    for step in 1..=1000 {
        let t = step as f64 * dt;
        let imu = ImuMeasurement {
            timestamp_s: t,
            accel_m_s2: Vector3D::new(0.0, 0.0, STANDARD_GRAVITY_M_S2),
            gyro_rad_s: Vector3D::ZERO,
            mag_ut: None,
            temperature_c: 25.0,
        };
        eskf.propagate_imu(&imu, dt);

        if step % 10 == 0 {
            // Baro update at 50 Hz keeps altitude tightly bounded:
            eskf.update_barometer_pressure(baro.theoretical_pressure_pa(2.0), &baro);
        }
    }

    // Altitude remains well-estimated despite zero GPS:
    assert_relative_eq!(eskf.nominal.position.z, 2.0, epsilon = 0.15);
    // X position dead-reckoned at ~ 1.0 m:
    assert_relative_eq!(eskf.nominal.position.x, 1.0, epsilon = 0.25);
}
