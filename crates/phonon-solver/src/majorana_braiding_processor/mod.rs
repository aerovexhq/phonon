#![deny(unsafe_code)]

//! Autonomous Topological Majorana Zero-Mode Braiding Processor & Parity Qubit Synthesizer
//! solver module for the Phonon multi-scale visual CAD platform.

pub mod processor_benchmark;
pub mod processor_solver;

pub use processor_benchmark::*;
pub use processor_solver::*;
