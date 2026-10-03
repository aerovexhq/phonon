#![deny(unsafe_code)]

//! Topological Acoustic Chern Insulator Chiral Circulator & Non-Reciprocal Router Module.
//!
//! Provides honeycomb acoustic metamaterial lattice models with spinning fluid cylinders,
//! broken acoustic time-reversal symmetry, quantized first Chern numbers C in {-1, +1},
//! 3-port cyclic scattering matrix calculations (S_21, S_31, S_11), and backscattering defect immunity.

pub mod chern_lattice;
pub mod chiral_router;

pub use chern_lattice::{
    BerryCurvaturePoint, ChernLatticeParams, ChernLatticeState, ChiralEdgeMode,
    EdgeDispersionPoint,
};
pub use chiral_router::{
    CirculatorPort, DefectImmunityResult, ObstacleKind, SParameterSpectrum,
    SParameterSpectrumPoint, ScatteringMatrix3x3, ThreePortCirculator,
};
