//! Solvers for Terahertz cavity magnon polaritons, relativistic Néel domain walls,
//! and antiferromagnetic spintronics.

pub mod afm_polariton_solver;
pub mod afm_spintronics_benchmark;
pub mod neel_domain_wall_solver;

pub use afm_polariton_solver::*;
pub use afm_spintronics_benchmark::*;
pub use neel_domain_wall_solver::*;
