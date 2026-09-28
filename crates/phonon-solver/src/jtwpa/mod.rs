//! Superconducting Traveling-Wave Parametric Amplifier (JTWPA) solvers:
//! spatial coupled-mode propagation, quantum noise analysis, and parallel Rayon benchmarks.

pub mod coupled_mode_solver;
pub mod jtwpa_benchmark;
pub mod quantum_noise_solver;

pub use coupled_mode_solver::{CoupledModeResult, CoupledModeSolver};
pub use jtwpa_benchmark::{JtwpaBenchmarkReport, JtwpaBenchmarkRunner};
pub use quantum_noise_solver::{QuantumNoiseResult, QuantumNoiseSolver};
