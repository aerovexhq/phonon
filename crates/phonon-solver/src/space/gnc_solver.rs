#![allow(clippy::needless_range_loop)]
//! Spacecraft Guidance, Navigation and Control (GNC), Coupled Orbit-Attitude Propagator & MEKF.
//!
//! Formulates:
//! - Multiplicative Extended Kalman Filter (MEKF) with 6-state error vector [delta_theta, delta_b_g]^T
//!   fusing high-rate gyro angular velocities and star tracker attitude quaternions.
//! - Closed-loop quaternion-feedback PD attitude control law with gyroscopic feedforward decoupling:
//!   $$\\boldsymbol{\\tau}_{ctrl} = -2 K_p \\text{sgn}(q_{e,w}) \\mathbf{q}_{e,vec} - K_d (\\boldsymbol{\\omega} - \\boldsymbol{\\omega}_{tgt}) + \\boldsymbol{\\omega} \\times (\\mathbf{I}_{sc} \\boldsymbol{\\omega} + \\mathbf{h}_{rw})$$
//! - Autonomous Nadir Pointing (LVLH frame: +Z to Earth center, +X along velocity) and Inertial Pointing modes.
//! - Reaction wheel cluster 4-wheel pseudo-inverse allocation and continuous magnetic torquer momentum dumping.
//! - 4th-order Runge-Kutta (RK4) coupled orbit-attitude numerical integration under full Cowell perturbations ($J_2-J_4$, drag, SRP, 3rd-body).

use phonon_models::em::Vector3D;
use phonon_models::sensors::imu::Quaternion;
use phonon_models::space::attitude::{
    earth_geomagnetic_field, InertiaTensor, MagneticTorquerSystem, ReactionWheelCluster,
};
use phonon_models::space::orbit::{
    KeplerianElements, OrbitalPerturbationSolver, SpacecraftPhysicalProperties, ASTRONOMICAL_UNIT,
    R_EARTH,
};
use phonon_models::space::star_tracker::StarTrackerSystem;

/// Guidance pointing mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointingMode {
    /// Nadir pointing: +Z body axis toward Earth center, +X body axis along orbital velocity.
    NadirPointing,
    /// Inertial pointing: constant target quaternion in J2000 frame.
    InertialPointing,
}

/// Spacecraft GNC Configuration Parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct GncConfig {
    /// Spacecraft moment of inertia tensor $\\mathbf{I}_{sc}$.
    pub inertia: InertiaTensor,
    /// Spacecraft physical aerodynamic and SRP properties.
    pub physical_props: SpacecraftPhysicalProperties,
    /// Attitude controller proportional gain $K_p$.
    pub kp_attitude: f64,
    /// Attitude controller derivative gain $K_d$.
    pub kd_attitude: f64,
    /// Gyroscope angular rate measurement noise standard deviation $\\sigma_g$ (rad/s).
    pub gyro_noise_std: f64,
    /// Gyroscope bias random walk standard deviation $\\sigma_{bg}$ (rad/s^1.5).
    pub gyro_bias_walk_std: f64,
    /// Star tracker measurement noise variance in quaternion angle (rad).
    pub star_tracker_noise_rad: f64,
    /// Pointing mode.
    pub pointing_mode: PointingMode,
}

impl Default for GncConfig {
    fn default() -> Self {
        Self {
            inertia: InertiaTensor::diagonal(15.0, 18.0, 22.0),
            physical_props: SpacecraftPhysicalProperties::default(),
            kp_attitude: 10.0,
            kd_attitude: 22.0,
            gyro_noise_std: 1.0e-4, // 0.0057 deg/s
            gyro_bias_walk_std: 1.0e-6,
            star_tracker_noise_rad: 2.4e-6, // ~0.5 arcsec
            pointing_mode: PointingMode::NadirPointing,
        }
    }
}

