#![deny(unsafe_code)]

//! Fractional Quantum Hall Anyon Braiding & Non-Abelian Topological Circuit Engine.
//!
//! Provides non-Abelian anyon braiding matrices, conformal blocks, topological gate synthesis,
//! and edge state Aharonov-Bohm and shot noise interferometry.

pub mod braiding_engine;
pub mod edge_interferometer;

pub use braiding_engine::{
    AnyonModelKind, BraidGenerator, BraidSequence, Complex, ComplexMatrix2x2, ComplexMatrix4x4,
    SynthesisResult, TargetGate, TopologicalGateSynthesizer,
};
pub use edge_interferometer::{
    FillingFraction, FqhEdgeInterferometer, InterferometerType, BOLTZMANN_CONSTANT_KB,
    CONDUCTANCE_QUANTUM_G0, ELEMENTARY_CHARGE_E, FLUX_QUANTUM_PHI0, PLANCK_CONSTANT_H,
};
