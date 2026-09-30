#![deny(unsafe_code)]

//! Multi-physics solver and parallel benchmark suite for quantum acoustic
//! non-Abelian chiral topological anyon braiding circuit compilers and topological QASM synthesizers.

pub mod compiler_benchmark;
pub mod compiler_solver;

pub use compiler_benchmark::*;
pub use compiler_solver::*;