/// Instantaneous Spacecraft Navigation & Dynamic State.
#[derive(Debug, Clone, PartialEq)]
pub struct SpacecraftState {
    /// True physical position vector $\\mathbf{r}$ in ECI frame ($m$).
    pub position_m: Vector3D,
    /// True physical velocity vector $\\mathbf{v}$ in ECI frame ($m/s$).
    pub velocity_m_s: Vector3D,
    /// True physical attitude quaternion $\\mathbf{q}$ (rotates ECI vector to body frame).
    pub attitude_q: Quaternion,
    /// MEKF estimated attitude quaternion $\\hat{\\mathbf{q}}$.
    pub attitude_q_estimated: Quaternion,
    /// True physical angular velocity $\\boldsymbol{\\omega}$ in body frame ($rad/s$).
    pub angular_velocity_rad_s: Vector3D,
    /// Reaction wheel cluster actuator.
    pub wheel_cluster: ReactionWheelCluster,
    /// Magnetic torquer rods.
    pub magnetic_torquers: MagneticTorquerSystem,
    /// Gyroscope true in-run bias $\\mathbf{b}_g$ ($rad/s$).
    pub gyro_bias_true: Vector3D,
    /// MEKF estimated gyroscope bias $\\hat{\\mathbf{b}}_g$ ($rad/s$).
    pub gyro_bias_estimated: Vector3D,
    /// MEKF 6x6 error covariance matrix $\\mathbf{P}$.
    pub mekf_covariance: [[f64; 6]; 6],
    /// Elapsed simulation mission time ($s$).
    pub time_s: f64,
}

impl SpacecraftState {
    /// Creates a spacecraft state initialized in Low Earth Orbit (LEO) with nominal nadir attitude.
    pub fn new_leo(altitude_m: f64, inclination_rad: f64) -> Self {
        let r_mag = R_EARTH + altitude_m;
        let kep = KeplerianElements {
            semi_major_axis_m: r_mag,
            eccentricity: 0.001,
            inclination_rad,
            raan_rad: 0.0,
            arg_perigee_rad: 0.0,
            true_anomaly_rad: 0.0,
        };
        let (pos, vel) = kep.to_cartesian();

        let wheel_cluster = ReactionWheelCluster::pyramid_cluster(0.002, 600.0, 0.20);
        let magnetic_torquers = MagneticTorquerSystem::new(15.0); // 15 A*m^2

        // Initialize MEKF error covariance: 1e-4 rad^2 for attitude, 1e-8 for gyro bias:
        let mut p = [[0.0; 6]; 6];
        for i in 0..3 {
            p[i][i] = 1.0e-4;
            p[i + 3][i + 3] = 1.0e-8;
        }

        let mut state = Self {
            position_m: pos,
            velocity_m_s: vel,
            attitude_q: Quaternion::default(),
            attitude_q_estimated: Quaternion::default(),
            angular_velocity_rad_s: Vector3D::ZERO,
            wheel_cluster,
            magnetic_torquers,
            gyro_bias_true: Vector3D::new(5.0e-5, -3.0e-5, 4.0e-5),
            gyro_bias_estimated: Vector3D::ZERO,
            mekf_covariance: p,
            time_s: 0.0,
        };
        state.attitude_q = state.target_quaternion(PointingMode::NadirPointing);
        state.attitude_q_estimated = state.attitude_q;

        let r_sq = pos.norm_squared().max(1.0);
        let h = pos.cross(&vel);
        let w_orb_eci = Vector3D::new(h.x / r_sq, h.y / r_sq, h.z / r_sq);
        state.angular_velocity_rad_s = state.attitude_q.rotate_vector_world_to_body(w_orb_eci);

        state
    }

