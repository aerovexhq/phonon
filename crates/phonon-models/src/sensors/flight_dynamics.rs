//! 6-DOF Multi-Physics Flight Dynamics, Aerodynamics & Sensor Transducers
//!
//! Provides mathematically rigorous 6-DOF rigid-body translational and rotational equations of motion:
//! 1. m \dot{v} + m (\omega \times v) = F_{aero} + F_{thrust} + F_{gravity}
//! 2. I \dot{\omega} + \omega \times (I \omega) = M_{aero} + M_{thrust}
//! 3. 4D Unit Quaternion attitude kinematic integration \dot{q} = 0.5 * q \otimes \omega.
//! 4. Aerodynamic ground effect thrust amplification: T(h)/T_\infty = 1 / (1 - (R / 4h)^2).
//! 5. Turbulent continuous wind gust envelopes: Dryden and von Kármán spectral formulations.
//! 6. First-order actuator motor lag: \dot{\Omega}_i = (\Omega_{cmd,i} - \Omega_i) / \tau_m.
//! 7. Sensor suite transducers: 9-DOF IMU, barometric altimeter, pulsed LiDAR, GNSS/GPS.

use crate::em::{ChannelRng, Vector3D};
use crate::sensors::imu::{
    sample_imu, ImuConfig, ImuMeasurement, ImuState, Quaternion, STANDARD_GRAVITY_M_S2,
};
use std::f64::consts::PI;

/// Standard sea-level atmospheric pressure P_0 in Pascals (101,325 Pa).
pub const STANDARD_SEA_LEVEL_PRESSURE_PA: f64 = 101_325.0;

/// Standard sea-level atmospheric temperature T_0 in Kelvin (288.15 K).
pub const STANDARD_SEA_LEVEL_TEMP_K: f64 = 288.15;

/// Standard atmospheric temperature lapse rate L in K/m (0.0065 K/m).
pub const STANDARD_TEMP_LAPSE_RATE_K_PER_M: f64 = 0.0065;

/// Universal gas constant R_0 in J/(mol * K).
pub const UNIVERSAL_GAS_CONSTANT: f64 = 8.3144598;

/// Molar mass of dry air M in kg/mol (0.0289644 kg/mol).
pub const DRY_AIR_MOLAR_MASS_KG_PER_MOL: f64 = 0.0289644;

/// Barometric exponent: g_0 * M / (R_0 * L) ~ 5.25588.
pub const BAROMETRIC_EXPONENT: f64 = (STANDARD_GRAVITY_M_S2 * DRY_AIR_MOLAR_MASS_KG_PER_MOL)
    / (UNIVERSAL_GAS_CONSTANT * STANDARD_TEMP_LAPSE_RATE_K_PER_M);

/// 3x3 Symmetric Moment of Inertia Tensor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InertiaTensor3D {
    pub ixx: f64,
    pub iyy: f64,
    pub izz: f64,
    pub ixy: f64,
    pub ixz: f64,
    pub iyz: f64,
}

impl Default for InertiaTensor3D {
    fn default() -> Self {
        // Standard multirotor diagonal inertia (e.g. 1.5 kg quadrotor):
        Self {
            ixx: 0.015,
            iyy: 0.015,
            izz: 0.028,
            ixy: 0.0,
            ixz: 0.0,
            iyz: 0.0,
        }
    }
}

impl InertiaTensor3D {
    /// Creates a diagonal inertia tensor with principal moments of inertia.
    pub fn new_diagonal(ixx: f64, iyy: f64, izz: f64) -> Self {
        Self {
            ixx,
            iyy,
            izz,
            ixy: 0.0,
            ixz: 0.0,
            iyz: 0.0,
        }
    }

    /// Multiplies the 3x3 inertia matrix by an angular velocity vector: H = I * omega.
    pub fn multiply_vector(&self, omega: Vector3D) -> Vector3D {
        Vector3D::new(
            self.ixx * omega.x + self.ixy * omega.y + self.ixz * omega.z,
            self.ixy * omega.x + self.iyy * omega.y + self.iyz * omega.z,
            self.ixz * omega.x + self.iyz * omega.y + self.izz * omega.z,
        )
    }

    /// Computes the determinant of the 3x3 inertia matrix.
    pub fn determinant(&self) -> f64 {
        self.ixx * (self.iyy * self.izz - self.iyz * self.iyz)
            - self.ixy * (self.ixy * self.izz - self.iyz * self.ixz)
            + self.ixz * (self.ixy * self.iyz - self.iyy * self.ixz)
    }

