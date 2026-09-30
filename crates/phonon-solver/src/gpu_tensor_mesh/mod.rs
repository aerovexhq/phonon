#![deny(unsafe_code)]

//! GPU Tensor Mesh solver and benchmark runner for Phonon Universal Multi-Scale
//! Visual Studio GPU WebGPU / Metal Accelerators & Real-Time Tensor Mesh Solvers.

pub mod mesh_benchmark;
pub mod mesh_solver;

pub use mesh_benchmark::*;
pub use mesh_solver::*;
