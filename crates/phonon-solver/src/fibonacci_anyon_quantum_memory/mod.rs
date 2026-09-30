#![deny(unsafe_code)]

//! Multi-physics solvers and benchmark suites for quantum acoustic non-Abelian anyonic
//! quantum memory and chiral Fibonacci braiding gate fabrics.

pub mod fibonacci_benchmark;
pub mod fibonacci_solver;

pub use fibonacci_benchmark::*;
pub use fibonacci_solver::*;
