//! 15-State Multi-Rate Error-State Kalman Filter (ESKF) & Sensor Fusion Engine
//!
//! Provides strapdown inertial navigation propagation and asynchronous multi-rate
//! sensor updates with Mahalanobis chi-square innovation gating for outlier / spoofing rejection:
//! 1. 15-state error vector: \delta x = [\delta p, \delta v, \delta \theta, \delta b_a, \delta b_g]^T.
//! 2. High-rate (500 Hz) IMU strapdown mechanization and covariance propagation.
//! 3. Asynchronous multi-rate observation updates:
//!    - Barometer / LiDAR Altitude (50 Hz)
//!    - Magnetometer Heading (50 Hz)
//!    - GNSS/GPS Position & Velocity (10 Hz)
#![allow(clippy::needless_range_loop)]

use phonon_models::em::Vector3D;
use phonon_models::sensors::{
    BarometerSensor, GpsMeasurement, ImuMeasurement, Quaternion, STANDARD_GRAVITY_M_S2,
};

/// 3x3 Matrix with basic stack-allocated linear algebra operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix3x3 {
    pub data: [[f64; 3]; 3],
}

impl Matrix3x3 {
    pub const ZERO: Self = Self {
        data: [[0.0; 3]; 3],
    };

    pub const IDENTITY: Self = Self {
        data: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    };

    #[inline]
    pub fn new(data: [[f64; 3]; 3]) -> Self {
        Self { data }
    }

    #[inline]
    pub fn from_diagonal(v: Vector3D) -> Self {
        Self {
            data: [[v.x, 0.0, 0.0], [0.0, v.y, 0.0], [0.0, 0.0, v.z]],
        }
    }

    /// Skew-symmetric cross-product matrix: [v]_\times.
    #[inline]
    pub fn skew_symmetric(v: Vector3D) -> Self {
        Self {
            data: [[0.0, -v.z, v.y], [v.z, 0.0, -v.x], [-v.y, v.x, 0.0]],
        }
    }

    #[inline]
    pub fn multiply(&self, other: &Self) -> Self {
        let mut out = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                out[i][j] = self.data[i][0] * other.data[0][j]
                    + self.data[i][1] * other.data[1][j]
                    + self.data[i][2] * other.data[2][j];
            }
        }
        Self { data: out }
    }

    #[inline]
    pub fn multiply_vector(&self, v: Vector3D) -> Vector3D {
        Vector3D::new(
            self.data[0][0] * v.x + self.data[0][1] * v.y + self.data[0][2] * v.z,
            self.data[1][0] * v.x + self.data[1][1] * v.y + self.data[1][2] * v.z,
            self.data[2][0] * v.x + self.data[2][1] * v.y + self.data[2][2] * v.z,
        )
    }

    #[inline]
    pub fn transpose(&self) -> Self {
        Self {
            data: [
                [self.data[0][0], self.data[1][0], self.data[2][0]],
                [self.data[0][1], self.data[1][1], self.data[2][1]],
                [self.data[0][2], self.data[1][2], self.data[2][2]],
            ],
        }
    }

    #[inline]
    pub fn determinant(&self) -> f64 {
        let d = &self.data;
        d[0][0] * (d[1][1] * d[2][2] - d[1][2] * d[2][1])
            - d[0][1] * (d[1][0] * d[2][2] - d[1][2] * d[2][0])
            + d[0][2] * (d[1][0] * d[2][1] - d[1][1] * d[2][0])
    }

    /// Computes matrix inverse if non-singular.
    pub fn inverse(&self) -> Option<Self> {
        let det = self.determinant();
        if det.abs() < 1e-15 {
            return None;
        }
        let inv_det = 1.0 / det;
        let d = &self.data;

        Some(Self {
            data: [
                [
                    (d[1][1] * d[2][2] - d[1][2] * d[2][1]) * inv_det,
                    (d[0][2] * d[2][1] - d[0][1] * d[2][2]) * inv_det,
                    (d[0][1] * d[1][2] - d[0][2] * d[1][1]) * inv_det,
                ],
                [
                    (d[1][2] * d[2][0] - d[1][0] * d[2][2]) * inv_det,
                    (d[0][0] * d[2][2] - d[0][2] * d[2][0]) * inv_det,
                    (d[0][2] * d[1][0] - d[0][0] * d[1][2]) * inv_det,
                ],
                [
                    (d[1][0] * d[2][1] - d[1][1] * d[2][0]) * inv_det,
                    (d[0][1] * d[2][0] - d[0][0] * d[2][1]) * inv_det,
                    (d[0][0] * d[1][1] - d[0][1] * d[1][0]) * inv_det,
                ],
            ],
        })
    }
}

