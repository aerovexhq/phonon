#![deny(unsafe_code)]

//! Higher-Order Axion Insulator simulator for 3D topological acoustic metamaterials.
//!
//! Provides:
//! - 3D 4-band tight-binding Axion Hamiltonian with Clifford Gamma matrices,
//!   quantized magnetoelectric response P_3 = 1/2 (mod 1), and half-quantized surface Hall conductance.
//! - Finite Nx x Ny rod lattice cross-section solver with surface TRS-breaking mass,
//!   cyclic Jacobi Hermitian eigensolver, and chiral 1D gapless hinge states traversing
//!   the bulk and surface bandgaps along the rod corners.

pub mod hamiltonian;
pub mod hinge_modes;

pub use hamiltonian::{
    AxionBandPoint, AxionComplex, AxionHamiltonian, AxionParams, CliffordGamma,
    HighSymmetryPoint, AXION_HIGH_SYMMETRY_PATH,
};
pub use hinge_modes::{
    hermitian_eigensolver, jacobi_symmetric_eigensolver, AxionRodLattice, AxionRodResult,
    HingeDisorderResult, HingeEigenmode, HingeId, HingeSParameters, XorShiftRng,
};
