#![deny(unsafe_code)]

//! Topological Acoustic Flat-Band Lieb Lattice and Synthetic Gauge Simulator.
//!
//! Provides the 3-band momentum-space Hamiltonian, exact flat-band dispersion,
//! finite real-space lattice assembly, single-plaquette Compact Localized States (CLS),
//! and synthetic Aharonov-Bohm (AB) caging engine under gauge flux modulation.

pub mod ab_caging;
pub mod hamiltonian;

pub use ab_caging::{
    hermitian_eigensolver, jacobi_symmetric_eigensolver, AbCagingSimulator, CompactLocalizedState,
    DisorderResilienceResult, LiebLattice, LiebLatticeResult, XorShiftRng,
};
pub use hamiltonian::{
    HighSymmetryPoint, LiebBandPoint, LiebComplex, LiebHamiltonian, LiebParams, HIGH_SYMMETRY_PATH,
};
