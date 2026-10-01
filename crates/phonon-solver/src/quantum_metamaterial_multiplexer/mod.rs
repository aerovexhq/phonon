#![deny(unsafe_code)]

//! Quantum metamaterial polariton multiplexer solver module definitions and runners.

pub mod multiplexer_solver;
pub mod multiplexer_benchmark;

pub use multiplexer_solver::*;
pub use multiplexer_benchmark::*;
