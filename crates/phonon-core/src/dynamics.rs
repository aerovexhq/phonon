#![deny(unsafe_code)]

//! Abstract open-source physics dynamics backend trait and normalized telemetry/actuator interfaces.
//!
//! This module decouples the Phonon platform from proprietary flight simulation kernels
//! by defining a normalized interface for 6-DOF multi-rotor and fixed-wing dynamics engines.

use thiserror::Error;

/// Actuator command inputs dispatched to the dynamics backend at each integration step.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActuatorInputs {
    /// 4 rotor thrust forces in Newtons, clamped to >= 0.0 N.
    pub rotor_thrust_n: [f64; 4],
    /// Aerodynamic control surface deflection angles in radians, clamped to [-0.5, 0.5] rad.
    pub control_surfaces_rad: [f64; 4],
    /// Simulation timestamp associated with these actuator commands in seconds.
    pub timestamp_s: f64,
}

impl Default for ActuatorInputs {
    fn default() -> Self {
        Self {
            rotor_thrust_n: [0.0; 4],
            control_surfaces_rad: [0.0; 4],
            timestamp_s: 0.0,
        }
    }
}

impl ActuatorInputs {
    /// Creates a new `ActuatorInputs` instance with boundary clamping applied.
    #[inline]
    pub fn new(
        rotor_thrust_n: [f64; 4],
        control_surfaces_rad: [f64; 4],
        timestamp_s: f64,
    ) -> Self {
        Self {
            rotor_thrust_n: [
                rotor_thrust_n[0].max(0.0),
                rotor_thrust_n[1].max(0.0),
                rotor_thrust_n[2].max(0.0),
                rotor_thrust_n[3].max(0.0),
            ],
            control_surfaces_rad: [
                control_surfaces_rad[0].clamp(-0.5, 0.5),
                control_surfaces_rad[1].clamp(-0.5, 0.5),
                control_surfaces_rad[2].clamp(-0.5, 0.5),
                control_surfaces_rad[3].clamp(-0.5, 0.5),
            ],
            timestamp_s,
        }
    }

    /// Constructs an actuator command configured for hover equilibrium.
    #[inline]
    pub fn hover(thrust_per_rotor: f64) -> Self {
        let clamped = thrust_per_rotor.max(0.0);
        Self {
            rotor_thrust_n: [clamped; 4],
            control_surfaces_rad: [0.0; 4],
            timestamp_s: 0.0,
        }
    }

    /// Returns the sum of all rotor thrust forces in Newtons.
    #[inline]
    pub fn total_thrust_n(&self) -> f64 {
        self.rotor_thrust_n[0]
            + self.rotor_thrust_n[1]
            + self.rotor_thrust_n[2]
            + self.rotor_thrust_n[3]
    }
}

/// Instantaneous 6-DOF flight dynamics telemetry packet.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DynamicsTelemetry {
    /// Inertial position [x, y, z] in the North-East-Down (NED) frame in meters.
    pub position_m: [f64; 3],
    /// Inertial velocity [vx, vy, vz] in the North-East-Down (NED) frame in m/s.
    pub velocity_m_per_s: [f64; 3],
    /// Linear acceleration [ax, ay, az] in the North-East-Down (NED) frame in m/s^2.
    pub acceleration_m_per_s2: [f64; 3],
    /// Orientation unit quaternion [w, x, y, z] mapping body frame to NED frame.
    pub orientation_quat: [f64; 4],
    /// Body-frame angular velocity [p, q, r] in rad/s (roll, pitch, yaw rates).
    pub angular_velocity_rad_per_s: [f64; 3],
    /// Body-frame angular acceleration [p_dot, q_dot, r_dot] in rad/s^2.
    pub angular_acceleration_rad_per_s2: [f64; 3],
    /// Total accumulated simulation time in seconds.
    pub sim_time_s: f64,
    /// Total accumulated simulation integration steps.
    pub step_count: u64,
}

impl Default for DynamicsTelemetry {
    fn default() -> Self {
        Self {
            position_m: [0.0; 3],
            velocity_m_per_s: [0.0; 3],
            acceleration_m_per_s2: [0.0; 3],
            orientation_quat: [1.0, 0.0, 0.0, 0.0],
            angular_velocity_rad_per_s: [0.0; 3],
            angular_acceleration_rad_per_s2: [0.0; 3],
            sim_time_s: 0.0,
            step_count: 0,
        }
    }
}

