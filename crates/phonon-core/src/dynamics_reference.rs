#![deny(unsafe_code)]

//! Pure safe Rust reference 6-DOF Newton-Euler flight dynamics backend with 4th-order Runge-Kutta (RK4) integration.
//!
//! This module provides a completely standalone, self-contained reference physics solver for
//! quadrotors and multi-rotor aircraft with zero external dependencies and zero proprietary code.

use crate::dynamics::{
    ActuatorInputs, BackendInfo, DynamicsError, DynamicsTelemetry, PhysicsDynamicsBackend,
};

/// Physical and aerodynamic parameters configuring the reference dynamics backend.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReferenceDynamicsParams {
    /// Total aircraft mass in kilograms (default: 1.5 kg).
    pub mass_kg: f64,
    /// Diagonal elements of the body inertia tensor [Ixx, Iyy, Izz] in kg*m^2 (default: [0.015, 0.015, 0.025]).
    pub inertia_diag_kg_m2: [f64; 3],
    /// Moment arm distance from vehicle center of gravity to rotor center in meters (default: 0.225 m).
    pub arm_length_m: f64,
    /// Rotor torque-to-thrust reaction coefficient in meters (default: 0.016 m, tau_z = torque_coeff * thrust).
    pub torque_coefficient: f64,
    /// Quadratic aerodynamic drag coefficient in kg/m (default: 0.08 kg/m, F_drag = C_d * |v| * v).
    pub drag_coefficient: f64,
    /// Gravitational acceleration in m/s^2 (default: 9.80665 m/s^2).
    pub gravity_m_per_s2: f64,
}

impl Default for ReferenceDynamicsParams {
    fn default() -> Self {
        Self {
            mass_kg: 1.5,
            inertia_diag_kg_m2: [0.015, 0.015, 0.025],
            arm_length_m: 0.225,
            torque_coefficient: 0.016,
            drag_coefficient: 0.08,
            gravity_m_per_s2: 9.80665,
        }
    }
}

impl ReferenceDynamicsParams {
    /// Constructs a new configuration with explicit physical parameters.
    pub fn new(
        mass_kg: f64,
        inertia_diag_kg_m2: [f64; 3],
        arm_length_m: f64,
        torque_coefficient: f64,
        drag_coefficient: f64,
        gravity_m_per_s2: f64,
    ) -> Self {
        Self {
            mass_kg,
            inertia_diag_kg_m2,
            arm_length_m,
            torque_coefficient,
            drag_coefficient,
            gravity_m_per_s2,
        }
    }

    /// Sets the total aircraft mass in kilograms.
    pub fn with_mass_kg(mut self, mass_kg: f64) -> Self {
        self.mass_kg = mass_kg;
        self
    }

    /// Sets the diagonal elements of the inertia tensor in kg*m^2.
    pub fn with_inertia_diag_kg_m2(mut self, inertia: [f64; 3]) -> Self {
        self.inertia_diag_kg_m2 = inertia;
        self
    }

    /// Sets the rotor moment arm length in meters.
    pub fn with_arm_length_m(mut self, arm_length_m: f64) -> Self {
        self.arm_length_m = arm_length_m;
        self
    }

    /// Sets the rotor torque reaction coefficient in meters.
    pub fn with_torque_coefficient(mut self, torque_coefficient: f64) -> Self {
        self.torque_coefficient = torque_coefficient;
        self
    }

    /// Sets the quadratic aerodynamic drag coefficient in kg/m.
    pub fn with_drag_coefficient(mut self, drag_coefficient: f64) -> Self {
        self.drag_coefficient = drag_coefficient;
        self
    }

    /// Sets the gravitational acceleration constant in m/s^2.
    pub fn with_gravity_m_per_s2(mut self, gravity_m_per_s2: f64) -> Self {
        self.gravity_m_per_s2 = gravity_m_per_s2;
        self
    }
}

/// Pure safe Rust reference 6-DOF flight dynamics engine.
///
/// Implements 6-DOF Newton-Euler equations of motion using classical Runge-Kutta 4th-order (RK4)
/// numerical integration, quaternion attitude kinematics, diagonal inertia dynamics, quadratic drag,
/// and quadrotor thrust/moment mapping.
#[derive(Debug, Clone)]
pub struct ReferenceDynamicsBackend {
    params: ReferenceDynamicsParams,
    telemetry: DynamicsTelemetry,
    healthy: bool,
}

impl Default for ReferenceDynamicsBackend {
    fn default() -> Self {
        Self::new(ReferenceDynamicsParams::default())
    }
}

impl ReferenceDynamicsBackend {
    /// Creates a new reference dynamics backend instance with the specified parameters.
    pub fn new(params: ReferenceDynamicsParams) -> Self {
        Self {
            params,
            telemetry: DynamicsTelemetry::default(),
            healthy: true,
        }
    }

