//! Phononic metamaterial continuum solvers, acoustic logic wave propagation, and benchmarks.

pub mod continuum_solver;
pub mod logic_solver;
pub mod phononic_benchmark;

pub use continuum_solver::{ContinuumNode, ContinuumSolver2D};
pub use logic_solver::{
    AcousticGateVerificationResult, AcousticLogicSolver, AcousticWaveformTrace,
};
pub use phononic_benchmark::{PhononicBenchmarkReport, PhononicBenchmarkRunner};
