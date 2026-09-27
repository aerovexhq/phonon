//! High-Performance Electromagnetic Wave & Space Link Solvers.

pub mod em_benchmark;
pub mod em_wave_solver;
pub mod space_link_solver;

pub use em_benchmark::{EmBenchmarkReport, EmBenchmarkRunner};
pub use em_wave_solver::{EmLinkResult, EmPropagationScene, EmWaveSolver};
pub use space_link_solver::{GroundStation, SatelliteNode, SpaceLinkBudgetResult, SpaceLinkSolver};
