#![deny(unsafe_code)]

//! Multi-physics solver and parallel benchmark suite for the autonomous
//! acoustically mediated magnon-phonon entanglement swapping and quantum repeater node engine.

pub mod repeater_benchmark;
pub mod repeater_solver;

pub use repeater_benchmark::*;
pub use repeater_solver::*;
