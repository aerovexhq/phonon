//! Multi-physics solver and parallel benchmark suite for quantum acoustic waveguide QED,
//! chiral phonon-atom bound states, and multi-qubit entanglement.

pub mod waveguide_qed_benchmark;
pub mod waveguide_qed_solver;

pub use waveguide_qed_benchmark::{WaveguideQedBenchmarkResult, WaveguideQedBenchmarkRunner};
pub use waveguide_qed_solver::QuantumAcousticWaveguideSolver;
