#![deny(unsafe_code)]

//! Multi-physics solver and parallel benchmark suite for topological
//! quantum acoustic memory and Majorana surface code decoders.

pub mod memory_benchmark;
pub mod memory_solver;

pub use memory_benchmark::{MemoryBenchmarkResult, MemoryBenchmarkRunner};
pub use memory_solver::MajoranaSurfaceMemorySolver;
