#![deny(unsafe_code)]

//! Quantum acoustic non-Abelian chiral topological surface-acoustic-wave (SAW)
//! soliton routing arrays and non-linear optical hybrid switchyards solver module.

pub mod routing_benchmark;
pub mod routing_solver;

pub use routing_benchmark::*;
pub use routing_solver::*;
