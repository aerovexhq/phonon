#![deny(unsafe_code)]

//! Multi-physics solver and parallel benchmark suite for the Phonon Universal Multi-Scale
//! Visual Studio Autonomous Acoustically Levitated Diamond Optomechanical Spin Sensor &
//! Micro-Tesla Magnetometer Engine.

pub mod magnetometer_benchmark;
pub mod magnetometer_solver;

pub use magnetometer_benchmark::*;
pub use magnetometer_solver::*;