/// 15x15 Covariance and Transition Matrix for Error-State Kalman Filter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix15x15 {
    pub data: [[f64; 15]; 15],
}

impl Matrix15x15 {
    pub const ZERO: Self = Self {
        data: [[0.0; 15]; 15],
    };

    pub fn identity() -> Self {
        let mut d = [[0.0; 15]; 15];
        for i in 0..15 {
            d[i][i] = 1.0;
        }
        Self { data: d }
    }

    pub fn from_diagonal(diag: &[f64; 15]) -> Self {
        let mut d = [[0.0; 15]; 15];
        for i in 0..15 {
            d[i][i] = diag[i];
        }
        Self { data: d }
    }

    pub fn add(&self, other: &Self) -> Self {
        let mut out = [[0.0; 15]; 15];
        for i in 0..15 {
            for j in 0..15 {
                out[i][j] = self.data[i][j] + other.data[i][j];
            }
        }
        Self { data: out }
    }

    pub fn multiply(&self, other: &Self) -> Self {
        let mut out = [[0.0; 15]; 15];
        for i in 0..15 {
            for k in 0..15 {
                let r = self.data[i][k];
                if r != 0.0 {
                    for j in 0..15 {
                        out[i][j] += r * other.data[k][j];
                    }
                }
            }
        }
        Self { data: out }
    }

    pub fn transpose(&self) -> Self {
        let mut out = [[0.0; 15]; 15];
        for i in 0..15 {
            for j in 0..15 {
                out[i][j] = self.data[j][i];
            }
        }
        Self { data: out }
    }

    pub fn symmetrize(&mut self) {
        for i in 0..15 {
            for j in (i + 1)..15 {
                let avg = 0.5 * (self.data[i][j] + self.data[j][i]);
                self.data[i][j] = avg;
                self.data[j][i] = avg;
            }
        }
    }
}

/// ESKF Noise and Tuning Parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct EskfConfig {
    /// Accelerometer continuous noise spectral density (m/s^2 / sqrt(Hz)).
    pub accel_noise_density: f64,
    /// Gyroscope continuous noise spectral density (rad/s / sqrt(Hz)).
    pub gyro_noise_density: f64,
    /// Accelerometer random walk bias instability (m/s^3 / sqrt(Hz)).
    pub accel_bias_instability: f64,
    /// Gyroscope random walk bias instability (rad/s^2 / sqrt(Hz)).
    pub gyro_bias_instability: f64,
    /// Barometer altitude measurement variance R_alt in m^2.
    pub var_alt: f64,
    /// Magnetometer measurement variance per axis R_mag in uT^2.
    pub var_mag: f64,
    /// GNSS position measurement variance R_pos in m^2.
    pub var_gps_pos: f64,
    /// GNSS velocity measurement variance R_vel in (m/s)^2.
    pub var_gps_vel: f64,
    /// Chi-square gating threshold for 1-DOF altitude updates (p=0.99 ~ 6.635, p=0.999 ~ 10.828).
    pub chi2_gate_alt: f64,
    /// Chi-square gating threshold for 3-DOF magnetometer updates (p=0.99 ~ 11.345).
    pub chi2_gate_mag: f64,
    /// Chi-square gating threshold for 3-DOF GPS position updates (p=0.99 ~ 11.345, p=0.999 ~ 16.266).
    pub chi2_gate_gps_pos: f64,
    /// Chi-square gating threshold for 3-DOF GPS velocity updates (p=0.99 ~ 11.345).
    pub chi2_gate_gps_vel: f64,
    /// Reference Earth magnetic field vector in world frame (micro-Tesla).
    pub earth_mag_world_ut: Vector3D,
    /// Calibrated hard-iron magnetic bias offset (micro-Tesla).
    pub hard_iron_bias_ut: Vector3D,
}

