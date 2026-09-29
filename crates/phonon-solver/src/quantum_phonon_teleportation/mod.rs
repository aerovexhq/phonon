//! Multi-physics solver and parallel benchmark suite for quantum phonon-mediated
//! superconducting qubit teleportation, state transfer, and remote Bell state creation.

pub mod teleportation_benchmark;
pub mod teleportation_solver;

pub use teleportation_benchmark::{
    PhononTeleportationBenchmarkResult, PhononTeleportationBenchmarkRunner,
};
pub use teleportation_solver::PhononTeleportationSolver;
