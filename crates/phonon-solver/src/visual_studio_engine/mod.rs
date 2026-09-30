#![deny(unsafe_code)]

//! Visual Studio Engine solver and benchmark runner for Phonon Universal Multi-Scale
//! Visual Studio Native Engine & WebAssembly Real-Time Physics Interactive Co-Processor.

pub mod studio_engine_benchmark;
pub mod studio_engine_solver;

pub use studio_engine_benchmark::*;
pub use studio_engine_solver::*;
