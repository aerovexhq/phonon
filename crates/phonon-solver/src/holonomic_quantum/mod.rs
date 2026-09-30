#![deny(unsafe_code)]

//! Multi-physics solver and benchmark suite for autonomous non-Abelian holonomic quantum
//! computing gate synthesizer and geometric phase engine.

pub mod holonomic_benchmark;
pub mod holonomic_solver;

pub use holonomic_benchmark::*;
pub use holonomic_solver::*;
