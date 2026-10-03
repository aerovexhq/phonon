#![deny(unsafe_code)]

//! Non-Abelian Wilczek-Zee Gauge Connection and Tripartite Acoustic Cavity Co-Simulator.
//!
//! Provides:
//! - Wilczek-Zee non-Abelian gauge connection matrices on degenerate tripod dark state manifold.
//! - Wilson loop path ordering and holonomy evaluation along closed parameter loops.
//! - Non-Abelian commutation failure verification [U(C_1), U(C_2)] != 0.
//! - Geometric quantum logic gate synthesis with process fidelity F >= 0.99.
//! - Tripartite cavity pulse co-simulation and dynamical phase cancellation validation (< 1e-4 rad).

pub mod cavity_co_simulator;
pub mod wilczek_zee;

pub use cavity_co_simulator::{
    HolonomicTrajectorySimulation, TripartiteCavityParams, TripartiteCoSimulator,
};
pub use wilczek_zee::{
    Complex as HolonomicComplex, ComplexMatrix2x2 as HolonomicComplexMatrix2x2, DarkSubspace,
    HolonomicGateType, HolonomicSynthesisResult, ParameterLoop, WilczekZeeConnection,
    WilsonLoopIntegrator,
};
