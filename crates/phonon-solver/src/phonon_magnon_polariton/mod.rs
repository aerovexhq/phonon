#![deny(unsafe_code)]

//! Coherent Phonon-Magnon Polariton Transducer & Quantum Microwave-to-Acoustic Interface Module.
//!
//! Exposes:
//! - Hybridized polariton dispersion, avoided crossing gap, and Hopfield fractions (`dispersion`).
//! - Quantum microwave-to-acoustic scattering matrix [S(omega)], efficiency, and noise (`transducer_s_matrix`).
//! - Dynamic magnetoelastic strain drive, RF field, and resonant acoustic attenuation (`magnetoelastic_drive`).

pub mod dispersion;
pub mod magnetoelastic_drive;
pub mod transducer_s_matrix;

pub use dispersion::{
    PhononMagnonParams, PolaritonDispersionEngine, PolaritonDispersionPoint, GYROMAGNETIC_RATIO,
};
pub use magnetoelastic_drive::{
    MagnetoelasticDriveEngine, MagnetoelasticDriveParams, MagnetoelasticTrackSnapshot,
};
pub use transducer_s_matrix::{
    QuantumTransducerSolver, SParameterSample, TransducerCouplingParams,
};
