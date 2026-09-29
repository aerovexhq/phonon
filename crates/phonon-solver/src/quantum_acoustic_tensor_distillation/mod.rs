#![deny(unsafe_code)]

//! Quantum acoustic tensor network simulators and continuous-variable
//! fault-tolerant magic state distillation solvers and parallel benchmark suite.

pub mod distillation_solver;
pub mod distillation_benchmark;

pub use distillation_solver::*;
pub use distillation_benchmark::*;
