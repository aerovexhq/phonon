#![deny(unsafe_code)]

//! Topological moire acoustic polaritonic lattices and flat-band phonon superfluidity.

mod moire_polariton_benchmark;
mod moire_polariton_solver;

pub use moire_polariton_benchmark::{MoirePolaritonBenchmarkResult, MoirePolaritonBenchmarkRunner};
pub use moire_polariton_solver::TopologicalMoirePolaritonSolver;
