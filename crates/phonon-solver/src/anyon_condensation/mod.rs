#![deny(unsafe_code)]

//! Quantum acoustic non-Abelian chiral topological anyon condensation networks
//! and higher-form gauge transceivers solver and parallel benchmark suite.

pub mod condensation_solver;
pub mod condensation_benchmark;

pub use condensation_solver::*;
pub use condensation_benchmark::*;
