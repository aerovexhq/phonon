#![deny(unsafe_code)]

//! Autonomous Non-Hermitian Exceptional Surface Sensor & Hypersensitive Phononic Metrology Engine.
//!
//! Provides:
//! - Continuous 2D manifold exceptional surface (ES) parameterization and eigensolver.
//! - Complex Jordan vector decomposition and generalized Petermann factor divergence.
//! - Directional chiral acoustic sensitivity and fractional power-law splitting.
//! - Phased ultrasonic transducer sensor array with forward/backward contrast and SNR analysis.
//! - Multi-physics metrology benchmark runner and physical metric solvers.

pub mod hamiltonian;
pub mod sensing_array;
pub mod surface_benchmark;
pub mod surface_solver;

pub use surface_benchmark::*;
pub use surface_solver::*;

pub use hamiltonian::{
    Complex as EsComplex, EsManifoldParams, ExceptionalSurfaceHamiltonian,
    ExceptionalSurfaceManifoldParams, RiemannSheetPoint, SurfaceEigenvalues,
};
pub use sensing_array::{
    ChiralDirectionalMetrics, ExceptionalSurfaceArray, SensorElement, SnrAnalysis, TransducerType,
};
