#![deny(unsafe_code)]

//! Quantum acoustic non-Abelian chiral topological Majorana-driven transmon hybrid interfaces
//! and cryogenic quantum bus transceivers solvers and parallel benchmark suite.

pub mod hybrid_benchmark;
pub mod hybrid_solver;

pub use hybrid_benchmark::*;
pub use hybrid_solver::*;
