#![deny(unsafe_code)]

//! Multi-physics solvers and benchmark suites for quantum acoustic higher-order axion
//! electrodynamics and chiral quadrupole-hinge polariton circulators.

pub mod hinge_benchmark;
pub mod hinge_solver;

pub use hinge_benchmark::*;
pub use hinge_solver::*;
