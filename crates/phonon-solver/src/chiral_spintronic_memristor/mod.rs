//! Chiral phonon-driven spintronic memristors and neuromorphic crossbars.

pub mod spintronic_memristor_benchmark;
pub mod spintronic_memristor_solver;

pub use spintronic_memristor_benchmark::{
    SpintronicMemristorBenchmarkReport, SpintronicMemristorBenchmarkRunner,
    SpintronicMemristorSweepPoint,
};
pub use spintronic_memristor_solver::SpintronicMemristorSolver;
