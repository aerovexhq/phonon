#![deny(unsafe_code)]

//! Multi-physics solver and benchmark suite for autonomous second-order topological quadrupole
//! insulator and corner-state qubit engine.

pub mod qubit_benchmark;
pub mod qubit_solver;

pub use qubit_benchmark::*;
pub use qubit_solver::*;