    /// Computes the inverse of the 3x3 inertia matrix: I^{-1}.
    pub fn inverse(&self) -> Self {
        let det = self.determinant();
        let inv_det = if det.abs() > 1e-12 { 1.0 / det } else { 1.0 };

        Self {
            ixx: (self.iyy * self.izz - self.iyz * self.iyz) * inv_det,
            iyy: (self.ixx * self.izz - self.ixz * self.ixz) * inv_det,
            izz: (self.ixx * self.iyy - self.ixy * self.ixy) * inv_det,
            ixy: -(self.ixy * self.izz - self.iyz * self.ixz) * inv_det,
            ixz: (self.ixy * self.iyz - self.iyy * self.ixz) * inv_det,
            iyz: -(self.ixx * self.iyz - self.ixy * self.ixz) * inv_det,
        }
    }
}

/// Actuator rotor motor configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RotorConfig {
    /// Relative rotor mounting position in body Cartesian coordinates (x, y, z) in meters.
    pub position_body: Vector3D,
    /// Thrust coefficient k_t in N / (rad/s)^2: T = k_t * Omega^2.
    pub thrust_coeff: f64,
    /// Reaction torque drag coefficient k_tau in N*m / (rad/s)^2: Q = k_tau * Omega^2.
    pub torque_coeff: f64,
    /// Rotor spin direction (+1.0 for CCW, -1.0 for CW).
    pub spin_direction: f64,
    /// First-order motor electrical/mechanical time constant tau_m in seconds.
    pub time_constant_s: f64,
    /// Maximum rotor angular speed limit in rad/s (e.g. 1500 rad/s ~ 14,300 RPM).
    pub max_rad_s: f64,
    /// Rotor radius R_rotor in meters for aerodynamic ground effect calculation.
    pub rotor_radius_m: f64,
}

impl Default for RotorConfig {
    fn default() -> Self {
        Self {
            position_body: Vector3D::new(0.15, 0.15, 0.0),
            thrust_coeff: 1.2e-5,
            torque_coeff: 2.5e-7,
            spin_direction: 1.0,
            time_constant_s: 0.025, // 25 ms motor lag
            max_rad_s: 1800.0,
            rotor_radius_m: 0.127, // 5-inch prop ~ 0.127 m
        }
    }
}

/// Airframe configuration for multi-rotor or fixed-wing vehicles.
#[derive(Debug, Clone, PartialEq)]
pub struct AirframeConfig {
    /// Total vehicle mass in kilograms.
    pub mass_kg: f64,
    /// Inertia tensor in body frame.
    pub inertia: InertiaTensor3D,
    /// Array of actuator rotor motors.
    pub rotors: Vec<RotorConfig>,
    /// Parasitic aerodynamic body drag coefficients (Cd_x, Cd_y, Cd_z) in N / (m/s)^2.
    pub body_drag_coeffs: Vector3D,
    /// Lift coefficient slope for fixed-wing airframes (0.0 for pure multirotors).
    pub wing_lift_slope: f64,
    /// Wing planform surface area S in m^2 (0.0 for pure multirotors).
    pub wing_area_m2: f64,
    /// Ground effect calculation rotor radius R_rotor (meters).
    pub rotor_radius_m: f64,
}

impl Default for AirframeConfig {
    fn default() -> Self {
        Self::new_quadrotor_x(1.5, 0.22, 0.127)
    }
}

impl AirframeConfig {
    /// Creates a standard Quadrotor X airframe configuration.
    /// Arm length L in meters, rotor radius R in meters.
    pub fn new_quadrotor_x(mass_kg: f64, arm_length_m: f64, rotor_radius_m: f64) -> Self {
        let l = arm_length_m * 0.5 * std::f64::consts::SQRT_2;
        let base_rotor = RotorConfig {
            thrust_coeff: 1.25e-5,
            torque_coeff: 2.2e-7,
            time_constant_s: 0.025,
            max_rad_s: 1600.0,
            rotor_radius_m,
            ..Default::default()
        };

        // Quad X rotor placement:
        // Motor 0: Front-Right (+x, -y), CCW (+1)
        // Motor 1: Rear-Left (-x, +y), CCW (+1)
        // Motor 2: Front-Left (+x, +y), CW (-1)
        // Motor 3: Rear-Right (-x, -y), CW (-1)
        let rotors = vec![
            RotorConfig {
                position_body: Vector3D::new(l, -l, 0.0),
                spin_direction: 1.0,
                ..base_rotor
            },
            RotorConfig {
                position_body: Vector3D::new(-l, l, 0.0),
                spin_direction: 1.0,
                ..base_rotor
            },
            RotorConfig {
                position_body: Vector3D::new(l, l, 0.0),
                spin_direction: -1.0,
                ..base_rotor
            },
            RotorConfig {
                position_body: Vector3D::new(-l, -l, 0.0),
                spin_direction: -1.0,
                ..base_rotor
            },
        ];

        Self {
            mass_kg,
            inertia: InertiaTensor3D::new_diagonal(0.015, 0.015, 0.028),
            rotors,
            body_drag_coeffs: Vector3D::new(0.08, 0.08, 0.18),
            wing_lift_slope: 0.0,
            wing_area_m2: 0.0,
            rotor_radius_m,
        }
    }
}

