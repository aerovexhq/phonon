#![deny(unsafe_code)]

//! Multi-physics solvers and parallel benchmark suites for the Phonon Universal Multi-Scale
//! Visual Studio Autonomous Acoustically Driven Floquet-Chern Parafermion Laser & Coherent
//! Soliton Router Engine (Phase 282).

pub mod laser_solver;
pub mod laser_benchmark;

pub use laser_solver::*;
pub use laser_benchmark::*;
