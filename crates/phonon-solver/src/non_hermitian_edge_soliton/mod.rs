#![deny(unsafe_code)]

//! Non-Hermitian topological acoustic edge soliton solver and parallel benchmark suite.

pub mod edge_soliton_benchmark;
pub mod edge_soliton_solver;

pub use edge_soliton_benchmark::*;
pub use edge_soliton_solver::*;
