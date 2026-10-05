#![deny(unsafe_code)]

//! Non-Hermitian Skin Effect Acoustic Sensor & Directional Funnel module.
//!
//! Provides:
//! - Hatano-Nelson non-reciprocal lattice model with point-gap winding number.
//! - Generalized Brillouin Zone (GBZ) radius and skin depth calculation.
//! - Non-Hermitian skin effect eigenstate accumulation solver.
//! - Directional acoustic funnel and non-reciprocal transmission S-parameters.
//! - Ultrasensitive EP_N boundary perturbation and mass sensing engine.

pub mod directional_amplifier_solver;
pub mod nhse_benchmark;
pub mod non_bloch_transfer_matrix_solver;
pub mod hatano_nelson;
pub mod directional_funnel;

pub use directional_amplifier_solver::*;
pub use nhse_benchmark::*;
pub use non_bloch_transfer_matrix_solver::*;
pub use hatano_nelson::{
    HatanoNelsonParams, NonHermitianSkinSolver, PointGapTopology,
};
pub use directional_funnel::{
    AcousticFunnelParams, AcousticFunnelSolver, FunnelSParameters,
};
