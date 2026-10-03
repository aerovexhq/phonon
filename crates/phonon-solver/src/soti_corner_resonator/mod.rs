#![deny(unsafe_code)]

//! SOTI Quadrupole Lattice BBH Hamiltonian & Corner Nanocavity Engine.
//!
//! Provides the 2D quadrupole topological insulator Benalcazar-Bernevig-Hughes (BBH)
//! model, quantized bulk quadrupole moment q_xy, real-space finite lattice assembly,
//! and localized corner cavity state eigensolver.

pub mod bbh_hamiltonian;
pub mod corner_cavity;

pub use bbh_hamiltonian::{
    BandDispersionPoint, BbhComplex, BbhHamiltonian, HighSymmetryPoint, QuadrupoleParams,
    HIGH_SYMMETRY_PATH,
};
pub use corner_cavity::{
    jacobi_eigensolver, CornerEigenstate, CornerId, SotiLattice, SotiLatticeResult,
};