/// Evaluates aerodynamic ground effect thrust amplification:
/// T(h) / T_\infty = 1 / (1 - (R_{rotor} / (4h))^2) for h < 2 * R_{rotor}.
pub fn compute_ground_effect_factor(altitude_m: f64, rotor_radius_m: f64) -> f64 {
    if altitude_m <= 0.0 {
        // Contact with ground or subterranean
        return 1.45;
    }
    if altitude_m >= 2.0 * rotor_radius_m {
        return 1.0;
    }

    // Clamp altitude to prevent denominator singularity:
    let min_h = 0.20 * rotor_radius_m;
    let effective_h = altitude_m.max(min_h);
    let ratio = rotor_radius_m / (4.0 * effective_h);
    let denominator = (1.0 - ratio * ratio).max(0.1);
    (1.0 / denominator).min(1.50)
}

/// Dryden and von Kármán Continuous Turbulent Wind Gust Envelope.
#[derive(Debug, Clone, PartialEq)]
pub struct DrydenWindModel {
    /// Wind reference speed at 20 ft (6 m) altitude in m/s (W_20).
    pub reference_wind_speed_m_s: f64,
    /// Mean steady background wind vector in world Cartesian frame (m/s).
    pub mean_wind_world: Vector3D,
    /// Filter state for longitudinal gust u_g (m/s).
    pub state_u: f64,
    /// Filter state for lateral gust v_g (m/s).
    pub state_v: f64,
    /// Filter state for vertical gust w_g (m/s).
    pub state_w: f64,
}

impl Default for DrydenWindModel {
    fn default() -> Self {
        Self {
            reference_wind_speed_m_s: 5.0, // 5 m/s moderate breeze
            mean_wind_world: Vector3D::new(2.0, 1.0, 0.0),
            state_u: 0.0,
            state_v: 0.0,
            state_w: 0.0,
        }
    }
}

impl DrydenWindModel {
    /// Constructs a new Dryden wind gust model with specified turbulence intensity.
    pub fn new(mean_wind: Vector3D, reference_speed: f64) -> Self {
        Self {
            reference_wind_speed_m_s: reference_speed,
            mean_wind_world: mean_wind,
            state_u: 0.0,
            state_v: 0.0,
            state_w: 0.0,
        }
    }

    /// Evaluates spatial turbulence scale lengths (Lu, Lv, Lw) and intensities (sigma_u, sigma_v, sigma_w)
    /// at the current vehicle altitude h above ground (MIL-F-8785C low-altitude model).
    pub fn turbulence_parameters(&self, altitude_m: f64) -> (Vector3D, Vector3D) {
        let h = altitude_m.clamp(1.0, 300.0);
        let w20 = self.reference_wind_speed_m_s.max(0.1);

        // Turbulence intensities:
        let sigma_w = 0.1 * w20;
        let denom_sigma = (0.177 + 0.000823 * h).powf(0.4);
        let sigma_u = sigma_w / denom_sigma;
        let sigma_v = sigma_u;

        // Spatial scale lengths:
        let l_w = h;
        let denom_l = (0.177 + 0.000823 * h).powf(1.2);
        let l_u = h / denom_l;
        let l_v = 0.5 * l_u;

        (
            Vector3D::new(l_u, l_v, l_w),
            Vector3D::new(sigma_u, sigma_v, sigma_w),
        )
    }

