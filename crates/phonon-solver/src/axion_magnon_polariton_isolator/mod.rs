#![deny(unsafe_code)]

//! Topological Axion-Magnon Polariton Isolator & Quantum Memory Routing Engine solver and benchmark suite.

pub mod isolator_benchmark;
pub mod isolator_solver;

pub use isolator_benchmark::*;
pub use isolator_solver::*;