impl Default for EskfConfig {
    fn default() -> Self {
        Self {
            accel_noise_density: 0.05,
            gyro_noise_density: 0.005,
            accel_bias_instability: 0.001,
            gyro_bias_instability: 0.0001,
            var_alt: 0.25,       // (0.5 m)^2
            var_mag: 4.0,        // (2.0 uT)^2
            var_gps_pos: 0.16,   // (0.4 m)^2
            var_gps_vel: 0.0064, // (0.08 m/s)^2
            chi2_gate_alt: 9.0,
            chi2_gate_mag: 11.345,
            chi2_gate_gps_pos: 11.345,
            chi2_gate_gps_vel: 11.345,
            earth_mag_world_ut: Vector3D::new(22.0, 0.0, -42.0),
            hard_iron_bias_ut: Vector3D::new(2.5, -1.8, 4.0),
        }
    }
}

/// Nominal Navigation State for 15-State ESKF.
#[derive(Debug, Clone, PartialEq)]
pub struct EskfNominalState {
    /// World Cartesian position (x, y, z) in meters.
    pub position: Vector3D,
    /// World Cartesian linear velocity (vx, vy, vz) in m/s.
    pub velocity: Vector3D,
    /// Attitude orientation unit quaternion (w, x, y, z).
    pub orientation: Quaternion,
    /// Body-frame accelerometer bias estimate in m/s^2.
    pub accel_bias: Vector3D,
    /// Body-frame gyroscope bias estimate in rad/s.
    pub gyro_bias: Vector3D,
}

impl Default for EskfNominalState {
    fn default() -> Self {
        Self {
            position: Vector3D::ZERO,
            velocity: Vector3D::ZERO,
            orientation: Quaternion::default(),
            accel_bias: Vector3D::ZERO,
            gyro_bias: Vector3D::ZERO,
        }
    }
}

/// 15-State Multi-Rate Error-State Kalman Filter (ESKF).
#[derive(Debug, Clone, PartialEq)]
pub struct MultiRateEskf {
    pub config: EskfConfig,
    pub nominal: EskfNominalState,
    pub covariance: Matrix15x15,
    pub last_update_time_s: f64,
    /// Total accepted GPS position updates.
    pub gps_pos_accepted_count: usize,
    /// Total rejected GPS position updates due to Mahalanobis chi-square gate (spoofing / jamming).
    pub gps_pos_rejected_count: usize,
    /// Total accepted altitude updates (baro / lidar).
    pub alt_accepted_count: usize,
    /// Total rejected altitude updates.
    pub alt_rejected_count: usize,
    /// Total accepted magnetometer updates.
    pub mag_accepted_count: usize,
    /// Total rejected magnetometer updates.
    pub mag_rejected_count: usize,
}

impl MultiRateEskf {
    /// Constructs a new ESKF with initial state and standard prior covariance.
    pub fn new(config: EskfConfig, initial_state: EskfNominalState) -> Self {
        let mut init_p = [0.0; 15];
        // Initial 1-sigma standard deviations:
        init_p[0] = 0.5 * 0.5; // pos x (0.5 m)
        init_p[1] = 0.5 * 0.5; // pos y
        init_p[2] = 0.5 * 0.5; // pos z
        init_p[3] = 0.2 * 0.2; // vel x (0.2 m/s)
        init_p[4] = 0.2 * 0.2; // vel y
        init_p[5] = 0.2 * 0.2; // vel z
        init_p[6] = 0.05 * 0.05; // att x (0.05 rad ~ 3 deg)
        init_p[7] = 0.05 * 0.05; // att y
        init_p[8] = 0.05 * 0.05; // att z
        init_p[9] = 0.02 * 0.02; // accel bias (0.02 m/s^2)
        init_p[10] = 0.02 * 0.02;
        init_p[11] = 0.02 * 0.02;
        init_p[12] = 0.002 * 0.002; // gyro bias (0.002 rad/s)
        init_p[13] = 0.002 * 0.002;
        init_p[14] = 0.002 * 0.002;

        Self {
            config,
            nominal: initial_state,
            covariance: Matrix15x15::from_diagonal(&init_p),
            last_update_time_s: 0.0,
            gps_pos_accepted_count: 0,
            gps_pos_rejected_count: 0,
            alt_accepted_count: 0,
            alt_rejected_count: 0,
            mag_accepted_count: 0,
            mag_rejected_count: 0,
        }
    }

