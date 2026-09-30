#![deny(unsafe_code)]

//! Collaboration Fabric solver and benchmark runner for Phonon Universal Multi-Scale
//! Visual Studio Native Binary IPC & Remote Cloud Collaboration Fabric.

pub mod fabric_benchmark;
pub mod fabric_solver;

pub use fabric_benchmark::*;
pub use fabric_solver::*;
