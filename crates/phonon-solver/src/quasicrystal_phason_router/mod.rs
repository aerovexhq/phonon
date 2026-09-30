#![deny(unsafe_code)]

//! Quantum acoustic non-Abelian chiral topological quasicrystal phason-defect
//! routers and higher-dimensional state concentrators solver and benchmark suite.

pub mod router_benchmark;
pub mod router_solver;

pub use router_benchmark::*;
pub use router_solver::*;
