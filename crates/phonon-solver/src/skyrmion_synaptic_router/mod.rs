#![deny(unsafe_code)]

//! Solver and benchmark modules for the autonomous acoustically driven skyrmion synaptic router engine.

pub mod router_benchmark;
pub mod router_solver;

pub use router_benchmark::*;
pub use router_solver::*;