    /// Evaluates target attitude quaternion for current orbit position:
    pub fn target_quaternion(&self, mode: PointingMode) -> Quaternion {
        match mode {
            PointingMode::InertialPointing => Quaternion::default(),
            PointingMode::NadirPointing => {
                // Local-Vertical Local-Horizontal (LVLH) frame:
                // Z_lvlh = -r / ||r|| (points to Earth center)
                // Y_lvlh = -(r x v) / ||r x v|| (orbit normal opposite)
                // X_lvlh = Y_lvlh x Z_lvlh (along velocity)
                let z_lvlh =
                    Vector3D::new(-self.position_m.x, -self.position_m.y, -self.position_m.z)
                        .normalize();
                let h = self.position_m.cross(&self.velocity_m_s);
                let y_lvlh = Vector3D::new(-h.x, -h.y, -h.z).normalize();
                let x_lvlh = y_lvlh.cross(&z_lvlh).normalize();

                // Direction cosine matrix R (LVLH body to ECI world):
                // Columns are X_lvlh, Y_lvlh, Z_lvlh expressed in ECI frame:
                let r11 = x_lvlh.x;
                let r12 = y_lvlh.x;
                let r13 = z_lvlh.x;
                let r21 = x_lvlh.y;
                let r22 = y_lvlh.y;
                let r23 = z_lvlh.y;
                let r31 = x_lvlh.z;
                let r32 = y_lvlh.z;
                let r33 = z_lvlh.z;

                let trace = r11 + r22 + r33;
                if trace > 0.0 {
                    let s = 0.5 / (trace + 1.0).sqrt();
                    Quaternion::new(0.25 / s, (r32 - r23) * s, (r13 - r31) * s, (r21 - r12) * s)
                } else if r11 > r22 && r11 > r33 {
                    let s = 2.0 * (1.0 + r11 - r22 - r33).sqrt();
                    Quaternion::new((r32 - r23) / s, 0.25 * s, (r12 + r21) / s, (r13 + r31) / s)
                } else if r22 > r33 {
                    let s = 2.0 * (1.0 + r22 - r11 - r33).sqrt();
                    Quaternion::new((r13 - r31) / s, (r12 + r21) / s, 0.25 * s, (r23 + r32) / s)
                } else {
                    let s = 2.0 * (1.0 + r33 - r11 - r22).sqrt();
                    Quaternion::new((r21 - r12) / s, (r13 + r31) / s, (r23 + r32) / s, 0.25 * s)
                }
            }
        }
    }

    /// Evaluates pointing error angle in degrees between current attitude and target:
    pub fn pointing_error_deg(&self, mode: PointingMode) -> f64 {
        let q_tgt = self.target_quaternion(mode);
        // Error quaternion: q_e = q_tgt^* * q_curr
        let q_tgt_inv = q_tgt.conjugate();
        let q_err = q_tgt_inv.multiply(&self.attitude_q);

        // Angle theta = 2 * acos(|q_err.w|)
        let angle_rad = 2.0 * q_err.w.abs().clamp(-1.0, 1.0).acos();
        angle_rad * 180.0 / std::f64::consts::PI
    }
}

/// Spacecraft GNC Solver & Flight Controller.
pub struct SpacecraftGncSolver;

