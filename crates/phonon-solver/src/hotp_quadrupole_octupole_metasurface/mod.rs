#![deny(unsafe_code)]

//! Quantum acoustic higher-order topological quadrupole-octupole superlattices
//! and non-Hermitian corner metasurface solvers and benchmark runners.

pub mod hotp_benchmark;
pub mod hotp_solver;

pub use hotp_benchmark::*;
pub use hotp_solver::*;
