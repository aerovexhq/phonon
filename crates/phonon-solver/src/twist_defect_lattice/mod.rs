#![deny(unsafe_code)]

//! Quantum acoustic non-Abelian chiral topological twist-defect Majorana braiding
//! lattices and gauge-invariant state teleporters solver and parallel benchmark suite.

pub mod defect_benchmark;
pub mod defect_solver;

pub use defect_benchmark::*;
pub use defect_solver::*;