    /// High-Rate (500 Hz) Strapdown IMU Inertial Navigation Propagation.
    /// Propagates nominal state kinematics and covariance matrix over time step dt.
    pub fn propagate_imu(&mut self, imu: &ImuMeasurement, dt: f64) {
        if dt <= 1e-9 {
            return;
        }

        // 1. Unbias sensor readings:
        let f_b = imu.accel_m_s2 - self.nominal.accel_bias;
        let w_b = imu.gyro_rad_s - self.nominal.gyro_bias;

        // 2. Body-to-World rotation matrix and attitude update:
        let r_mat_raw = self.nominal.orientation.to_rotation_matrix_3x3();
        let r_bw = Matrix3x3::new(r_mat_raw);

        // Attitude propagation:
        self.nominal.orientation = self.nominal.orientation.integrate_angular_velocity(w_b, dt);

        // Specific force in world coordinates:
        let f_w = r_bw.multiply_vector(f_b);
        let g_w = Vector3D::new(0.0, 0.0, -STANDARD_GRAVITY_M_S2);
        let a_w = f_w + g_w;

        // Kinematics propagation:
        self.nominal.position =
            self.nominal.position + self.nominal.velocity * dt + a_w * (0.5 * dt * dt);
        self.nominal.velocity = self.nominal.velocity + a_w * dt;

        // 3. Discrete State Transition Matrix F:
        let mut f_matrix = Matrix15x15::identity();

        // dp / dv = I_3 * dt:
        for i in 0..3 {
            f_matrix.data[i][i + 3] = dt;
        }

        // dv / d_theta = - R_bw * [f_b]_\times * dt:
        let skew_fb = Matrix3x3::skew_symmetric(f_b);
        let dv_dtheta = r_bw.multiply(&skew_fb);
        for i in 0..3 {
            for j in 0..3 {
                f_matrix.data[i + 3][j + 6] = -dv_dtheta.data[i][j] * dt;
            }
        }

        // dv / d_ba = - R_bw * dt:
        for i in 0..3 {
            for j in 0..3 {
                f_matrix.data[i + 3][j + 9] = -r_bw.data[i][j] * dt;
            }
        }

        // d_theta / d_theta = I_3 - [w_b]_\times * dt:
        let skew_wb = Matrix3x3::skew_symmetric(w_b);
        for i in 0..3 {
            for j in 0..3 {
                f_matrix.data[i + 6][j + 6] -= skew_wb.data[i][j] * dt;
            }
        }

        // d_theta / d_bg = - I_3 * dt:
        for i in 0..3 {
            f_matrix.data[i + 6][i + 12] = -dt;
        }

        // 4. Continuous-time Process Noise Matrix Q:
        let mut q_diag = [0.0; 15];
        let q_pos = 0.001 * dt;
        let q_vel = (self.config.accel_noise_density * self.config.accel_noise_density) * dt;
        let q_att = (self.config.gyro_noise_density * self.config.gyro_noise_density) * dt;
        let q_ba = (self.config.accel_bias_instability * self.config.accel_bias_instability) * dt;
        let q_bg = (self.config.gyro_bias_instability * self.config.gyro_bias_instability) * dt;

        q_diag[0] = q_pos;
        q_diag[1] = q_pos;
        q_diag[2] = q_pos;
        q_diag[3] = q_vel;
        q_diag[4] = q_vel;
        q_diag[5] = q_vel;
        q_diag[6] = q_att;
        q_diag[7] = q_att;
        q_diag[8] = q_att;
        q_diag[9] = q_ba;
        q_diag[10] = q_ba;
        q_diag[11] = q_ba;
        q_diag[12] = q_bg;
        q_diag[13] = q_bg;
        q_diag[14] = q_bg;

        let q_mat = Matrix15x15::from_diagonal(&q_diag);

        // Covariance propagation: P = F * P * F^T + Q
        let f_p = f_matrix.multiply(&self.covariance);
        let f_t = f_matrix.transpose();
        let mut next_p = f_p.multiply(&f_t).add(&q_mat);
        next_p.symmetrize();
        self.covariance = next_p;
        self.last_update_time_s = imu.timestamp_s;
    }

