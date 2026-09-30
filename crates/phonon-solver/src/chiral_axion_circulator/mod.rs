#![deny(unsafe_code)]

//! Chiral acoustic axion electrodynamics and dynamic magnetoelectric phonon circulator solver modules.

pub mod circulator_benchmark;
pub mod circulator_solver;

pub use circulator_benchmark::*;
pub use circulator_solver::*;
