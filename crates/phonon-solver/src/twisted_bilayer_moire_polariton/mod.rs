#![deny(unsafe_code)]

//! Quantum acoustic twisted bilayer moire polariton superlattices and
//! flat-band phonon superconductors module.

pub mod polariton_benchmark;
pub mod polariton_solver;

pub use polariton_benchmark::*;
pub use polariton_solver::*;
