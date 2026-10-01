#![deny(unsafe_code)]

//! Multi-physics solver and parallel parameter sweep benchmark suite for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Topological
//! Axion-Polariton Photonic Isolator & Quantum Routing Engine.

pub mod isolator_benchmark;
pub mod isolator_solver;

pub use isolator_benchmark::*;
pub use isolator_solver::*;