    /// Asynchronous Barometer / LiDAR Altitude Measurement Update (50 Hz).
    /// Performs 1-DOF scalar innovation chi-square gating.
    pub fn update_altitude(&mut self, measured_altitude_m: f64, measurement_var: f64) -> bool {
        // Measurement model: z = p_z + v
        let y = measured_altitude_m - self.nominal.position.z;
        let r_meas = measurement_var.max(self.config.var_alt);
        let s = self.covariance.data[2][2] + r_meas;

        if s <= 1e-12 {
            return false;
        }

        // Mahalanobis distance squared: gamma = y^2 / S
        let gamma = (y * y) / s;
        if gamma > self.config.chi2_gate_alt {
            self.alt_rejected_count += 1;
            return false;
        }

        // Kalman gain: K = P[:, 2] / S
        let mut k = [0.0; 15];
        for i in 0..15 {
            k[i] = self.covariance.data[i][2] / s;
        }

        // Error state correction:
        let mut dx = [0.0; 15];
        for i in 0..15 {
            dx[i] = k[i] * y;
        }

        // Covariance update: P = (I - K * H) * P
        let mut new_p = self.covariance;
        for i in 0..15 {
            for j in 0..15 {
                new_p.data[i][j] -= k[i] * self.covariance.data[2][j];
            }
        }
        new_p.symmetrize();
        self.covariance = new_p;

        // Inject error state into nominal state:
        self.inject_error_state(&dx);
        self.alt_accepted_count += 1;
        true
    }

    /// Asynchronous Barometer Pressure Update (translates raw Pa to altitude).
    pub fn update_barometer_pressure(&mut self, pressure_pa: f64, baro: &BarometerSensor) -> bool {
        let est_alt = baro.pressure_to_altitude(pressure_pa);
        self.update_altitude(est_alt, self.config.var_alt)
    }

    /// Asynchronous Magnetometer Heading Measurement Update (50 Hz).
    /// Formulates 3D body geomagnetic flux innovation gating.
    pub fn update_magnetometer(&mut self, mag_meas_ut: Vector3D) -> bool {
        // Predicted body magnetic field: m_b_hat = R_wb * m_earth_w
        let r_mat_raw = self.nominal.orientation.to_rotation_matrix_3x3();
        let r_bw = Matrix3x3::new(r_mat_raw);
        let r_wb = r_bw.transpose();

        let m_b_hat = r_wb.multiply_vector(self.config.earth_mag_world_ut);
        let calibrated_mag = mag_meas_ut - self.config.hard_iron_bias_ut;
        let y = calibrated_mag - m_b_hat;

        // Observation matrix H: d(m_b) / d_theta = [m_b_hat]_\times
        let skew_m = Matrix3x3::skew_symmetric(m_b_hat);

        // Sub-covariance for attitude error P_theta (3x3):
        let mut p_att = Matrix3x3::ZERO;
        for i in 0..3 {
            for j in 0..3 {
                p_att.data[i][j] = self.covariance.data[i + 6][j + 6];
            }
        }

        // Innovation covariance S = H * P_att * H^T + R_mag
        let hp = skew_m.multiply(&p_att);
        let hph_t = hp.multiply(&skew_m.transpose());
        let mut s_mat = hph_t;
        for i in 0..3 {
            s_mat.data[i][i] += self.config.var_mag;
        }

        let s_inv = match s_mat.inverse() {
            Some(inv) => inv,
            None => return false,
        };

        // Mahalanobis distance: gamma = y^T * S^{-1} * y
        let s_inv_y = s_inv.multiply_vector(y);
        let gamma = y.dot(&s_inv_y);

        if gamma > self.config.chi2_gate_mag {
            self.mag_rejected_count += 1;
            return false;
        }

        // Kalman gain for 15 states: K = P * H^T * S^{-1}
        let mut k_15x3 = [[0.0; 3]; 15];
        for i in 0..15 {
            for j in 0..3 {
                let mut p_ht_row = 0.0;
                for k in 0..3 {
                    // H is only non-zero for columns 6..9 (skew_m):
                    p_ht_row += self.covariance.data[i][k + 6] * skew_m.data[j][k];
                }
                for k in 0..3 {
                    k_15x3[i][k] += p_ht_row * s_inv.data[j][k];
                }
            }
        }

        // Error state: dx = K * y
        let mut dx = [0.0; 15];
        for i in 0..15 {
            dx[i] = k_15x3[i][0] * y.x + k_15x3[i][1] * y.y + k_15x3[i][2] * y.z;
        }

        // Covariance update: P = (I - K * H) * P
        let mut new_p = self.covariance;
        for i in 0..15 {
            for j in 0..15 {
                let mut kh_p = 0.0;
                for k in 0..3 {
                    for m in 0..3 {
                        kh_p += k_15x3[i][k] * skew_m.data[k][m] * self.covariance.data[m + 6][j];
                    }
                }
                new_p.data[i][j] -= kh_p;
            }
        }
        new_p.symmetrize();
        self.covariance = new_p;

        self.inject_error_state(&dx);
        self.mag_accepted_count += 1;
        true
    }

