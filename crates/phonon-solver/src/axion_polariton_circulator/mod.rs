#![deny(unsafe_code)]

//! Autonomous Acoustically Driven Topological Axion-Polariton Circulator &
//! Quantum Interconnect Hub Engine solver module.

pub mod circulator_benchmark;
pub mod circulator_solver;

pub use circulator_benchmark::{
    AxionPolaritonCirculatorBenchmarkResult, AxionPolaritonCirculatorBenchmarkRunner,
};
pub use circulator_solver::AxionPolaritonCirculatorSolver;
