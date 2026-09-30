#![deny(unsafe_code)]

//! Non-Abelian quantum acoustic fault-tolerant surface codes and chiral Majorana stabilizer solver modules.

pub mod surface_code_benchmark;
pub mod surface_code_solver;

pub use surface_code_benchmark::*;
pub use surface_code_solver::*;