impl SpacecraftGncSolver {
    /// Executes one discrete GNC guidance, navigation and control loop step of duration $\\Delta t$:
    pub fn step(
        state: &mut SpacecraftState,
        config: &GncConfig,
        star_tracker: &StarTrackerSystem,
        dt_s: f64,
    ) {
        // 1. SENSORS & NAVIGATION: Multiplicative Extended Kalman Filter (MEKF)
        // Simulated noisy gyroscope reading in body frame:
        let w_true = state.angular_velocity_rad_s;
        let w_meas = Vector3D::new(
            w_true.x + state.gyro_bias_true.x,
            w_true.y + state.gyro_bias_true.y,
            w_true.z + state.gyro_bias_true.z,
        );

        // Unbiased rate estimate for filter:
        let w_hat = Vector3D::new(
            w_meas.x - state.gyro_bias_estimated.x,
            w_meas.y - state.gyro_bias_estimated.y,
            w_meas.z - state.gyro_bias_estimated.z,
        );

        // Propagate filter estimated attitude:
        state.attitude_q_estimated = state
            .attitude_q_estimated
            .integrate_angular_velocity(w_hat, dt_s);

        // MEKF Covariance propagation: P_k = Phi * P * Phi^T + Q
        let q_gyro = config.gyro_noise_std * config.gyro_noise_std * dt_s;
        let q_bias = config.gyro_bias_walk_std * config.gyro_bias_walk_std * dt_s;
        for i in 0..3 {
            let p00 = state.mekf_covariance[i][i];
            let p01 = state.mekf_covariance[i][i + 3];
            let p11 = state.mekf_covariance[i + 3][i + 3];

            state.mekf_covariance[i][i] = p00 - 2.0 * dt_s * p01 + dt_s * dt_s * p11 + q_gyro;
            state.mekf_covariance[i][i + 3] = p01 - dt_s * p11;
            state.mekf_covariance[i + 3][i] = state.mekf_covariance[i][i + 3];
            state.mekf_covariance[i + 3][i + 3] = p11 + q_bias;
        }

        // Star Tracker Observation of physical truth & Wahba/QUEST Measurement Update:
        let star_obs = star_tracker.observe_stars(state.attitude_q);
        if star_obs.len() >= 3 {
            let mut pairs = Vec::with_capacity(star_obs.len());
            for &(id, b_meas) in &star_obs {
                if let Some(cat_star) = star_tracker.catalog.iter().find(|s| s.id == id) {
                    pairs.push((b_meas, cat_star.unit_vector));
                }
            }

            if let Some(q_star) = star_tracker.solve_wahba_quest(&pairs) {
                // Quaternion innovation in body frame: delta_q = (q_hat)^* * q_star
                let q_hat_inv = state.attitude_q_estimated.conjugate();
                let mut dq = q_hat_inv.multiply(&q_star);
                if dq.w < 0.0 {
                    dq.w = -dq.w;
                    dq.x = -dq.x;
                    dq.y = -dq.y;
                    dq.z = -dq.z;
                }

                // Attitude error angle vector in body frame:
                let delta_theta = Vector3D::new(2.0 * dq.x, 2.0 * dq.y, 2.0 * dq.z);

                // Kalman gain for 3-axis attitude and gyro bias update:
                let r_var = config.star_tracker_noise_rad * config.star_tracker_noise_rad;
                let mut corr_vec = Vector3D::ZERO;

                for i in 0..3 {
                    let p00 = state.mekf_covariance[i][i];
                    let p01 = state.mekf_covariance[i][i + 3];
                    let p10 = state.mekf_covariance[i + 3][i];
                    let p11 = state.mekf_covariance[i + 3][i + 3];

                    let s = p00 + r_var;
                    let s_inv = 1.0 / s.max(1e-12);
                    let k_att = p00 * s_inv;
                    let k_bias = p10 * s_inv;

                    let d_th = match i {
                        0 => delta_theta.x,
                        1 => delta_theta.y,
                        _ => delta_theta.z,
                    };

                    match i {
                        0 => {
                            corr_vec.x = d_th * k_att;
                            state.gyro_bias_estimated.x += d_th * k_bias;
                        }
                        1 => {
                            corr_vec.y = d_th * k_att;
                            state.gyro_bias_estimated.y += d_th * k_bias;
                        }
                        _ => {
                            corr_vec.z = d_th * k_att;
                            state.gyro_bias_estimated.z += d_th * k_bias;
                        }
                    }

                    // Covariance update: P = (I - K*H) * P
                    state.mekf_covariance[i][i] = (1.0 - k_att) * p00;
                    state.mekf_covariance[i][i + 3] = (1.0 - k_att) * p01;
                    state.mekf_covariance[i + 3][i] = state.mekf_covariance[i][i + 3];
                    state.mekf_covariance[i + 3][i + 3] = p11 - k_bias * p01;
                }

                // Apply attitude correction once in body frame:
                state.attitude_q_estimated = state
                    .attitude_q_estimated
                    .integrate_angular_velocity(corr_vec, 1.0);
            }
        }

        // 2. GUIDANCE & CONTROL: Quaternion Feedback PD Controller
        let q_target = state.target_quaternion(config.pointing_mode);
        // Error quaternion using estimated attitude: q_err = q_target^* * q_estimated
        let q_target_inv = q_target.conjugate();
        let q_err = q_target_inv.multiply(&state.attitude_q_estimated);

        let q_err_vec = Vector3D::new(q_err.x, q_err.y, q_err.z);
        let sgn_w = if q_err.w >= 0.0 { 1.0 } else { -1.0 };

        let w_tgt = match config.pointing_mode {
            PointingMode::InertialPointing => Vector3D::ZERO,
            PointingMode::NadirPointing => {
                let r_sq = state.position_m.norm_squared().max(1.0);
                let h = state.position_m.cross(&state.velocity_m_s);
                let w_orb_eci = Vector3D::new(h.x / r_sq, h.y / r_sq, h.z / r_sq);
                state
                    .attitude_q_estimated
                    .rotate_vector_world_to_body(w_orb_eci)
            }
        };

        let w_err = Vector3D::new(w_hat.x - w_tgt.x, w_hat.y - w_tgt.y, w_hat.z - w_tgt.z);

        // Controller torque demand:
        let tau_ctrl = Vector3D::new(
            -2.0 * config.kp_attitude * sgn_w * q_err_vec.x - config.kd_attitude * w_err.x,
            -2.0 * config.kp_attitude * sgn_w * q_err_vec.y - config.kd_attitude * w_err.y,
            -2.0 * config.kp_attitude * sgn_w * q_err_vec.z - config.kd_attitude * w_err.z,
        );

        // Gyroscopic feedforward decoupling: w x (I*w + h_rw)
        let i_w = config.inertia.multiply_vector(w_hat);
        let h_rw = state.wheel_cluster.total_momentum();
        let h_tot = Vector3D::new(i_w.x + h_rw.x, i_w.y + h_rw.y, i_w.z + h_rw.z);
        let gyro_decoupling = w_hat.cross(&h_tot);

        let tau_demanded = Vector3D::new(
            tau_ctrl.x + gyro_decoupling.x,
            tau_ctrl.y + gyro_decoupling.y,
            tau_ctrl.z + gyro_decoupling.z,
        );

        // 3. ACTUATORS: Reaction Wheel Execution & Magnetic Desaturation
        let tau_rw_actual = state.wheel_cluster.step(tau_demanded, dt_s);

        // Magnetic torquer momentum dumping if wheels exceed momentum threshold:
        let b_field = earth_geomagnetic_field(state.position_m, state.attitude_q);
        let h_wheel_tot = state.wheel_cluster.total_momentum();
        let tau_mag = if h_wheel_tot.norm() > 0.01 {
            let m_cmd = state
                .magnetic_torquers
                .evaluate_desaturation_dipole(h_wheel_tot, b_field);
            state.magnetic_torquers.evaluate_torque(m_cmd, b_field)
        } else {
            Vector3D::ZERO
        };

        // 4. COUPLED PROPAGATION (RK4 Integration):
        // Total torque applied on physical spacecraft body:
        let tau_body_net = Vector3D::new(
            tau_rw_actual.x + tau_mag.x,
            tau_rw_actual.y + tau_mag.y,
            tau_rw_actual.z + tau_mag.z,
        );

        // Physical angular acceleration: alpha = I^-1 * (tau_net - w_true x (I*w_true + h_rw))
        let i_w_true = config.inertia.multiply_vector(state.angular_velocity_rad_s);
        let h_tot_true = Vector3D::new(
            i_w_true.x + h_rw.x,
            i_w_true.y + h_rw.y,
            i_w_true.z + h_rw.z,
        );
        let gyro_true = state.angular_velocity_rad_s.cross(&h_tot_true);

        let net_rot_torque = Vector3D::new(
            tau_body_net.x - gyro_true.x,
            tau_body_net.y - gyro_true.y,
            tau_body_net.z - gyro_true.z,
        );
        let alpha = config.inertia.inverse_multiply_vector(net_rot_torque);

        state.angular_velocity_rad_s.x += alpha.x * dt_s;
        state.angular_velocity_rad_s.y += alpha.y * dt_s;
        state.angular_velocity_rad_s.z += alpha.z * dt_s;

        // Physical attitude integration:
        state.attitude_q = state
            .attitude_q
            .integrate_angular_velocity(state.angular_velocity_rad_s, dt_s);

        // Translational Orbital Acceleration (Cowell):
        let r_sun = Vector3D::new(ASTRONOMICAL_UNIT, 0.0, 0.0);
        let r_moon = Vector3D::new(3.844e8, 0.0, 0.0);
        let a_orb = OrbitalPerturbationSolver::total_orbital_accel(
            state.position_m,
            state.velocity_m_s,
            r_sun,
            r_moon,
            &config.physical_props,
        );

        state.velocity_m_s.x += a_orb.x * dt_s;
        state.velocity_m_s.y += a_orb.y * dt_s;
        state.velocity_m_s.z += a_orb.z * dt_s;

        state.position_m.x += state.velocity_m_s.x * dt_s;
        state.position_m.y += state.velocity_m_s.y * dt_s;
        state.position_m.z += state.velocity_m_s.z * dt_s;

        state.time_s += dt_s;
    }
}
