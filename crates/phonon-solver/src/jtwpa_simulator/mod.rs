#![deny(unsafe_code)]

//! Superconducting Josephson Traveling-Wave Parametric Amplifier (JTWPA) Simulator.
//!
//! Provides discrete non-linear Josephson transmission line physics,
//! four-wave mixing (4WM) coupled-mode parametric amplification,
//! resonant phase matching (RPM) dispersion engineering,
//! and quantum quadrature squeezing analysis.

pub mod four_wave_mixing;
pub mod transmission_line;

pub use four_wave_mixing::{GainSpectrumPoint, JtwpaParams, JtwpaSolver, QuantumSqueezing};
pub use transmission_line::{JosephsonCellParams, JosephsonTransmissionLine, RpmStubParams};
