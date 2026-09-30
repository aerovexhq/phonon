#![deny(unsafe_code)]

//! Multi-physics solver and parallel benchmark suite for Phonon Universal Multi-Scale
//! Visual Studio Autonomous Acoustically Driven Superconducting Nanowire Single-Photon Detector &
//! Hybrid Optomechanical Co-Readout Engine.

pub mod detector_benchmark;
pub mod detector_solver;

pub use detector_benchmark::*;
pub use detector_solver::*;
