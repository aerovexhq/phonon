#![deny(unsafe_code)]

//! Topological Acoustic Quadrupole Second-Harmonic Generation Metamaterial module.
//!
//! Provides:
//! - Benalcazar-Bernevig-Hughes (BBH) higher-order quadrupole topological lattice solver.
//! - Quantized quadrupole moment q_xy and corner mode localization decay length.
//! - Non-linear modal overlap integral kappa_SHG in corner nanocavities.
//! - Steady-state second-harmonic generation (SHG) power conversion engine.
//! - Dual-harmonic discrete spectrum, power saturation, and topological disorder immunity.

pub mod quadrupole_shg_lattice;
pub mod shg_emission_engine;

pub use quadrupole_shg_lattice::{
    CornerShgModalMetrics, QuadrupoleShgParams, QuadrupoleShgSolver,
};
pub use shg_emission_engine::{
    ShgEmissionEngine, ShgEmissionMetrics, ShgEmissionParams, ShgHarmonicSpectrumPoint,
};
