#![deny(unsafe_code)]

//! Quantum acoustic non-Abelian chiral topological optomechanical polariton
//! switchyards and multi-channel routing networks solvers and parallel benchmark suite.

pub mod switchyard_benchmark;
pub mod switchyard_solver;

pub use switchyard_benchmark::*;
pub use switchyard_solver::*;
