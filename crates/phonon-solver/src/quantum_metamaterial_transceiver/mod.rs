#![deny(unsafe_code)]

//! Autonomous Acoustically Driven Quantum Metamaterial Polariton Transceiver
//! & Multi-Scale Photonic Engine solver and benchmark suite.

pub mod transceiver_benchmark;
pub mod transceiver_solver;

pub use transceiver_benchmark::*;
pub use transceiver_solver::*;
