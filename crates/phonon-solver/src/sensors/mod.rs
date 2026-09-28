//! Physics-Coupled Sensors & Multi-Physics Aerovex Co-Simulation Engine
//!
//! Provides synchronized IMU (6-DOF / 9-DOF), piezoresistive/capacitive tactile sensing,
//! contact manifold force distribution, and Rayon parallel co-simulation benchmarks.

pub mod aerovex_bridge;
pub mod sensor_benchmark;

pub use aerovex_bridge::{
    AerovexCoSimPacket, AerovexPhononBridge, AerovexRigidBodyState, PhononTransducerOutput,
};
pub use sensor_benchmark::{SensorBenchmarkReport, SensorBenchmarkRunner};
