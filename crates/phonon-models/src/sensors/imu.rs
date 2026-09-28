//! 6-DOF & 9-DOF Inertial Measurement Unit (IMU) & Allan Variance Noise Physics
//!
//! Formulates triaxial MEMS accelerometers, Coriolis vibratory gyroscopes, and magnetometers:
//! 1. Specific force kinematics: f_meas = a_body - g_body.
//! 2. Body-frame gravity vector rotation via unit quaternions.
//! 3. Triaxial Coriolis gyroscope measuring angular rate [p, q, r].
//! 4. Triaxial geomagnetic fluxgate/Hall magnetometer with hard-iron and soft-iron distortions.
//! 5. Stochastic noise processes: Angle Random Walk (ARW), Velocity Random Walk (VRW),
//!    first-order Gauss-Markov in-run bias instability, and thermal bias drift.

use crate::em::{ChannelRng, Vector3D};

/// Standard Earth gravitational acceleration constant g_0 in m/s^2.
pub const STANDARD_GRAVITY_M_S2: f64 = 9.80665;

/// 4D Unit Quaternion representing vehicle attitude orientation (q_w, q_x, q_y, q_z).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternion {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Default for Quaternion {
    fn default() -> Self {
        Self {
            w: 1.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

impl Quaternion {
    pub fn new(w: f64, x: f64, y: f64, z: f64) -> Self {
        let norm = (w * w + x * x + y * y + z * z).sqrt().max(1e-12);
        Self {
            w: w / norm,
            x: x / norm,
            y: y / norm,
            z: z / norm,
        }
    }

    /// Constructs a unit quaternion from aerospace Euler angles (Roll phi, Pitch theta, Yaw psi) in radians.
    /// Rotation sequence: Yaw (Z) -> Pitch (Y) -> Roll (X).
    pub fn from_euler_rpy(roll: f64, pitch: f64, yaw: f64) -> Self {
        let (cr, sr) = ((roll * 0.5).cos(), (roll * 0.5).sin());
        let (cp, sp) = ((pitch * 0.5).cos(), (pitch * 0.5).sin());
        let (cy, sy) = ((yaw * 0.5).cos(), (yaw * 0.5).sin());

        Self::new(
            cr * cp * cy + sr * sp * sy,
            sr * cp * cy - cr * sp * sy,
            cr * sp * cy + sr * cp * sy,
            cr * cp * sy - sr * sp * cy,
        )
    }

    /// Rotates a 3D vector v_world from world frame to body frame:
    /// v_body = q^* * v_world * q
    pub fn rotate_vector_world_to_body(&self, v: Vector3D) -> Vector3D {
        // Inverse quaternion q^* has negated vector components:
        let qw = self.w;
        let qx = -self.x;
        let qy = -self.y;
        let qz = -self.z;

        let ix = qw * v.x + qy * v.z - qz * v.y;
        let iy = qw * v.y + qz * v.x - qx * v.z;
        let iz = qw * v.z + qx * v.y - qy * v.x;
        let iw = -qx * v.x - qy * v.y - qz * v.z;

        Vector3D::new(
            ix * qw + iw * -qx + iy * -qz - iz * -qy,
            iy * qw + iw * -qy + iz * -qx - ix * -qz,
            iz * qw + iw * -qz + ix * -qy - iy * -qx,
        )
    }
}

/// Allan Variance and Stochastic Noise Parameters for a 3-axis sensor.
#[derive(Debug, Clone, PartialEq)]
pub struct AllanNoiseConfig {
    /// White noise spectral density (VRW for accel in m/s^2/sqrt(Hz), ARW for gyro in rad/s/sqrt(Hz)).
    pub white_noise_density: f64,
    /// In-run bias instability standard deviation (1/f flicker noise floor).
    pub bias_instability: f64,
    /// First-order Gauss-Markov bias correlation time in seconds (tau_corr).
    pub correlation_time_s: f64,
    /// Temperature drift coefficient per Kelvin.
    pub temp_drift_coeff: f64,
}

impl AllanNoiseConfig {
    /// Creates industrial MEMS accelerometer noise parameters (e.g. ADIS16488, BMI088).
    pub fn industrial_accelerometer() -> Self {
        Self {
            white_noise_density: 0.002, // 200 ug/sqrt(Hz) ~ 0.002 m/s^2/sqrt(Hz)
            bias_instability: 0.0005,   // ~50 ug in-run bias stability
            correlation_time_s: 300.0,  // 5 minute correlation time
            temp_drift_coeff: 0.0002,   // 0.2 mm/s^2 per Kelvin
        }
    }

    /// Creates industrial MEMS gyroscope noise parameters (e.g. ADIS16488, BMI088).
    pub fn industrial_gyroscope() -> Self {
        Self {
            white_noise_density: 0.0003, // ~0.1 deg/sqrt(hr) ~ 0.0003 rad/s/sqrt(Hz)
            bias_instability: 0.00008,   // ~5 deg/hr in-run stability
            correlation_time_s: 600.0,   // 10 minute correlation time
            temp_drift_coeff: 0.00005,   // 50 urad/s per Kelvin
        }
    }
}

/// Comprehensive 6-DOF / 9-DOF IMU Sensor Configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct ImuConfig {
    /// Accelerometer stochastic noise parameters.
    pub accel_noise: AllanNoiseConfig,
    /// Gyroscope stochastic noise parameters.
    pub gyro_noise: AllanNoiseConfig,
    /// Magnetometer white noise standard deviation in micro-Tesla (uT).
    pub mag_noise_std_ut: f64,
    /// Hard-iron magnetic bias offset in micro-Tesla (uT).
    pub hard_iron_bias_ut: Vector3D,
    /// Reference operating temperature in Kelvin (T_0, e.g. 298.15 K = 25 C).
    pub reference_temp_kelvin: f64,
    /// Local Earth geomagnetic field vector in world frame in micro-Tesla (uT).
    pub earth_mag_field_world_ut: Vector3D,
    /// Flag enabling 9-DOF magnetometer sensing.
    pub has_magnetometer: bool,
}

impl Default for ImuConfig {
    fn default() -> Self {
        Self::new_industrial_9dof()
    }
}

impl ImuConfig {
    /// Constructs a standard high-performance industrial 9-DOF IMU configuration.
    pub fn new_industrial_9dof() -> Self {
        Self {
            accel_noise: AllanNoiseConfig::industrial_accelerometer(),
            gyro_noise: AllanNoiseConfig::industrial_gyroscope(),
            mag_noise_std_ut: 0.15, // 0.15 uT RMS noise
            hard_iron_bias_ut: Vector3D::new(2.5, -1.8, 4.0),
            reference_temp_kelvin: 298.15, // 25 C
            // Standard mid-latitude geomagnetic field: ~22 uT North, 0 East, 42 uT Down:
            earth_mag_field_world_ut: Vector3D::new(22.0, 0.0, -42.0),
            has_magnetometer: true,
        }
    }

    /// Constructs a 6-DOF tactical aerospace IMU (accelerometer + gyro only).
    pub fn new_tactical_6dof() -> Self {
        let mut cfg = Self::new_industrial_9dof();
        cfg.has_magnetometer = false;
        cfg
    }
}

/// Dynamic internal bias state of the IMU tracking Gauss-Markov stochastic evolution.
#[derive(Debug, Clone, PartialEq)]
pub struct ImuState {
    pub accel_bias: Vector3D,
    pub gyro_bias: Vector3D,
    pub last_timestamp_s: f64,
}

impl Default for ImuState {
    fn default() -> Self {
        Self {
            accel_bias: Vector3D::new(0.0, 0.0, 0.0),
            gyro_bias: Vector3D::new(0.0, 0.0, 0.0),
            last_timestamp_s: 0.0,
        }
    }
}

impl ImuState {
    /// Updates the Gauss-Markov stochastic bias state across time step dt in seconds.
    pub fn step_bias(&mut self, dt: f64, config: &ImuConfig, rng: &mut ChannelRng) {
        if dt <= 1e-9 {
            return;
        }

        // Accelerometer Gauss-Markov bias stepping:
        let tau_a = config.accel_noise.correlation_time_s.max(1.0);
        let decay_a = (-dt / tau_a).exp();
        let driving_std_a =
            config.accel_noise.bias_instability * (1.0 - (-2.0 * dt / tau_a).exp()).sqrt();
        let (ga1, ga2) = rng.next_gaussian();
        let (ga3, _) = rng.next_gaussian();

        self.accel_bias.x = self.accel_bias.x * decay_a + ga1 * driving_std_a;
        self.accel_bias.y = self.accel_bias.y * decay_a + ga2 * driving_std_a;
        self.accel_bias.z = self.accel_bias.z * decay_a + ga3 * driving_std_a;

        // Gyroscope Gauss-Markov bias stepping:
        let tau_g = config.gyro_noise.correlation_time_s.max(1.0);
        let decay_g = (-dt / tau_g).exp();
        let driving_std_g =
            config.gyro_noise.bias_instability * (1.0 - (-2.0 * dt / tau_g).exp()).sqrt();
        let (gg1, gg2) = rng.next_gaussian();
        let (gg3, _) = rng.next_gaussian();

        self.gyro_bias.x = self.gyro_bias.x * decay_g + gg1 * driving_std_g;
        self.gyro_bias.y = self.gyro_bias.y * decay_g + gg2 * driving_std_g;
        self.gyro_bias.z = self.gyro_bias.z * decay_g + gg3 * driving_std_g;
    }
}

/// Instantaneous calibrated telemetry measurement packet from the IMU.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImuMeasurement {
    /// Timestamp of the sample in seconds.
    pub timestamp_s: f64,
    /// Measured body-frame specific force in m/s^2 (a_body - g_body).
    pub accel_m_s2: Vector3D,
    /// Measured body-frame angular rate [p, q, r] in rad/s.
    pub gyro_rad_s: Vector3D,
    /// Measured body-frame geomagnetic field vector in micro-Tesla (uT) if 9-DOF.
    pub mag_ut: Option<Vector3D>,
    /// Instantaneous sensor die temperature in Celsius.
    pub temperature_c: f64,
}

