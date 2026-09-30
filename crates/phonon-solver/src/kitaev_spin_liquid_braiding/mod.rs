#![deny(unsafe_code)]

//! Non-Abelian quantum acoustic Kitaev spin-liquid anyon braiding and Majorana
//! nanoresonator transceivers solver module.

pub mod kitaev_benchmark;
pub mod kitaev_solver;

pub use kitaev_benchmark::*;
pub use kitaev_solver::*;
