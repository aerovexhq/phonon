#![deny(unsafe_code)]

//! Cavity Optomechanical Squeezing, Wigner Function & Phonon Counting Engine.
//!
//! Models back-action evading (BAE) quadrature squeezing below SQL, 2D phase-space Wigner
//! quasi-probability distributions, non-classical Fock state distributions, and strongly dispersive
//! number-resolved cavity spectroscopy.

pub mod phonon_counting;
pub mod squeezing_engine;

pub use phonon_counting::{
    FockStateDistribution, NonClassicalityMetrics, PhononCountingResolvedSpectrum, PhononStateKind,
    ResolvedPeak,
};
pub use squeezing_engine::{
    OptomechanicalSqueezingParams, QuadratureSqueezingSolver, QuadratureVariance,
    WignerQuasiProbability, HBAR, SQL_VARIANCE,
};