    /// Returns an immutable reference to the physical parameters.
    #[inline]
    pub fn params(&self) -> &ReferenceDynamicsParams {
        &self.params
    }

    /// Returns a mutable reference to the physical parameters.
    #[inline]
    pub fn params_mut(&mut self) -> &mut ReferenceDynamicsParams {
        &mut self.params
    }

    /// Overwrites the physical parameters.
    #[inline]
    pub fn set_params(&mut self, params: ReferenceDynamicsParams) {
        self.params = params;
    }

    /// Returns the current state vector [x, y, z, vx, vy, vz, qw, qx, qy, qz, p, q, r].
    pub fn state_vector(&self) -> [f64; 13] {
        [
            self.telemetry.position_m[0],
            self.telemetry.position_m[1],
            self.telemetry.position_m[2],
            self.telemetry.velocity_m_per_s[0],
            self.telemetry.velocity_m_per_s[1],
            self.telemetry.velocity_m_per_s[2],
            self.telemetry.orientation_quat[0],
            self.telemetry.orientation_quat[1],
            self.telemetry.orientation_quat[2],
            self.telemetry.orientation_quat[3],
            self.telemetry.angular_velocity_rad_per_s[0],
            self.telemetry.angular_velocity_rad_per_s[1],
            self.telemetry.angular_velocity_rad_per_s[2],
        ]
    }

    /// Sets the internal state vector [x, y, z, vx, vy, vz, qw, qx, qy, qz, p, q, r].
    pub fn set_state_vector(&mut self, state: [f64; 13]) {
        self.telemetry.position_m = [state[0], state[1], state[2]];
        self.telemetry.velocity_m_per_s = [state[3], state[4], state[5]];

        let mut quat = [state[6], state[7], state[8], state[9]];
        normalize_quaternion(&mut quat);
        self.telemetry.orientation_quat = quat;

        self.telemetry.angular_velocity_rad_per_s = [state[10], state[11], state[12]];
    }

    /// Evaluates state derivatives for a given state vector and thrust inputs.
    ///
    /// State indices:
    /// - [0..3]: Position [x, y, z] in NED frame (m)
    /// - [3..6]: Velocity [vx, vy, vz] in NED frame (m/s)
    /// - [6..10]: Attitude quaternion [qw, qx, qy, qz] (body to NED)
    /// - [10..13]: Angular velocity [p, q, r] in body frame (rad/s)
    #[inline]
    fn compute_derivatives(
        state: &[f64; 13],
        thrusts: &[f64; 4],
        params: &ReferenceDynamicsParams,
    ) -> [f64; 13] {
        let mut ds = [0.0; 13];

        // 1. Kinematics: d(position)/dt = velocity
        ds[0] = state[3];
        ds[1] = state[4];
        ds[2] = state[5];

        let qw = state[6];
        let qx = state[7];
        let qy = state[8];
        let qz = state[9];

        // 2. Linear Dynamics: Forces in NED frame
        // Thrust vector in body frame: [0, 0, -sum(thrust_i)]
        let total_thrust = thrusts[0] + thrusts[1] + thrusts[2] + thrusts[3];

        // Body-to-NED rotation matrix applied to [0, 0, -total_thrust]:
        // R(q) * [0, 0, -T]^T:
        // R_13 = 2(qx*qz + qw*qy)
        // R_23 = 2(qy*qz - qw*qx)
        // R_33 = 1 - 2(qx^2 + qy^2)
        let f_thrust_ned = [
            2.0 * (qx * qz + qw * qy) * (-total_thrust),
            2.0 * (qy * qz - qw * qx) * (-total_thrust),
            (1.0 - 2.0 * (qx * qx + qy * qy)) * (-total_thrust),
        ];

        // Gravity force in NED frame: [0, 0, m * g]
        let f_gravity_ned = [0.0, 0.0, params.mass_kg * params.gravity_m_per_s2];

        // Quadratic aerodynamic drag force in NED frame: F_drag = C_d * |v| * v
        let vx = state[3];
        let vy = state[4];
        let vz = state[5];
        let speed = (vx * vx + vy * vy + vz * vz).sqrt();
        let f_drag_ned = [
            params.drag_coefficient * speed * vx,
            params.drag_coefficient * speed * vy,
            params.drag_coefficient * speed * vz,
        ];

        // Total force and linear acceleration in NED frame: a_ned = (F_thrust + F_gravity - F_drag) / mass
        let inv_mass = 1.0 / params.mass_kg;
        ds[3] = (f_thrust_ned[0] + f_gravity_ned[0] - f_drag_ned[0]) * inv_mass;
        ds[4] = (f_thrust_ned[1] + f_gravity_ned[1] - f_drag_ned[1]) * inv_mass;
        ds[5] = (f_thrust_ned[2] + f_gravity_ned[2] - f_drag_ned[2]) * inv_mass;

        // 3. Attitude Kinematics: dq/dt = 0.5 * q \otimes [0, p, q_rate, r]
        let p = state[10];
        let q_rate = state[11];
        let r = state[12];

        ds[6] = 0.5 * (-qx * p - qy * q_rate - qz * r);
        ds[7] = 0.5 * ( qw * p + qy * r      - qz * q_rate);
        ds[8] = 0.5 * ( qw * q_rate - qx * r + qz * p);
        ds[9] = 0.5 * ( qw * r      + qx * q_rate - qy * p);

        // 4. Rotational Dynamics: Euler equations of motion
        // Torques generated by rotor configuration:
        // Roll: tau_x = arm * (T3 - T1)
        // Pitch: tau_y = arm * (T2 - T0)
        // Yaw: tau_z = torque_coeff * (T0 - T1 + T2 - T3)
        let tau_x = params.arm_length_m * (thrusts[3] - thrusts[1]);
        let tau_y = params.arm_length_m * (thrusts[2] - thrusts[0]);
        let tau_z = params.torque_coefficient * (thrusts[0] - thrusts[1] + thrusts[2] - thrusts[3]);

        let ixx = params.inertia_diag_kg_m2[0];
        let iyy = params.inertia_diag_kg_m2[1];
        let izz = params.inertia_diag_kg_m2[2];

        // I * w_dot + w x (I * w) = tau => w_dot = I^{-1} * (tau - w x (I * w))
        ds[10] = (tau_x - (izz - iyy) * q_rate * r) / ixx;
        ds[11] = (tau_y - (ixx - izz) * p * r) / iyy;
        ds[12] = (tau_z - (iyy - ixx) * p * q_rate) / izz;

        ds
    }
}

