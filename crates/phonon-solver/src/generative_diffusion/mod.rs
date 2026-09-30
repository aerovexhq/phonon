#![deny(unsafe_code)]

//! Generative inverse-design diffusion engine and automated metamaterial synthesizer solver module.

pub mod diffusion_benchmark;
pub mod diffusion_solver;

pub use diffusion_benchmark::*;
pub use diffusion_solver::*;