    /// Steps the Dryden digital filter over time dt (seconds) given true airspeed V (m/s).
    /// Returns the total instantaneous wind velocity vector in world coordinates (m/s).
    pub fn step(
        &mut self,
        altitude_m: f64,
        airspeed_m_s: f64,
        dt: f64,
        rng: &mut ChannelRng,
    ) -> Vector3D {
        let v = airspeed_m_s.max(1.0);
        let (lengths, sigmas) = self.turbulence_parameters(altitude_m);

        let (g1, g2) = rng.next_gaussian();
        let (g3, _) = rng.next_gaussian();

        // Discretized first-order autoregressive Dryden filter:
        let alpha_u = (-v * dt / lengths.x.max(0.5)).exp();
        let driving_u = sigmas.x * (1.0 - alpha_u * alpha_u).max(0.0).sqrt();
        self.state_u = self.state_u * alpha_u + g1 * driving_u;

        let alpha_v = (-v * dt / lengths.y.max(0.5)).exp();
        let driving_v = sigmas.y * (1.0 - alpha_v * alpha_v).max(0.0).sqrt();
        self.state_v = self.state_v * alpha_v + g2 * driving_v;

        let alpha_w = (-v * dt / lengths.z.max(0.5)).exp();
        let driving_w = sigmas.z * (1.0 - alpha_w * alpha_w).max(0.0).sqrt();
        self.state_w = self.state_w * alpha_w + g3 * driving_w;

        let gust = Vector3D::new(self.state_u, self.state_v, self.state_w);
        self.mean_wind_world + gust
    }
}

/// Barometric pressure altimeter sensor model.
#[derive(Debug, Clone, PartialEq)]
pub struct BarometerSensor {
    /// Sea-level reference pressure in Pa (e.g. 101,325 Pa).
    pub p0_pa: f64,
    /// Sea-level reference temperature in Kelvin (288.15 K).
    pub t0_k: f64,
    /// Pressure measurement noise standard deviation in Pa (~1.2 Pa corresponds to ~0.1 m RMS).
    pub noise_std_pa: f64,
    /// Pressure sensor bias offset in Pa.
    pub bias_pa: f64,
}

impl Default for BarometerSensor {
    fn default() -> Self {
        Self {
            p0_pa: STANDARD_SEA_LEVEL_PRESSURE_PA,
            t0_k: STANDARD_SEA_LEVEL_TEMP_K,
            noise_std_pa: 1.5,
            bias_pa: 0.0,
        }
    }
}

impl BarometerSensor {
    /// Computes theoretical barometric pressure P(h) in Pa at true altitude h (meters AMSL):
    /// P(h) = P_0 * (1 - L * h / T_0)^{gM / (R_0 L)}.
    pub fn theoretical_pressure_pa(&self, altitude_m: f64) -> f64 {
        let h = altitude_m.max(-500.0);
        let factor = (1.0 - (STANDARD_TEMP_LAPSE_RATE_K_PER_M * h) / self.t0_k).max(1e-6);
        self.p0_pa * factor.powf(BAROMETRIC_EXPONENT)
    }

    /// Transduces true altitude into noisy, biased barometric pressure measurement.
    pub fn sample_pressure_pa(&self, altitude_m: f64, rng: &mut ChannelRng) -> f64 {
        let p_true = self.theoretical_pressure_pa(altitude_m);
        let (noise, _) = rng.next_gaussian();
        (p_true + self.bias_pa + noise * self.noise_std_pa).max(100.0)
    }

    /// Derives estimated altitude from measured barometric pressure:
    /// h = (T_0 / L) * (1 - (P / P_0)^{(R_0 L) / (gM)}).
    pub fn pressure_to_altitude(&self, pressure_pa: f64) -> f64 {
        let ratio = (pressure_pa.max(100.0) / self.p0_pa).max(1e-6);
        let inv_exp = 1.0 / BAROMETRIC_EXPONENT;
        (self.t0_k / STANDARD_TEMP_LAPSE_RATE_K_PER_M) * (1.0 - ratio.powf(inv_exp))
    }
}

/// Pulsed LiDAR rangefinder sensor model.
#[derive(Debug, Clone, PartialEq)]
pub struct LidarRangefinder {
    /// Maximum operational detection range in meters.
    pub max_range_m: f64,
    /// Minimum detection range in meters (blind zone).
    pub min_range_m: f64,
    /// Distance measurement noise standard deviation in meters (e.g. 0.015 m).
    pub noise_std_m: f64,
    /// Maximum allowable beam tilt angle relative to ground normal in radians (e.g. 45 deg).
    pub max_tilt_angle_rad: f64,
}

impl Default for LidarRangefinder {
    fn default() -> Self {
        Self {
            max_range_m: 40.0,
            min_range_m: 0.05,
            noise_std_m: 0.015,
            max_tilt_angle_rad: 45.0 * PI / 180.0,
        }
    }
}