impl DynamicsTelemetry {
    /// Constructs a new telemetry record with explicit state variables.
    #[inline]
    pub fn new(
        position_m: [f64; 3],
        velocity_m_per_s: [f64; 3],
        acceleration_m_per_s2: [f64; 3],
        orientation_quat: [f64; 4],
        angular_velocity_rad_per_s: [f64; 3],
        angular_acceleration_rad_per_s2: [f64; 3],
        sim_time_s: f64,
        step_count: u64,
    ) -> Self {
        Self {
            position_m,
            velocity_m_per_s,
            acceleration_m_per_s2,
            orientation_quat,
            angular_velocity_rad_per_s,
            angular_acceleration_rad_per_s2,
            sim_time_s,
            step_count,
        }
    }

    /// Computes the scalar speed (Euclidean norm of NED velocity) in meters per second.
    #[inline]
    pub fn speed_m_per_s(&self) -> f64 {
        (self.velocity_m_per_s[0].powi(2)
            + self.velocity_m_per_s[1].powi(2)
            + self.velocity_m_per_s[2].powi(2))
        .sqrt()
    }

    /// Computes the altitude above ground in meters (defined as negative z in NED).
    #[inline]
    pub fn altitude_m(&self) -> f64 {
        -self.position_m[2]
    }
}

/// Metadata and capability descriptor for a physics dynamics backend.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BackendInfo {
    /// Human-readable name of the dynamics backend engine.
    pub name: String,
    /// Semantic version of the dynamics engine.
    pub version: String,
    /// Indicates whether the backend leverages GPU/hardware acceleration.
    pub is_hardware_accelerated: bool,
    /// Theoretical maximum tick rate in Hertz.
    pub max_tick_rate_hz: f64,
    /// Detailed description of the solver architecture and physics assumptions.
    pub description: String,
}

impl BackendInfo {
    /// Constructs a new `BackendInfo` descriptor.
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        is_hardware_accelerated: bool,
        max_tick_rate_hz: f64,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            is_hardware_accelerated,
            max_tick_rate_hz,
            description: description.into(),
        }
    }
}

/// Error conditions originating from dynamics backend evaluation and numerical integration.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum DynamicsError {
    /// Numerical divergence or non-finite values encountered during state integration.
    #[error("Numerical divergence detected: {0}")]
    NumericalDivergence(String),

    /// Invalid actuator input values (e.g., NaN or infinite commands).
    #[error("Invalid actuator input: {0}")]
    InvalidActuatorInput(String),

    /// The dynamics backend state has not been initialized.
    #[error("Dynamics state uninitialized")]
    StateUninitialized,

    /// The dynamics backend is currently unavailable or disconnected.
    #[error("Dynamics backend unavailable: {0}")]
    BackendUnavailable(String),

    /// Integration step failed to execute.
    #[error("Simulation step failed: {0}")]
    StepFailed(String),
}

/// Abstract open-source physics dynamics backend trait.
///
/// Any physics engine (reference RK4, shared memory bridge, Gazebo plugin, or C-ABI solver)
/// must implement this trait to integrate seamlessly into Phonon Studio.
pub trait PhysicsDynamicsBackend: Send + Sync {
    /// Returns the capability and metadata descriptor of this dynamics backend.
    fn info(&self) -> BackendInfo;

    /// Resets the dynamics simulation to an optional initial state or default rest pose.
    fn reset(&mut self, initial_state: Option<&DynamicsTelemetry>) -> Result<(), DynamicsError>;

    /// Advances the dynamical state by time step `dt_s` using the supplied `inputs`.
    fn step(&mut self, dt_s: f64, inputs: &ActuatorInputs) -> Result<&DynamicsTelemetry, DynamicsError>;

    /// Returns a reference to the latest calculated telemetry state.
    fn current_telemetry(&self) -> &DynamicsTelemetry;

    /// Verifies that the dynamics engine is running normally and has not diverged.
    fn is_healthy(&self) -> bool;
}
