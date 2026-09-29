#![deny(unsafe_code)]

//! Floquet-Bloch synthetic gauge acoustic solver and parallel benchmark suite.

pub mod gauge_benchmark;
pub mod gauge_solver;

pub use gauge_benchmark::*;
pub use gauge_solver::*;
