#![deny(unsafe_code)]

//! Quantum phonon-exciton polariton condensate solver and parallel benchmark suite.

pub mod polariton_benchmark;
pub mod polariton_solver;

pub use polariton_benchmark::*;
pub use polariton_solver::*;
