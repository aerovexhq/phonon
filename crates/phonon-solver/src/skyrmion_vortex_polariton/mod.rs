#![deny(unsafe_code)]

//! Quantum acoustic non-Abelian chiral topological skyrmion-vortex polariton networks
//! and non-Clifford geometric braiding engines solver and parallel benchmark suite.

pub mod polariton_benchmark;
pub mod polariton_solver;

pub use polariton_benchmark::*;
pub use polariton_solver::*;