    /// Asynchronous GNSS/GPS Position & Velocity Measurement Update (10 Hz).
    /// Executes innovation Mahalanobis gating for autonomous GPS spoofing / jamming rejection.
    /// Returns (pos_accepted: bool, vel_accepted: bool).
    pub fn update_gps(&mut self, gps: &GpsMeasurement) -> (bool, bool) {
        // 1. Position update:
        let y_pos = gps.position_world_m - self.nominal.position;

        // Sub-covariance for position P_pos:
        let mut p_pos = Matrix3x3::ZERO;
        for i in 0..3 {
            for j in 0..3 {
                p_pos.data[i][j] = self.covariance.data[i][j];
            }
        }

        // S_pos = P_pos + R_pos * I_3
        let mut s_pos = p_pos;
        for i in 0..3 {
            s_pos.data[i][i] += self.config.var_gps_pos;
        }

        let pos_accepted = if let Some(s_inv) = s_pos.inverse() {
            let s_inv_y = s_inv.multiply_vector(y_pos);
            let gamma = y_pos.dot(&s_inv_y);

            if gamma <= self.config.chi2_gate_gps_pos {
                // Compute Kalman gain: K_pos = P[:, 0..3] * S^{-1}
                let mut k_pos = [[0.0; 3]; 15];
                for i in 0..15 {
                    for j in 0..3 {
                        for k in 0..3 {
                            k_pos[i][j] += self.covariance.data[i][k] * s_inv.data[k][j];
                        }
                    }
                }

                // Error state update:
                let mut dx = [0.0; 15];
                for i in 0..15 {
                    dx[i] = k_pos[i][0] * y_pos.x + k_pos[i][1] * y_pos.y + k_pos[i][2] * y_pos.z;
                }

                // Covariance update: P = (I - K_pos * H_pos) * P
                let mut new_p = self.covariance;
                for i in 0..15 {
                    for j in 0..15 {
                        let mut kh_p = 0.0;
                        for k in 0..3 {
                            kh_p += k_pos[i][k] * self.covariance.data[k][j];
                        }
                        new_p.data[i][j] -= kh_p;
                    }
                }
                new_p.symmetrize();
                self.covariance = new_p;

                self.inject_error_state(&dx);
                self.gps_pos_accepted_count += 1;
                true
            } else {
                self.gps_pos_rejected_count += 1;
                false
            }
        } else {
            self.gps_pos_rejected_count += 1;
            false
        };

        // 2. Velocity update:
        let y_vel = gps.velocity_world_m_s - self.nominal.velocity;
        let mut p_vel = Matrix3x3::ZERO;
        for i in 0..3 {
            for j in 0..3 {
                p_vel.data[i][j] = self.covariance.data[i + 3][j + 3];
            }
        }

        let mut s_vel = p_vel;
        for i in 0..3 {
            s_vel.data[i][i] += self.config.var_gps_vel;
        }

        let vel_accepted = if let Some(s_inv) = s_vel.inverse() {
            let s_inv_y = s_inv.multiply_vector(y_vel);
            let gamma = y_vel.dot(&s_inv_y);

            if gamma <= self.config.chi2_gate_gps_vel {
                let mut k_vel = [[0.0; 3]; 15];
                for i in 0..15 {
                    for j in 0..3 {
                        for k in 0..3 {
                            k_vel[i][j] += self.covariance.data[i][k + 3] * s_inv.data[k][j];
                        }
                    }
                }

                let mut dx = [0.0; 15];
                for i in 0..15 {
                    dx[i] = k_vel[i][0] * y_vel.x + k_vel[i][1] * y_vel.y + k_vel[i][2] * y_vel.z;
                }

                let mut new_p = self.covariance;
                for i in 0..15 {
                    for j in 0..15 {
                        let mut kh_p = 0.0;
                        for k in 0..3 {
                            kh_p += k_vel[i][k] * self.covariance.data[k + 3][j];
                        }
                        new_p.data[i][j] -= kh_p;
                    }
                }
                new_p.symmetrize();
                self.covariance = new_p;

                self.inject_error_state(&dx);
                true
            } else {
                false
            }
        } else {
            false
        };

        (pos_accepted, vel_accepted)
    }

