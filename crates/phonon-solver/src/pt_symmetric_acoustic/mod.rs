#![deny(unsafe_code)]

//! Parity-Time (PT) Symmetric Non-Hermitian Acoustic Metamaterial & Unidirectional Invisibility Engine.
//!
//! Provides:
//! - 2-level PT-symmetric acoustic resonator Hamiltonian and exceptional point (EP) eigensolver.
//! - Petermann factor K quantifying eigenvector non-orthogonality and coalescent singularity.
//! - Unidirectional reflectionlessness (R_L = 0, R_R != 0) and acoustic cloaking.
//! - Generalized PT-symmetric scattering unitarity relation |T - 1| = sqrt(R_L * R_R).
//! - 1D spatial acoustic pressure field profile for left vs right wave incidence.

pub mod pt_hamiltonian;
pub mod invisibility_engine;

pub use pt_hamiltonian::{
    PtAcousticParams, PtEigenvalue, PtHamiltonianSolver, PtModalMetrics, PtPhaseClassification,
};
pub use invisibility_engine::{
    InvisibilityEngine, InvisibilityMetrics, InvisibilityParams, ScatteringSpectrumPoint,
    SpatialFieldPoint,
};
