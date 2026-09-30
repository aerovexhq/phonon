#![deny(unsafe_code)]

//! Quantum acoustic non-Abelian chiral topological quantum error-mitigating
//! spin-phonon braiding engines solver and benchmark suite.

pub mod braiding_benchmark;
pub mod braiding_solver;

pub use braiding_benchmark::*;
pub use braiding_solver::*;