    /// Injects estimated error state vector dx into nominal state variables and resets dx = 0.
    fn inject_error_state(&mut self, dx: &[f64; 15]) {
        // 1. Position error injection:
        self.nominal.position.x += dx[0];
        self.nominal.position.y += dx[1];
        self.nominal.position.z += dx[2];

        // 2. Velocity error injection:
        self.nominal.velocity.x += dx[3];
        self.nominal.velocity.y += dx[4];
        self.nominal.velocity.z += dx[5];

        // 3. Attitude error injection (small rotation vector):
        let d_theta = Vector3D::new(dx[6], dx[7], dx[8]);
        let dq = Quaternion {
            w: 1.0,
            x: 0.5 * d_theta.x,
            y: 0.5 * d_theta.y,
            z: 0.5 * d_theta.z,
        };
        self.nominal.orientation = self.nominal.orientation.multiply(&dq);

        // 4. Bias error injection:
        self.nominal.accel_bias.x += dx[9];
        self.nominal.accel_bias.y += dx[10];
        self.nominal.accel_bias.z += dx[11];

        self.nominal.gyro_bias.x += dx[12];
        self.nominal.gyro_bias.y += dx[13];
        self.nominal.gyro_bias.z += dx[14];
    }

    /// Returns estimated 1-sigma standard deviations for [pos_m, vel_m_s, att_rad, ba_m_s2, bg_rad_s].
    pub fn get_std_deviations(&self) -> (Vector3D, Vector3D, Vector3D, Vector3D, Vector3D) {
        let p = &self.covariance.data;
        let pos_std = Vector3D::new(
            p[0][0].max(0.0).sqrt(),
            p[1][1].max(0.0).sqrt(),
            p[2][2].max(0.0).sqrt(),
        );
        let vel_std = Vector3D::new(
            p[3][3].max(0.0).sqrt(),
            p[4][4].max(0.0).sqrt(),
            p[5][5].max(0.0).sqrt(),
        );
        let att_std = Vector3D::new(
            p[6][6].max(0.0).sqrt(),
            p[7][7].max(0.0).sqrt(),
            p[8][8].max(0.0).sqrt(),
        );
        let ba_std = Vector3D::new(
            p[9][9].max(0.0).sqrt(),
            p[10][10].max(0.0).sqrt(),
            p[11][11].max(0.0).sqrt(),
        );
        let bg_std = Vector3D::new(
            p[12][12].max(0.0).sqrt(),
            p[13][13].max(0.0).sqrt(),
            p[14][14].max(0.0).sqrt(),
        );

        (pos_std, vel_std, att_std, ba_std, bg_std)
    }
}
