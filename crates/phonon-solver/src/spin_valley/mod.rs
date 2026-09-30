#![deny(unsafe_code)]

//! Multi-physics solver and benchmark suite for autonomous acoustically mediated spin-valley
//! polariton multiplexer and 2D valleytronics engine.

pub mod valley_benchmark;
pub mod valley_solver;

pub use valley_benchmark::*;
pub use valley_solver::*;
