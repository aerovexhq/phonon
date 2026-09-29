//! Chiral phonon-magnon skyrmion braiding and non-volatile acoustic memory.

pub mod skyrmion_memory_benchmark;
pub mod skyrmion_memory_solver;

pub use skyrmion_memory_benchmark::{
    SkyrmionMemoryBenchmarkReport, SkyrmionMemoryBenchmarkRunner, SkyrmionMemorySweepPoint,
};
pub use skyrmion_memory_solver::SkyrmionMemorySolver;
