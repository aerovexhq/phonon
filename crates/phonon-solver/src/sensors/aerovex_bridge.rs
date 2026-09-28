//! Bidirectional Aerovex-Sim & Phonon Multi-Physics Co-Simulation Bridge
//!
//! Provides synchronized lock-free co-simulation stepping bridging discrete
//! Aerovex multi-world physics ticks (250 - 1000 Hz) with Phonon continuous
//! MNA circuit and transducer models.

use phonon_models::em::{ChannelRng, Vector3D};
use phonon_models::sensors::{
    sample_imu, CapacitiveSensorConfig, CollisionContactInput, ImuConfig, ImuMeasurement, ImuState,
    PiezoresistiveSensorConfig, Quaternion, TactileMatrixArray,
};

/// Kinematic state vector of an Aerovex rigid body in world and body reference frames.
#[derive(Debug, Clone, PartialEq)]
pub struct AerovexRigidBodyState {
    /// Position in world Cartesian space in meters.
    pub position_world: Vector3D,
    /// Linear velocity in world Cartesian space in m/s.
    pub linear_velocity_world: Vector3D,
    /// Linear acceleration in body reference frame in m/s^2.
    pub linear_accel_body: Vector3D,
    /// Angular velocity [p, q, r] in body reference frame in rad/s.
    pub angular_velocity_body: Vector3D,
    /// Vehicle attitude orientation unit quaternion (w, x, y, z).
    pub orientation: Quaternion,
    /// Total vehicle mass in kilograms.
    pub mass_kg: f64,
}

impl Default for AerovexRigidBodyState {
    fn default() -> Self {
        Self {
            position_world: Vector3D::new(0.0, 0.0, 0.0),
            linear_velocity_world: Vector3D::new(0.0, 0.0, 0.0),
            linear_accel_body: Vector3D::new(0.0, 0.0, 0.0),
            angular_velocity_body: Vector3D::new(0.0, 0.0, 0.0),
            orientation: Quaternion::default(),
            mass_kg: 1.5,
        }
    }
}

/// Co-simulation packet passed from Aerovex simulation kernel to Phonon every physics tick.
#[derive(Debug, Clone, PartialEq)]
pub struct AerovexCoSimPacket {
    /// Simulation epoch time in seconds.
    pub sim_time_s: f64,
    /// Monotonic physics tick counter.
    pub tick_index: u64,
    /// World index in multi-world simulation (0..127).
    pub world_id: u32,
    /// Rigid body kinematics.
    pub rigid_body: AerovexRigidBodyState,
    /// Active collision contact manifold inputs from rigid body solver.
    pub collision_contacts: Vec<CollisionContactInput>,
    /// Environmental operating temperature in Kelvin.
    pub ambient_temp_kelvin: f64,
}

/// Transduced multi-physics sensor telemetry returned from Phonon back to Aerovex.
#[derive(Debug, Clone, PartialEq)]
pub struct PhononTransducerOutput {
    /// Simulation epoch time in seconds.
    pub sim_time_s: f64,
    /// Monotonic physics tick counter.
    pub tick_index: u64,
    /// Calibrated 6-DOF / 9-DOF IMU telemetry measurement.
    pub imu_telemetry: ImuMeasurement,
    /// Distributed normal forces across tactile matrix taxels in Newtons.
    pub tactile_forces_n: Vec<f64>,
    /// Tactile Wheatstone bridge differential voltages in Volts.
    pub tactile_voltages_v: Vec<f64>,
    /// Dynamic elastomeric capacitance under contact load in Farads.
    pub capacitive_contact_f: f64,
    /// Total integrated contact normal force in Newtons.
    pub total_contact_force_n: f64,
    /// 2D Center of Pressure (CoP) coordinates (x, y) in meters on sensor surface.
    pub center_of_pressure_m: (f64, f64),
}

/// Bidirectional Aerovex-Phonon Co-Simulation Bridge Engine.
#[derive(Debug, Clone, PartialEq)]
pub struct AerovexPhononBridge {
    pub imu_config: ImuConfig,
    pub imu_state: ImuState,
    pub tactile_matrix: TactileMatrixArray,
    pub capacitive_sensor: CapacitiveSensorConfig,
    pub sensor_center_body: Vector3D,
    pub sensor_normal_body: Vector3D,
    pub sensor_up_body: Vector3D,
    pub last_sim_time_s: f64,
}

impl AerovexPhononBridge {
    /// Creates a new co-simulation bridge for a vehicle equipped with IMU and 4x4 tactile array.
    pub fn new(
        imu_config: ImuConfig,
        tactile_rows: usize,
        tactile_cols: usize,
        tactile_pitch_m: f64,
    ) -> Self {
        let tactile_matrix = TactileMatrixArray::new(
            tactile_rows,
            tactile_cols,
            tactile_pitch_m,
            PiezoresistiveSensorConfig::default(),
        );

        Self {
            imu_config,
            imu_state: ImuState::default(),
            tactile_matrix,
            capacitive_sensor: CapacitiveSensorConfig::default(),
            sensor_center_body: Vector3D::new(0.0, 0.0, -0.05), // Mounted on bottom landing pad
            sensor_normal_body: Vector3D::new(0.0, 0.0, -1.0),  // Normal points downwards
            sensor_up_body: Vector3D::new(1.0, 0.0, 0.0),       // Forward axis is local up
            last_sim_time_s: 0.0,
        }
    }

    /// Steps the multi-physics transducer models forward by one Aerovex co-simulation tick.
    pub fn step(
        &mut self,
        packet: &AerovexCoSimPacket,
        rng: &mut ChannelRng,
    ) -> PhononTransducerOutput {
        self.last_sim_time_s = packet.sim_time_s;

        // 1. IMU Transduction:
        let imu_meas = sample_imu(
            &self.imu_config,
            &mut self.imu_state,
            packet.sim_time_s,
            packet.rigid_body.linear_accel_body,
            packet.rigid_body.angular_velocity_body,
            packet.rigid_body.orientation,
            packet.ambient_temp_kelvin,
            rng,
        );

        // 2. Tactile Matrix Transduction from Collision Contacts:
        self.tactile_matrix.taxel_forces_n.fill(0.0);
        let mut total_contact_force = 0.0;

        for contact in &packet.collision_contacts {
            total_contact_force += contact.normal_force_n;
            // Update spatial distribution:
            self.tactile_matrix.update_from_contact(
                contact,
                packet.rigid_body.position_world + self.sensor_center_body,
                self.sensor_normal_body,
                self.sensor_up_body,
            );
        }

        let tactile_voltages = self.tactile_matrix.taxel_voltages_v();
        let cop = self.tactile_matrix.center_of_pressure_m();

        // 3. Capacitive elastomeric sensor:
        let dyn_capacitance = self.capacitive_sensor.capacitance_f(total_contact_force);

        PhononTransducerOutput {
            sim_time_s: packet.sim_time_s,
            tick_index: packet.tick_index,
            imu_telemetry: imu_meas,
            tactile_forces_n: self.tactile_matrix.taxel_forces_n.clone(),
            tactile_voltages_v: tactile_voltages,
            capacitive_contact_f: dyn_capacitance,
            total_contact_force_n: total_contact_force,
            center_of_pressure_m: cop,
        }
    }
}
