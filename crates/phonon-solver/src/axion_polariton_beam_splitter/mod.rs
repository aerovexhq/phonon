#![deny(unsafe_code)]

//! Autonomous acoustically driven topological axion-polariton waveguide and
//! quantum Hall beam splitter solver and parallel benchmark suite.

pub mod splitter_benchmark;
pub mod splitter_solver;

pub use splitter_benchmark::*;
pub use splitter_solver::*;
