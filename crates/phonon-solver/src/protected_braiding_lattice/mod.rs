#![deny(unsafe_code)]

//! Quantum Acoustic Protected Braiding Lattice Solver & Surface Code Stabilizer Module.

pub mod braiding_lattice;
pub mod surface_stabilizer;

pub use braiding_lattice::{
    BraidStep, CompiledBraidResult, Complex as BraidingComplex, MajoranaBraidingParams,
    MajoranaMode, MajoranaTargetGate, NonAbelianBraidGenerator, ParityReadout, TargetGate,
    UnitaryMatrix, HBAR_J_S,
};
pub use surface_stabilizer::{
    CorrectionResult, StabilizerCheck, StabilizerKind, SurfaceCodeGrid, SyndromeResult,
    XorShiftRng,
};
