#![deny(unsafe_code)]

//! Quantum Acoustic Solvers: cQAD Lindblad master equations, remote phonon entanglement,
//! acoustic beam splitters, and parallel benchmarks.

pub mod cqa_master_equation;
pub mod phonon_entanglement_solver;
pub mod quantum_acoustic_benchmark;

pub use cqa_master_equation::{
    Complex64, CqaMasterEquationSolver, QubitPhononDensityMatrix, FOCK_DIM, HILBERT_DIM,
};
pub use phonon_entanglement_solver::{
    evaluate_saw_beam_splitter, HomRoutingReport, PhononEntanglementSolver, TwoQubitDensityMatrix,
    TWO_QUBIT_DIM,
};
pub use quantum_acoustic_benchmark::{
    run_quantum_acoustic_benchmark, QuantumAcousticBenchmarkReport, QuantumAcousticSweepResult,
};
