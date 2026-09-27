//! Topological quantum computing solver engines, non-Abelian braiding, and benchmark runners.

pub mod braiding_solver;
pub mod parity_tracker;
pub mod topological_benchmark;

pub use braiding_solver::{BraidingStepResult, MajoranaBraidingSolver};
pub use parity_tracker::{FermionParitySolver, ParityTrackingReport};
pub use topological_benchmark::{BenchmarkComparisonReport, TopologicalBenchmarkRunner};
