#![deny(unsafe_code)]

//! Topological acoustic fracton dynamics and sub-system symmetry-protected
//! phononic multipole routers solver module.

pub mod fracton_benchmark;
pub mod fracton_solver;

pub use fracton_benchmark::*;
pub use fracton_solver::*;
