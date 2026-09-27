//! Molecular quantum-interference solvers, synthesis, and comparative benchmarking.
//!
//! Modules:
//! - `molecular_solver`: Self-consistent NEGF-Poisson electrostatic solver.
//! - `molecular_synthesis`: Autonomous molecular logic network synthesizer.
//! - `molecular_benchmark`: Parallel multi-threaded comparative benchmark engine vs 3nm CMOS.

pub mod molecular_benchmark;
pub mod molecular_solver;
pub mod molecular_synthesis;

pub use molecular_benchmark::{
    Cmos3nmBaseline, MolecularBenchmarkRunner, MolecularComparisonReport,
};
pub use molecular_solver::{
    SelfConsistentNegfConfig, SelfConsistentNegfResult, SelfConsistentNegfSolver,
};
pub use molecular_synthesis::{
    MolecularCandidate, MolecularLogicSynthesizer, SynthesizedMolecularLogic, TargetLogicFunction,
};
