//! Physics-Coupled Sensors & Multi-Axis Inertial Transducers
//!
//! Provides models for tactile pressure sensors, contact manifolds,
//! and 6-DOF / 9-DOF MEMS IMUs with Allan variance noise processes.

pub mod imu;
pub mod tactile;

pub use imu::{
    sample_imu, AllanNoiseConfig, ImuConfig, ImuMeasurement, ImuState, Quaternion,
    STANDARD_GRAVITY_M_S2,
};
pub use tactile::{
    CapacitiveSensorConfig, CollisionContactInput, PiezoresistiveSensorConfig, TactileMatrixArray,
};
