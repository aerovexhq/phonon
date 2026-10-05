#![deny(unsafe_code)]

//! Topological Acoustic Synthetic Dimension & 4D Quantum Hall Effect Metamaterial module.
//!
//! Provides:
//! - 4D Dirac Hamiltonian with Clifford Gamma matrices in synthetic frequency dimensions.
//! - Quantized second Chern number C_2 calculation and 4D bulk band dispersion.
//! - 3D gapless chiral boundary hyper-surface states under open boundary conditions.
//! - Quantized non-linear 4D Hall current j_x = sigma_4D * E_y * B_zw.
//! - Synthetic frequency sideband dynamics and topological defect immunity.

pub mod four_dim_lattice;
pub mod hall_response_engine;

pub use four_dim_lattice::{
    BoundaryHyperSurfaceMode, FourDimDispersionPoint, FourDimLatticeSolver, Synthetic4dParams,
};
pub use hall_response_engine::{
    SyntheticHallEngine, SyntheticHallMetrics, SyntheticHallParams, SyntheticHarmonicPoint,
};
