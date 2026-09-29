#![deny(unsafe_code)]

//! Non-Hermitian skin-topological phonon diodes and unidirectional quantum acoustic amplifiers.

mod skin_amplifier_benchmark;
mod skin_amplifier_solver;

pub use skin_amplifier_benchmark::{SkinAmplifierBenchmarkResult, SkinAmplifierBenchmarkRunner};
pub use skin_amplifier_solver::NonHermitianSkinAmplifierSolver;
