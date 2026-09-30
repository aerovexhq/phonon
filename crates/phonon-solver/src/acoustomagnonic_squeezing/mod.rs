#![deny(unsafe_code)]

//! Multi-physics solver and benchmark suite for autonomous cavity acoustomagnonic
//! squeezing and quantum entangled spin-phonon comb engine.

pub mod squeezing_benchmark;
pub mod squeezing_solver;

pub use squeezing_benchmark::*;
pub use squeezing_solver::*;
