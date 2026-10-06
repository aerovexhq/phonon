#![deny(unsafe_code)]

//! Topological Acoustic Bound States in the Continuum (BIC) & High-Q Vortex Cavity Engine.
//!
//! Provides:
//! - Symmetry-protected Gamma-point BICs and off-Gamma Friedrich-Wintgen interference BICs.
//! - Momentum-space far-field polarization vortices with integer topological charge q = +-1, +-2.
//! - Inverse-quadratic quasi-BIC quality factor scaling Q proportional to 1 / alpha^2.
//! - Ultra-sharp Fano resonance transmission lineshapes with giant field enhancement (|p_cav|^2 / |p_inc|^2 >= 1000x).
//! - Radiation acoustic orbital angular momentum (OAM) vortex beam generation with purity >= 95%.

pub mod bic_lattice;
pub mod cavity_vortex_engine;

pub use bic_lattice::{
    BicKind, BicLatticeParams, BicLatticeSolver, FarFieldPolarizationVector,
};
pub use cavity_vortex_engine::{
    AcousticVortexFieldPoint, CavityVortexEngine, CavityVortexMetrics, CavityVortexParams,
    FanoTransmissionPoint,
};
