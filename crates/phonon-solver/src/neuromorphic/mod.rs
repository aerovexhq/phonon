//! Neuromorphic reservoir computing, liquid state machines, and hardware benchmarks.

pub mod liquid_state_machine;
pub mod reservoir_benchmark;
pub mod reservoir_solver;

pub use liquid_state_machine::{LiquidStateMachine, LsmConfig};
pub use reservoir_benchmark::{NeuromorphicBenchmarkReport, NeuromorphicBenchmarkRunner};
pub use reservoir_solver::{
    generate_lorenz63, generate_mackey_glass, generate_narma10, ReservoirSolver, TrainedReadout,
};
