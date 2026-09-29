#![deny(unsafe_code)]

//! Quantum non-Abelian holonomic acoustic gate processor solver and parallel benchmark suite.

pub mod holonomic_processor_benchmark;
pub mod holonomic_processor_solver;

pub use holonomic_processor_benchmark::*;
pub use holonomic_processor_solver::*;
