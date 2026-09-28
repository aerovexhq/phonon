//! Physics-Coupled Sensors & Multi-Physics Aerovex Co-Simulation Engine
//!
//! Provides synchronized IMU (6-DOF / 9-DOF), piezoresistive/capacitive tactile sensing,
//! contact manifold force distribution, and Rayon parallel co-simulation benchmarks.

pub mod aerovex_bridge;
pub mod nv_benchmark;
pub mod nv_imaging;
pub mod nv_solver;
pub mod sensor_benchmark;

pub use aerovex_bridge::{
    AerovexCoSimPacket, AerovexPhononBridge, AerovexRigidBodyState, PhononTransducerOutput,
};
pub use nv_benchmark::{
    BenchmarkScenario, MagnetometerSpecs, MagnetometerTechnology, NvBenchmarkReport,
    NvBenchmarkRunner,
};
pub use nv_imaging::{fft_1d, fft_2d, CircuitDefect, DefectType, NvProbeGrid};
pub use nv_solver::{CpmgResult, HahnEchoResult, NvSolver, PulseSequenceType, RamseyResult};
pub use sensor_benchmark::{SensorBenchmarkReport, SensorBenchmarkRunner};
