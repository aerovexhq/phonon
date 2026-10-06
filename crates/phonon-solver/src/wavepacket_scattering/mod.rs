#![deny(unsafe_code)]

//! Real-Time Microscopic Electron & Phonon Wavepacket Scattering Module.
//!
//! Exposes the 1D Time-Dependent Schroedinger Equation (TDSE) unitary Crank-Nicolson stepper,
//! acoustic phonon lattice deformation potential coupling, and potential barrier quantum tunneling.

pub mod boundary_reflection;
pub mod phonon_scattering;
pub mod schroedinger_stepper;

pub use boundary_reflection::{
    BarrierShape, BoundaryTransmissionResult, PotentialBarrier,
};
pub use phonon_scattering::{
    AcousticPhononMode, InelasticScatteringKinematics,
};
pub use schroedinger_stepper::{
    Complex, SchroedingerStepper, WavepacketDiagnostics, WavepacketParams,
    ELECTRON_MASS_KG, ELEMENTARY_CHARGE, HBAR,
};
