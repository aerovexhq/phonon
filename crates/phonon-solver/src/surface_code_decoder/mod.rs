#![deny(unsafe_code)]

//! Quantum acoustic non-Abelian chiral topological surface code anyon decoders
//! and fault-tolerant syndrome processors.

pub mod decoder_benchmark;
pub mod decoder_solver;

pub use decoder_benchmark::*;
pub use decoder_solver::*;
