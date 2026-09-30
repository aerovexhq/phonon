#![deny(unsafe_code)]

//! Multi-physics solver and benchmark suite for autonomous skyrmionic-phononic memory lattice
//! and chiral domain wall track engine.

pub mod memory_benchmark;
pub mod memory_solver;

pub use memory_benchmark::*;
pub use memory_solver::*;
