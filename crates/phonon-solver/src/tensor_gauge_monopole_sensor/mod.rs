#![deny(unsafe_code)]

//! Topological acoustic higher-rank tensor gauge fields and chiral
//! monopole-plaquette phononic sensors solver module.

pub mod tensor_benchmark;
pub mod tensor_solver;

pub use tensor_benchmark::*;
pub use tensor_solver::*;
