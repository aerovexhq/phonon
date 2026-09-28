pub mod coincidence_solver;
pub mod electrothermal_solver;
pub mod quantum_lidar_benchmark;

pub use coincidence_solver::{CoincidenceConfig, CoincidenceResult, CoincidenceSolver};
pub use electrothermal_solver::{ElectroThermalConfig, ElectroThermalSolver, SnspdPulseTrace};
pub use quantum_lidar_benchmark::{
    QuantumLidarBenchmarkConfig, QuantumLidarBenchmarkReport, QuantumLidarBenchmarkRunner,
};