/// Normalizes a 4D quaternion vector in place to unit length.
#[inline]
fn normalize_quaternion(q: &mut [f64; 4]) {
    let norm_sq = q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3];
    if norm_sq > 1e-24 {
        let inv_norm = 1.0 / norm_sq.sqrt();
        q[0] *= inv_norm;
        q[1] *= inv_norm;
        q[2] *= inv_norm;
        q[3] *= inv_norm;
    } else {
        *q = [1.0, 0.0, 0.0, 0.0];
    }
}

impl PhysicsDynamicsBackend for ReferenceDynamicsBackend {
    fn info(&self) -> BackendInfo {
        BackendInfo::new(
            "Phonon Pure Safe Rust Reference RK4 Dynamics Engine",
            "0.1.0",
            false,
            10_000_000.0,
            "Pure safe Rust 6-DOF Newton-Euler flight dynamics solver with 4th-order Runge-Kutta numerical integration",
        )
    }

    fn reset(&mut self, initial_state: Option<&DynamicsTelemetry>) -> Result<(), DynamicsError> {
        if let Some(state) = initial_state {
            // Verify all values are finite
            for (idx, v) in state.position_m.iter().chain(&state.velocity_m_per_s)
                .chain(&state.acceleration_m_per_s2)
                .chain(&state.orientation_quat)
                .chain(&state.angular_velocity_rad_per_s)
                .chain(&state.angular_acceleration_rad_per_s2)
                .enumerate()
            {
                if !v.is_finite() {
                    return Err(DynamicsError::InvalidActuatorInput(format!(
                        "Initial state contains non-finite value at index {}: {}",
                        idx, v
                    )));
                }
            }

            let mut quat = state.orientation_quat;
            normalize_quaternion(&mut quat);

            self.telemetry = DynamicsTelemetry {
                position_m: state.position_m,
                velocity_m_per_s: state.velocity_m_per_s,
                acceleration_m_per_s2: state.acceleration_m_per_s2,
                orientation_quat: quat,
                angular_velocity_rad_per_s: state.angular_velocity_rad_per_s,
                angular_acceleration_rad_per_s2: state.angular_acceleration_rad_per_s2,
                sim_time_s: state.sim_time_s,
                step_count: state.step_count,
            };
        } else {
            self.telemetry = DynamicsTelemetry::default();
        }

        self.healthy = true;
        Ok(())
    }

