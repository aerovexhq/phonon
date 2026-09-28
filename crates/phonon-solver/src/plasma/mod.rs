//! Autonomous high-energy plasma dynamics, tokamak fusion magnetics, and Alfvén wave co-simulation.

pub mod alfven_mhd_stepper;
pub mod boris_pic_tracker;
pub mod grad_shafranov_solver;
pub mod tokamak_benchmark;

pub use alfven_mhd_stepper::{AlfvenFluxNode, AlfvenMhdConfig, AlfvenMhdStepper};
pub use boris_pic_tracker::{BorisPicTracker, OrbitTopology, ParticleOrbitReport};
pub use grad_shafranov_solver::{GradShafranovGrid, GradShafranovSolution, GradShafranovSolver};
pub use tokamak_benchmark::{TokamakBenchmarkReport, TokamakBenchmarkRunner, TokamakScenario};
