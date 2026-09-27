//! Modified Nodal Analysis (MNA) matrix formulation, stamping, and DC solving.

pub mod assembler;
pub mod diagnostics;
pub mod linear_solver;
pub mod non_linear_solver;
pub mod stamp;

pub use assembler::{assemble_mna_dc, MnaSystem, SolverOptions};
pub use diagnostics::diagnose_mna_singularity;
pub use linear_solver::{solve_dc_linear, DcSolution};
pub use non_linear_solver::{
    solve_dc_non_linear, AtomisticCompanionModel, ModelContext, NewtonOptions,
};