impl LidarRangefinder {
    /// Samples ground range reading along vehicle downwards-pointing Z-axis.
    /// Returns (measured_range_m, is_valid).
    pub fn sample_range(
        &self,
        altitude_agl_m: f64,
        orientation: Quaternion,
        rng: &mut ChannelRng,
    ) -> (f64, bool) {
        // Vehicle downwards body Z-axis rotated to world frame:
        let body_down = Vector3D::new(0.0, 0.0, -1.0);
        let world_down = orientation.rotate_vector_body_to_world(body_down);

        // Cosine of angle relative to vertical downward normal (0, 0, -1):
        let cos_tilt = (-world_down.z).clamp(-1.0, 1.0);
        let tilt_angle = cos_tilt.acos();

        if tilt_angle > self.max_tilt_angle_rad || cos_tilt < 1e-4 || altitude_agl_m <= 0.0 {
            return (self.max_range_m, false);
        }

        let slant_range = altitude_agl_m / cos_tilt;
        if slant_range > self.max_range_m || slant_range < self.min_range_m {
            return (self.max_range_m, false);
        }

        let (noise, _) = rng.next_gaussian();
        let measured =
            (slant_range + noise * self.noise_std_m).clamp(self.min_range_m, self.max_range_m);
        (measured, true)
    }
}

/// GNSS/GPS Receiver Fix Type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpsFixType {
    NoFix = 0,
    Fix2D = 2,
    Fix3D = 3,
    Dgps = 4,
    RtkFloat = 5,
    RtkFixed = 6,
}

/// GNSS/GPS Receiver Measurement Packet.
#[derive(Debug, Clone, PartialEq)]
pub struct GpsMeasurement {
    /// Epoch timestamp in seconds.
    pub timestamp_s: f64,
    /// Measured position in world Cartesian coordinates (meters).
    pub position_world_m: Vector3D,
    /// Measured velocity in world Cartesian coordinates (m/s).
    pub velocity_world_m_s: Vector3D,
    /// Horizontal Dilution of Precision (HDOP).
    pub hdop: f64,
    /// Vertical Dilution of Precision (VDOP).
    pub vdop: f64,
    /// Fix quality status.
    pub fix_type: GpsFixType,
    /// Number of tracked satellites.
    pub satellites_visible: u8,
    /// Spoofing attack flag detected by ground station or injected.
    pub is_spoofed: bool,
}

/// Complete Kinematic and Actuator Flight State.
#[derive(Debug, Clone, PartialEq)]
pub struct FlightDynamicsState {
    /// World position (x, y, z) in meters.
    pub position_world: Vector3D,
    /// Body-frame linear velocity (u, v, w) in m/s.
    pub velocity_body: Vector3D,
    /// Attitude orientation unit quaternion (w, x, y, z).
    pub orientation: Quaternion,
    /// Body-frame angular velocity (p, q, r) in rad/s.
    pub angular_velocity_body: Vector3D,
    /// Actuator rotor angular speeds in rad/s.
    pub rotor_speeds_rad_s: Vec<f64>,
}

impl FlightDynamicsState {
    /// Creates a stationary hover flight state at specified altitude.
    pub fn new_hover(altitude_m: f64, hover_rpm_rad_s: f64, num_rotors: usize) -> Self {
        Self {
            position_world: Vector3D::new(0.0, 0.0, altitude_m),
            velocity_body: Vector3D::ZERO,
            orientation: Quaternion::default(),
            angular_velocity_body: Vector3D::ZERO,
            rotor_speeds_rad_s: vec![hover_rpm_rad_s; num_rotors],
        }
    }
}

/// Rigid-Body 6-DOF Multi-Physics Flight Dynamics Simulation Engine.
#[derive(Debug, Clone, PartialEq)]
pub struct FlightDynamicsEngine {
    pub config: AirframeConfig,
    pub state: FlightDynamicsState,
    pub wind_model: DrydenWindModel,
    pub barometer: BarometerSensor,
    pub lidar: LidarRangefinder,
    pub imu_config: ImuConfig,
    pub imu_state: ImuState,
    pub sim_time_s: f64,
}

