#![deny(unsafe_code)]

//! Physical solvers and parallel parameter sweep benchmarks for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Skyrmion-Majorana
//! Hybrid Qubit Register & Topological Crossbar Engine.

pub mod crossbar_solver;
pub mod crossbar_benchmark;

pub use crossbar_solver::*;
pub use crossbar_benchmark::*;
