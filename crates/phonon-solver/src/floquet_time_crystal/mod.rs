#![deny(unsafe_code)]

//! Floquet-Bloch Quantum Acoustic Discrete Time Crystal (DTC) Simulator.
//!
//! Provides non-equilibrium periodic drive Hamiltonian modeling, stroboscopic time-translation
//! symmetry breaking dynamics, subharmonic Fourier spectral rigidity analysis, MBL stabilization,
//! and temporal Edwards-Anderson spin glass order.

pub mod drive_hamiltonian;
pub mod subharmonic_order;

pub use drive_hamiltonian::{
    Complex, FloquetState, FloquetStateKind, FloquetTimeCrystalParams, FloquetUnitaryOperator,
    StroboscopicTrajectory,
};
pub use subharmonic_order::{
    EdwardsAndersonOrder, RigidityPhaseDiagram, SubharmonicSpectralAnalysis,
};
