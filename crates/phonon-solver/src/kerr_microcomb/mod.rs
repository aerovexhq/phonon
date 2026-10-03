#![deny(unsafe_code)]

//! Non-Linear Soliton Kerr Microcomb Phononic Frequency Comb Engine.
//!
//! Provides:
//! 1. Normalized Lugiato-Lefever Equation (LLE) split-step Fourier solver (`LleSplitStepSolver`).
//! 2. Microresonator physical parameter models (`MicroresonatorParams`).
//! 3. Frequency comb spectrum analysis in dBm and multi-threshold bandwidths (`CombSpectrum`).
//! 4. Soliton hyperbolic secant (sech^2) pulse fitting and duration metrics.
//! 5. Detuning sweep hysteresis scan and regime classification (`MicrocombRegime`, `DetuningScanResult`).

pub mod comb_metrics;
pub mod lle_solver;

pub use comb_metrics::CombSpectrum;
pub use lle_solver::{
    fft_1d, DetuningScanResult, LleSplitStepSolver, MicrocombRegime, MicrocombState,
    MicroresonatorParams,
};
