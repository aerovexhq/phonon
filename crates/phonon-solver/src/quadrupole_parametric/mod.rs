#![deny(unsafe_code)]

//! Topological Higher-Order Acoustic Quadrupole Parametric Waveguide & Second-Harmonic Generation Module.
//!
//! Provides:
//! - 2D Benalcazar-Bernevig-Hughes (BBH) acoustic quadrupole metamaterial waveguide with quantized quadrupole bulk moment q_xy.
//! - Non-linear second-harmonic generation (SHG) coupled-mode propagation engine along topological edge channels.
//! - Directional traveling-wave parametric amplification with high non-reciprocal isolation (>= 25 dB) and quantum-limited added noise.

pub mod parametric_amplifier;
pub mod quadrupole_waveguide;
pub mod shg_engine;

pub use parametric_amplifier::{
    ParametricAmplifierMetrics, ParametricDriveParams, ParametricEdgeAmplifier,
    ParametricGainSample,
};
pub use quadrupole_waveguide::{
    BbhBandPoint, BoundaryDispersionPoint, QuadrupoleWaveguide, QuadrupoleWaveguideParams,
};
pub use shg_engine::{ShgParams, ShgPhaseMatchSample, ShgSolver, ShgStepPoint};
