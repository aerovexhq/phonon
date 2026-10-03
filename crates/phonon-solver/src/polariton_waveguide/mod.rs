#![deny(unsafe_code)]

//! Polariton Waveguide & Topological Photonic Cavity Simulator.
//!
//! Multi-mode exciton-polariton waveguide dispersion solver, vacuum Rabi splitting,
//! topological Chern insulator metamaterial waveguide lattice, and chiral edge states.

pub mod chiral_edge;
pub mod dispersion;

pub use chiral_edge::{
    ChiralEdgeModeSolver, ChiralLatticeDefect, ChiralTransmissionPoint, ChiralWavefunction2D,
};
pub use dispersion::{
    MultiModePolaritonDispersionSolver, PolaritonDispersionPoint, PolaritonWaveguideParams,
    ELECTRON_REST_MASS_ENERGY_MEV, HBAR_C_MEV_UM, SPEED_OF_LIGHT_UM_PER_PS,
};