/// Transduces true vehicle kinematics into noisy, bias-drifted IMU telemetry.
#[allow(clippy::too_many_arguments)]
pub fn sample_imu(
    config: &ImuConfig,
    state: &mut ImuState,
    timestamp_s: f64,
    body_accel: Vector3D,
    body_gyro: Vector3D,
    orientation: Quaternion,
    temp_kelvin: f64,
    rng: &mut ChannelRng,
) -> ImuMeasurement {
    let dt = (timestamp_s - state.last_timestamp_s).max(1e-4);
    state.step_bias(dt, config, rng);
    state.last_timestamp_s = timestamp_s;

    // 1. Gravity vector rotated into vehicle body frame:
    let g_world = Vector3D::new(0.0, 0.0, -STANDARD_GRAVITY_M_S2);
    let g_body = orientation.rotate_vector_world_to_body(g_world);

    // Specific force: f = a_body - g_body
    let f_ideal = body_accel - g_body;

    // Thermal drift offset:
    let delta_t = temp_kelvin - config.reference_temp_kelvin;
    let thermal_bias_a = Vector3D::new(
        config.accel_noise.temp_drift_coeff * delta_t,
        config.accel_noise.temp_drift_coeff * delta_t,
        config.accel_noise.temp_drift_coeff * delta_t,
    );

    // White noise Velocity Random Walk:
    let white_std_a = config.accel_noise.white_noise_density / dt.sqrt().max(1e-4);
    let (wa1, wa2) = rng.next_gaussian();
    let (wa3, _) = rng.next_gaussian();
    let white_noise_a = Vector3D::new(wa1 * white_std_a, wa2 * white_std_a, wa3 * white_std_a);

    let accel_meas = f_ideal + state.accel_bias + thermal_bias_a + white_noise_a;

    // 2. Gyroscope measurement:
    let thermal_bias_g = Vector3D::new(
        config.gyro_noise.temp_drift_coeff * delta_t,
        config.gyro_noise.temp_drift_coeff * delta_t,
        config.gyro_noise.temp_drift_coeff * delta_t,
    );

    let white_std_g = config.gyro_noise.white_noise_density / dt.sqrt().max(1e-4);
    let (wg1, wg2) = rng.next_gaussian();
    let (wg3, _) = rng.next_gaussian();
    let white_noise_g = Vector3D::new(wg1 * white_std_g, wg2 * white_std_g, wg3 * white_std_g);

    let gyro_meas = body_gyro + state.gyro_bias + thermal_bias_g + white_noise_g;

    // 3. Magnetometer measurement (if enabled):
    let mag_meas = if config.has_magnetometer {
        let b_body = orientation.rotate_vector_world_to_body(config.earth_mag_field_world_ut);
        let (wm1, wm2) = rng.next_gaussian();
        let (wm3, _) = rng.next_gaussian();
        let white_noise_m = Vector3D::new(
            wm1 * config.mag_noise_std_ut,
            wm2 * config.mag_noise_std_ut,
            wm3 * config.mag_noise_std_ut,
        );
        Some(b_body + config.hard_iron_bias_ut + white_noise_m)
    } else {
        None
    };

    ImuMeasurement {
        timestamp_s,
        accel_m_s2: accel_meas,
        gyro_rad_s: gyro_meas,
        mag_ut: mag_meas,
        temperature_c: temp_kelvin - 273.15,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quaternion_euler_rotation() {
        let q_ident = Quaternion::from_euler_rpy(0.0, 0.0, 0.0);
        let v = Vector3D::new(0.0, 0.0, -STANDARD_GRAVITY_M_S2);
        let v_rot = q_ident.rotate_vector_world_to_body(v);
        assert!((v_rot.z - (-STANDARD_GRAVITY_M_S2)).abs() < 1e-6);

        // Pitch 90 deg up (theta = pi/2):
        let q_pitch90 = Quaternion::from_euler_rpy(0.0, std::f64::consts::FRAC_PI_2, 0.0);
        let v_pitch =
            q_pitch90.rotate_vector_world_to_body(Vector3D::new(0.0, 0.0, -STANDARD_GRAVITY_M_S2));
        assert!((v_pitch.x - STANDARD_GRAVITY_M_S2).abs() < 1e-4);
        assert!(v_pitch.y.abs() < 1e-4);
        assert!(v_pitch.z.abs() < 1e-4);
    }

    #[test]
    fn test_stationary_imu_gravity_sensing() {
        let config = ImuConfig::new_industrial_9dof();
        let mut state = ImuState::default();
        let mut rng = ChannelRng::new(42);

        let meas = sample_imu(
            &config,
            &mut state,
            0.01,
            Vector3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, 0.0),
            Quaternion::default(),
            298.15,
            &mut rng,
        );

        // Specific force should read +9.80665 m/s^2 upwards along body Z (with small noise):
        assert!((meas.accel_m_s2.z - STANDARD_GRAVITY_M_S2).abs() < 0.2);
        assert!(meas.mag_ut.is_some());
    }
}
