#![deny(unsafe_code)]

//! Skyrmion-Parafermion Topological Quantum Transceiver & Metamaterial Crossbar Switch Engine module.

pub mod transceiver_benchmark;
pub mod transceiver_solver;

pub use transceiver_benchmark::*;
pub use transceiver_solver::*;