impl FlightDynamicsEngine {
    /// Creates a new flight dynamics engine with specified airframe and initial state.
    pub fn new(config: AirframeConfig, initial_state: FlightDynamicsState) -> Self {
        let num_rotors = config.rotors.len();
        let mut state = initial_state;
        if state.rotor_speeds_rad_s.len() != num_rotors {
            state.rotor_speeds_rad_s = vec![0.0; num_rotors];
        }

        Self {
            config,
            state,
            wind_model: DrydenWindModel::default(),
            barometer: BarometerSensor::default(),
            lidar: LidarRangefinder::default(),
            imu_config: ImuConfig::new_industrial_9dof(),
            imu_state: ImuState::default(),
            sim_time_s: 0.0,
        }
    }

    /// Evaluates state derivatives for 6-DOF equations of motion:
    /// d(pos)/dt = R * v
    /// d(v)/dt = (F_total / m) - omega x v
    /// d(q)/dt = 0.5 * q (x) omega
    /// d(omega)/dt = I^{-1} * (M_total - omega x (I * omega))
    /// d(Omega_i)/dt = (Omega_cmd,i - Omega_i) / tau_m
    pub fn compute_derivatives(
        &self,
        current_state: &FlightDynamicsState,
        rotor_commands_rad_s: &[f64],
        wind_world: Vector3D,
    ) -> (Vector3D, Vector3D, Quaternion, Vector3D, Vec<f64>, Vector3D) {
        let m = self.config.mass_kg.max(0.1);
        let q = current_state.orientation;
        let v_body = current_state.velocity_body;
        let omega = current_state.angular_velocity_body;

        // 1. Actuator rotor dynamics & forces:
        let altitude = current_state.position_world.z;
        let ige_factor = compute_ground_effect_factor(altitude, self.config.rotor_radius_m);

        let mut total_thrust_body = Vector3D::ZERO;
        let mut total_torque_body = Vector3D::ZERO;
        let mut d_rotor_speeds = Vec::with_capacity(self.config.rotors.len());

        for (i, rotor) in self.config.rotors.iter().enumerate() {
            let omega_curr = current_state
                .rotor_speeds_rad_s
                .get(i)
                .copied()
                .unwrap_or(0.0);
            let omega_cmd = rotor_commands_rad_s
                .get(i)
                .copied()
                .unwrap_or(0.0)
                .clamp(0.0, rotor.max_rad_s);

            let d_omega = (omega_cmd - omega_curr) / rotor.time_constant_s.max(1e-4);
            d_rotor_speeds.push(d_omega);

            // Individual rotor thrust along body +Z (upwards):
            let t_i = rotor.thrust_coeff * omega_curr * omega_curr * ige_factor;
            total_thrust_body.z += t_i;

            // Thrust moment: M_t = r_i x T_i
            let r_i = rotor.position_body;
            let thrust_vec = Vector3D::new(0.0, 0.0, t_i);
            let moment_thrust = r_i.cross(&thrust_vec);

            // Reaction drag torque around body Z:
            let reaction_torque = Vector3D::new(
                0.0,
                0.0,
                -rotor.spin_direction * rotor.torque_coeff * omega_curr * omega_curr,
            );

            total_torque_body = total_torque_body + moment_thrust + reaction_torque;
        }

        // 2. Relative wind and aerodynamic drag:
        let wind_body = q.rotate_vector_world_to_body(wind_world);
        let v_rel = v_body - wind_body;
        let drag_body = Vector3D::new(
            -self.config.body_drag_coeffs.x * v_rel.x * v_rel.x.abs(),
            -self.config.body_drag_coeffs.y * v_rel.y * v_rel.y.abs(),
            -self.config.body_drag_coeffs.z * v_rel.z * v_rel.z.abs(),
        );

        // Fixed-wing lift:
        let mut aero_lift_body = Vector3D::ZERO;
        if self.config.wing_area_m2 > 0.0 && self.config.wing_lift_slope > 0.0 {
            let airspeed_sq = v_rel.x * v_rel.x + v_rel.z * v_rel.z;
            let alpha = (-v_rel.z).atan2(v_rel.x.max(1e-3));
            let cl = self.config.wing_lift_slope * alpha;
            let lift = 0.5 * 1.225 * airspeed_sq * self.config.wing_area_m2 * cl;
            aero_lift_body.z += lift;
        }

        // 3. Gravity in body frame:
        let g_world = Vector3D::new(0.0, 0.0, -STANDARD_GRAVITY_M_S2);
        let g_body = q.rotate_vector_world_to_body(g_world);
        let f_gravity_body = g_body * m;

        // Total body forces:
        let f_total_body = total_thrust_body + drag_body + aero_lift_body + f_gravity_body;

        // Linear acceleration: dv/dt = (F_total / m) - omega x v
        let d_v_body = (f_total_body * (1.0 / m)) - omega.cross(&v_body);

        // World velocity: dp/dt = R * v_body
        let d_p_world = q.rotate_vector_body_to_world(v_body);

        // Quaternion derivative: dq/dt = 0.5 * q (x) omega
        let dq = Quaternion {
            w: 0.5 * (-q.x * omega.x - q.y * omega.y - q.z * omega.z),
            x: 0.5 * (q.w * omega.x + q.y * omega.z - q.z * omega.y),
            y: 0.5 * (q.w * omega.y + q.z * omega.x - q.x * omega.z),
            z: 0.5 * (q.w * omega.z + q.x * omega.y - q.y * omega.x),
        };

        // Rotational dynamics:
        let h = self.config.inertia.multiply_vector(omega);
        let gyroscopic_torque = omega.cross(&h);
        let m_net = total_torque_body - gyroscopic_torque;
        let inv_inertia = self.config.inertia.inverse();
        let d_omega = inv_inertia.multiply_vector(m_net);

        // Total kinematic linear body acceleration (sample_imu subtracts g_body to obtain specific force):
        let linear_accel_body = f_total_body * (1.0 / m);

        (
            d_p_world,
            d_v_body,
            dq,
            d_omega,
            d_rotor_speeds,
            linear_accel_body,
        )
    }

