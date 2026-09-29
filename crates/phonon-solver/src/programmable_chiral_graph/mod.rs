#![deny(unsafe_code)]

//! Multi-physics solver and parallel benchmark suite for programmable chiral
//! phonon networks and high-dimensional quantum acoustic graph states.

pub mod graph_benchmark;
pub mod graph_solver;

pub use graph_benchmark::{GraphBenchmarkResult, GraphBenchmarkRunner};
pub use graph_solver::ProgrammableChiralGraphSolver;
