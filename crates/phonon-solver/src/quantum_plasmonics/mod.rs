#![deny(unsafe_code)]

//! Quantum Plasmonic Solvers: Maxwell-Bloch integrators, sub-diffraction waveguide routing,
//! and high-performance parallel benchmarks.

pub mod maxwell_bloch_solver;
pub mod plasmonic_benchmark;
pub mod subdiffraction_routing_solver;

pub use maxwell_bloch_solver::{BlochVector, PlasmonicMaxwellBlochSolver};
pub use plasmonic_benchmark::{
    run_quantum_plasmonic_benchmark, PlasmonicSweepResult, QuantumPlasmonicBenchmarkReport,
};
pub use subdiffraction_routing_solver::{
    evaluate_plasmonic_directional_coupler, evaluate_transistor_logic, evaluate_waveguide_bend,
    PlasmonicCouplerReport, TransistorLogicReport, WaveguideBendReport,
};