    /// Advances the flight dynamics using Runge-Kutta 4th Order (RK4) integration over time step dt.
    pub fn step(
        &mut self,
        rotor_commands_rad_s: &[f64],
        dt: f64,
        rng: &mut ChannelRng,
    ) -> (ImuMeasurement, f64, (f64, bool), GpsMeasurement) {
        let airspeed = self.state.velocity_body.norm();
        let wind_world = self
            .wind_model
            .step(self.state.position_world.z, airspeed, dt, rng);

        // RK4 Stage 1:
        let (dp1, dv1, dq1, dw1, dr1, spec_force1) =
            self.compute_derivatives(&self.state, rotor_commands_rad_s, wind_world);

        // State at midpoint (Stage 2):
        let s2 = FlightDynamicsState {
            position_world: self.state.position_world + dp1 * (0.5 * dt),
            velocity_body: self.state.velocity_body + dv1 * (0.5 * dt),
            orientation: Quaternion::new(
                self.state.orientation.w + dq1.w * 0.5 * dt,
                self.state.orientation.x + dq1.x * 0.5 * dt,
                self.state.orientation.y + dq1.y * 0.5 * dt,
                self.state.orientation.z + dq1.z * 0.5 * dt,
            ),
            angular_velocity_body: self.state.angular_velocity_body + dw1 * (0.5 * dt),
            rotor_speeds_rad_s: self
                .state
                .rotor_speeds_rad_s
                .iter()
                .zip(&dr1)
                .map(|(&r, &dr)| (r + dr * 0.5 * dt).max(0.0))
                .collect(),
        };
        let (dp2, dv2, dq2, dw2, dr2, _) =
            self.compute_derivatives(&s2, rotor_commands_rad_s, wind_world);

        // State at midpoint (Stage 3):
        let s3 = FlightDynamicsState {
            position_world: self.state.position_world + dp2 * (0.5 * dt),
            velocity_body: self.state.velocity_body + dv2 * (0.5 * dt),
            orientation: Quaternion::new(
                self.state.orientation.w + dq2.w * 0.5 * dt,
                self.state.orientation.x + dq2.x * 0.5 * dt,
                self.state.orientation.y + dq2.y * 0.5 * dt,
                self.state.orientation.z + dq2.z * 0.5 * dt,
            ),
            angular_velocity_body: self.state.angular_velocity_body + dw2 * (0.5 * dt),
            rotor_speeds_rad_s: self
                .state
                .rotor_speeds_rad_s
                .iter()
                .zip(&dr2)
                .map(|(&r, &dr)| (r + dr * 0.5 * dt).max(0.0))
                .collect(),
        };
        let (dp3, dv3, dq3, dw3, dr3, _) =
            self.compute_derivatives(&s3, rotor_commands_rad_s, wind_world);

        // State at endpoint (Stage 4):
        let s4 = FlightDynamicsState {
            position_world: self.state.position_world + dp3 * dt,
            velocity_body: self.state.velocity_body + dv3 * dt,
            orientation: Quaternion::new(
                self.state.orientation.w + dq3.w * dt,
                self.state.orientation.x + dq3.x * dt,
                self.state.orientation.y + dq3.y * dt,
                self.state.orientation.z + dq3.z * dt,
            ),
            angular_velocity_body: self.state.angular_velocity_body + dw3 * dt,
            rotor_speeds_rad_s: self
                .state
                .rotor_speeds_rad_s
                .iter()
                .zip(&dr3)
                .map(|(&r, &dr)| (r + dr * dt).max(0.0))
                .collect(),
        };
        let (dp4, dv4, dq4, dw4, dr4, _) =
            self.compute_derivatives(&s4, rotor_commands_rad_s, wind_world);

        // RK4 Weighted Combination:
        let inv_6 = 1.0 / 6.0;
        self.state.position_world =
            self.state.position_world + (dp1 + dp2 * 2.0 + dp3 * 2.0 + dp4) * (dt * inv_6);
        self.state.velocity_body =
            self.state.velocity_body + (dv1 + dv2 * 2.0 + dv3 * 2.0 + dv4) * (dt * inv_6);

        let final_qw =
            self.state.orientation.w + (dq1.w + dq2.w * 2.0 + dq3.w * 2.0 + dq4.w) * (dt * inv_6);
        let final_qx =
            self.state.orientation.x + (dq1.x + dq2.x * 2.0 + dq3.x * 2.0 + dq4.x) * (dt * inv_6);
        let final_qy =
            self.state.orientation.y + (dq1.y + dq2.y * 2.0 + dq3.y * 2.0 + dq4.y) * (dt * inv_6);
        let final_qz =
            self.state.orientation.z + (dq1.z + dq2.z * 2.0 + dq3.z * 2.0 + dq4.z) * (dt * inv_6);
        self.state.orientation = Quaternion::new(final_qw, final_qx, final_qy, final_qz);

        self.state.angular_velocity_body =
            self.state.angular_velocity_body + (dw1 + dw2 * 2.0 + dw3 * 2.0 + dw4) * (dt * inv_6);

        for i in 0..self.state.rotor_speeds_rad_s.len() {
            let delta = (dr1[i] + dr2[i] * 2.0 + dr3[i] * 2.0 + dr4[i]) * (dt * inv_6);
            self.state.rotor_speeds_rad_s[i] = (self.state.rotor_speeds_rad_s[i] + delta).max(0.0);
        }

        // Ground constraint: prevent sinking below ground (z = 0):
        if self.state.position_world.z < 0.0 {
            self.state.position_world.z = 0.0;
            if self.state.velocity_body.z < 0.0 {
                self.state.velocity_body.z = 0.0;
            }
        }

        self.sim_time_s += dt;

        // Sample physical sensor suite:
        // 1. IMU:
        let imu_meas = sample_imu(
            &self.imu_config,
            &mut self.imu_state,
            self.sim_time_s,
            spec_force1,
            self.state.angular_velocity_body,
            self.state.orientation,
            298.15,
            rng,
        );

        // 2. Barometer:
        let baro_pressure = self
            .barometer
            .sample_pressure_pa(self.state.position_world.z, rng);

        // 3. LiDAR Rangefinder:
        let lidar_reading =
            self.lidar
                .sample_range(self.state.position_world.z, self.state.orientation, rng);

        // 4. GNSS/GPS Receiver:
        let (pos_noise_x, pos_noise_y) = rng.next_gaussian();
        let (pos_noise_z, vel_noise_x) = rng.next_gaussian();
        let (vel_noise_y, vel_noise_z) = rng.next_gaussian();
        let gps_pos_std = 0.35; // 35 cm RMS horizontal
        let gps_vel_std = 0.05; // 5 cm/s RMS velocity

        let world_velocity = self
            .state
            .orientation
            .rotate_vector_body_to_world(self.state.velocity_body);
        let gps_meas = GpsMeasurement {
            timestamp_s: self.sim_time_s,
            position_world_m: self.state.position_world
                + Vector3D::new(
                    pos_noise_x * gps_pos_std,
                    pos_noise_y * gps_pos_std,
                    pos_noise_z * (gps_pos_std * 1.5),
                ),
            velocity_world_m_s: world_velocity
                + Vector3D::new(
                    vel_noise_x * gps_vel_std,
                    vel_noise_y * gps_vel_std,
                    vel_noise_z * gps_vel_std,
                ),
            hdop: 0.85,
            vdop: 1.20,
            fix_type: GpsFixType::Fix3D,
            satellites_visible: 14,
            is_spoofed: false,
        };

        (imu_meas, baro_pressure, lidar_reading, gps_meas)
    }
}
