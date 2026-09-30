#![deny(unsafe_code)]

//! Quantum acoustic non-Abelian chiral topological surface-code lattice anyon transceivers
//! and braiding fabric routers solver module.

pub mod transceiver_benchmark;
pub mod transceiver_solver;

pub use transceiver_benchmark::*;
pub use transceiver_solver::*;
