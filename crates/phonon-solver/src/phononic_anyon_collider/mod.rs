#![deny(unsafe_code)]

//! Multi-physics solver and parallel benchmark suite for quantum phononic
//! non-Abelian anyon colliders and multi-qubit topological braiding interferometers.

pub mod collider_benchmark;
pub mod collider_solver;

pub use collider_benchmark::{ColliderBenchmarkResult, ColliderBenchmarkRunner};
pub use collider_solver::PhononicAnyonColliderSolver;
