#![deny(unsafe_code)]

//! Multi-physics solver and benchmark suite for autonomous quantum acoustoelectric
//! metamaterial transistor and non-reciprocal microwave isolator engine.

pub mod transistor_benchmark;
pub mod transistor_solver;

pub use transistor_benchmark::*;
pub use transistor_solver::*;
