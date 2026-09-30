#![deny(unsafe_code)]

//! Multi-physics solvers and parallel benchmark suite for topological acoustic chiral
//! skyrmion-lattice transducers and non-reciprocal magnon-polaron interconnects.

pub mod skyrmion_polaron_benchmark;
pub mod skyrmion_polaron_solver;

pub use skyrmion_polaron_benchmark::*;
pub use skyrmion_polaron_solver::*;
