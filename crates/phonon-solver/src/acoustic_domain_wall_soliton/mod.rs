#![deny(unsafe_code)]

pub mod sine_gordon_solver;
pub mod waveguide_engine;

pub use sine_gordon_solver::{
    SineGordonParams, SineGordonSolver, SineGordonState, SolitonKind,
};
pub use waveguide_engine::{
    DomainWallWaveguideParams, DomainWallWaveguideRouter, WaveguideSParameters,
};
