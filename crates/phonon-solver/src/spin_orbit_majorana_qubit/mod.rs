#![deny(unsafe_code)]

//! Autonomous Acoustically Driven Spin-Orbit Majorana Parity Qubit Synthesizer
//! & Fault-Tolerant Logic Engine solver module.

pub mod qubit_benchmark;
pub mod qubit_solver;

pub use qubit_benchmark::*;
pub use qubit_solver::*;