    fn step(&mut self, dt_s: f64, inputs: &ActuatorInputs) -> Result<&DynamicsTelemetry, DynamicsError> {
        if dt_s <= 0.0 || !dt_s.is_finite() {
            return Err(DynamicsError::StepFailed(format!(
                "Invalid time step dt_s: {}. Must be positive and finite.",
                dt_s
            )));
        }

        if !self.healthy {
            return Err(DynamicsError::BackendUnavailable(
                "Dynamics backend is in an unhealthy diverged state; call reset() first.".to_string(),
            ));
        }

        // Validate actuator inputs
        for (i, t) in inputs.rotor_thrust_n.iter().enumerate() {
            if !t.is_finite() {
                return Err(DynamicsError::InvalidActuatorInput(format!(
                    "Rotor thrust[{}] is non-finite: {}",
                    i, t
                )));
            }
        }
        for (i, cs) in inputs.control_surfaces_rad.iter().enumerate() {
            if !cs.is_finite() {
                return Err(DynamicsError::InvalidActuatorInput(format!(
                    "Control surface[{}] is non-finite: {}",
                    i, cs
                )));
            }
        }

        let clamped_thrusts = [
            inputs.rotor_thrust_n[0].max(0.0),
            inputs.rotor_thrust_n[1].max(0.0),
            inputs.rotor_thrust_n[2].max(0.0),
            inputs.rotor_thrust_n[3].max(0.0),
        ];

        let mut s0 = self.state_vector();

        // Runge-Kutta 4th Order (RK4) Integration
        // Stage 1
        let k1 = Self::compute_derivatives(&s0, &clamped_thrusts, &self.params);

        // Stage 2
        let mut s1 = [0.0; 13];
        for i in 0..13 {
            s1[i] = s0[i] + 0.5 * dt_s * k1[i];
        }
        let mut q_stage2 = [s1[6], s1[7], s1[8], s1[9]];
        normalize_quaternion(&mut q_stage2);
        s1[6] = q_stage2[0];
        s1[7] = q_stage2[1];
        s1[8] = q_stage2[2];
        s1[9] = q_stage2[3];
        let k2 = Self::compute_derivatives(&s1, &clamped_thrusts, &self.params);

        // Stage 3
        let mut s2 = [0.0; 13];
        for i in 0..13 {
            s2[i] = s0[i] + 0.5 * dt_s * k2[i];
        }
        let mut q_stage3 = [s2[6], s2[7], s2[8], s2[9]];
        normalize_quaternion(&mut q_stage3);
        s2[6] = q_stage3[0];
        s2[7] = q_stage3[1];
        s2[8] = q_stage3[2];
        s2[9] = q_stage3[3];
        let k3 = Self::compute_derivatives(&s2, &clamped_thrusts, &self.params);

        // Stage 4
        let mut s3 = [0.0; 13];
        for i in 0..13 {
            s3[i] = s0[i] + dt_s * k3[i];
        }
        let mut q_stage4 = [s3[6], s3[7], s3[8], s3[9]];
        normalize_quaternion(&mut q_stage4);
        s3[6] = q_stage4[0];
        s3[7] = q_stage4[1];
        s3[8] = q_stage4[2];
        s3[9] = q_stage4[3];
        let k4 = Self::compute_derivatives(&s3, &clamped_thrusts, &self.params);

        // Accumulate RK4 weighted sum
        let rk_weight = dt_s / 6.0;
        for i in 0..13 {
            s0[i] += rk_weight * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
        }

        // Re-normalize final attitude quaternion
        let mut q_final = [s0[6], s0[7], s0[8], s0[9]];
        normalize_quaternion(&mut q_final);
        s0[6] = q_final[0];
        s0[7] = q_final[1];
        s0[8] = q_final[2];
        s0[9] = q_final[3];

        // Check for numerical divergence
        for val in &s0 {
            if !val.is_finite() {
                self.healthy = false;
                return Err(DynamicsError::NumericalDivergence(
                    "State vector produced non-finite NaN/Inf during integration".to_string(),
                ));
            }
        }

        let speed_sq = s0[3] * s0[3] + s0[4] * s0[4] + s0[5] * s0[5];
        if speed_sq > 1e12 {
            self.healthy = false;
            return Err(DynamicsError::NumericalDivergence(format!(
                "Speed exceeded divergence threshold: {:.3} m/s",
                speed_sq.sqrt()
            )));
        }

        // Compute instantaneous acceleration and angular acceleration at new state
        let final_derivs = Self::compute_derivatives(&s0, &clamped_thrusts, &self.params);

        self.telemetry.position_m = [s0[0], s0[1], s0[2]];
        self.telemetry.velocity_m_per_s = [s0[3], s0[4], s0[5]];
        self.telemetry.acceleration_m_per_s2 = [final_derivs[3], final_derivs[4], final_derivs[5]];
        self.telemetry.orientation_quat = q_final;
        self.telemetry.angular_velocity_rad_per_s = [s0[10], s0[11], s0[12]];
        self.telemetry.angular_acceleration_rad_per_s2 = [final_derivs[10], final_derivs[11], final_derivs[12]];
        self.telemetry.sim_time_s += dt_s;
        self.telemetry.step_count += 1;

        Ok(&self.telemetry)
    }

    fn current_telemetry(&self) -> &DynamicsTelemetry {
        &self.telemetry
    }

    fn is_healthy(&self) -> bool {
        self.healthy
    }
}
