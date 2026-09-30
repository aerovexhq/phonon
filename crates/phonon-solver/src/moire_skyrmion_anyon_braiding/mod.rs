#![deny(unsafe_code)]

//! Non-Abelian quantum acoustic anyonic braiding in moire skyrmion crystals
//! and chiral topological spin-Peierls transducers solvers and benchmark runners.

pub mod skyrmion_benchmark;
pub mod skyrmion_solver;

pub use skyrmion_benchmark::*;
pub use skyrmion_solver::*;
