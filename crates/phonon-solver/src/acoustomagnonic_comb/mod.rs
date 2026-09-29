#![deny(unsafe_code)]

//! Chiral phonon-magnon polariton frequency combs and quantum topological acoustomagnonics.

pub mod comb_benchmark;
pub mod comb_solver;

pub use comb_benchmark::{CombBenchmarkResult, CombBenchmarkRunner};
pub use comb_solver::AcoustomagnonicCombSolver;
