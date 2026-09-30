#![deny(unsafe_code)]

//! Physical solvers and parallel parameter sweep benchmarks for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Topological Non-Abelian
//! Parafermion Braiding Lattice & Fractional Fault-Tolerant Surface Code Engine.

pub mod surface_code_solver;
pub mod surface_code_benchmark;

pub use surface_code_solver::*;
pub use surface_code_benchmark::*;
