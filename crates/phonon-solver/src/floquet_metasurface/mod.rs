#![deny(unsafe_code)]

//! Floquet Engineered Spatio-Temporal Acoustic Metasurface module.
//!
//! Provides:
//! - MetasurfaceUnitCell: unit cell resonance, spatial pitch, acoustic speed of sound, and impedance.
//! - FloquetModulationParams: spatio-temporal carrier, modulation frequency, phase gradient, and modulation depth.
//! - FloquetSidebandResult: discrete Floquet harmonic reflection angles, Doppler frequency shifts, and Bessel efficiency.
//! - FloquetMetasurfaceSolver: generalized Snell's law deflection and non-reciprocal Doppler transmission isolation.
//! - SyntheticGaugeField: synthetic vector potential A_eff and magnetic field B_eff with Aharonov-Bohm phase.
//! - OrbitalAngularMomentum: topological charge transfer, helical phase winding, and high purity vortex beam synthesis.

pub mod dispersion;
pub mod spatio_temporal;

pub use dispersion::{
    bessel_j, FloquetMetasurfaceSolver, FloquetModulationParams, FloquetSidebandResult,
    MetasurfaceUnitCell, NonReciprocalScattering,
};
pub use spatio_temporal::{
    OrbitalAngularMomentum, PolarPhaseMap, SyntheticGaugeField,
};
