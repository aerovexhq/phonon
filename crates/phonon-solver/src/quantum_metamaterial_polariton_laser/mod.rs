#![deny(unsafe_code)]

//! Quantum metamaterial polariton laser solver module for Phase 271.

pub mod laser_benchmark;
pub mod laser_solver;

pub use laser_benchmark::*;
pub use laser_solver::*;
