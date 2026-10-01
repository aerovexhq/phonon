#![deny(unsafe_code)]

//! Multi-physics solvers and parallel benchmark runners for the autonomous
//! acoustically driven topological non-Abelian Majorana-parafermion braid lattice
//! and quantum error correction engine.

pub mod lattice_benchmark;
pub mod lattice_solver;

pub use lattice_benchmark::*;
pub use lattice_solver::*;
