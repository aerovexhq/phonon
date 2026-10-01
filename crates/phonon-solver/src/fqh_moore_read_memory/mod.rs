#![deny(unsafe_code)]

//! Multi-physics solver and parallel benchmark suite for the Phonon Universal Multi-Scale
//! Visual Studio Autonomous Acoustically Driven Fractional Quantum Hall Moore-Read Anyon
//! Quantum Memory & Surface Code Router Engine (Phase 293).

pub mod memory_benchmark;
pub mod memory_solver;

pub use memory_benchmark::*;
pub use memory_solver::*;
