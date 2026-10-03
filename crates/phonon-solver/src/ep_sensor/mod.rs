#![deny(unsafe_code)]

//! Non-Hermitian Exceptional Point (EP) sensor, Jordan block decomposition,
//! and Parity-Time (PT) symmetric circuit simulator.
//!
//! Models fractional sensitivity enhancement at higher-order exceptional points (EP2, EP3, EP4),
//! Petermann excess noise factor divergence, PT-symmetric coupled RLC circuits,
//! and non-Hermitian skin effect (NHSE) boundary localization.

pub mod jordan_eigensolver;
pub mod pt_circuit;

pub use jordan_eigensolver::{
    Complex, EpOrder, NonHermitianHamiltonian, RiemannBranchPoint,
};
pub use pt_circuit::{
    NhseLattice, NhseResult, PtCircuitParams, PtCircuitState, PtPhase,
};
