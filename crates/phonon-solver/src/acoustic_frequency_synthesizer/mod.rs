#![deny(unsafe_code)]

//! Solver and parallel benchmark modules for the autonomous topological phononic
//! acoustic frequency synthesizer and ultra-low phase noise local oscillator engine.

pub mod synthesizer_solver;
pub mod synthesizer_benchmark;

pub use synthesizer_solver::*;
pub use synthesizer_benchmark::*;
