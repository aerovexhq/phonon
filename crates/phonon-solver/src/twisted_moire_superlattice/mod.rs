#![deny(unsafe_code)]

//! Twisted Bilayer Moiré Phonon Polariton Superlattice Module.
//!
//! Provides continuum elasticity relaxation, dynamic strain tensor calculations,
//! mini-Brillouin zone flat polariton band structures, Van Hove singularity density of states,
//! and localized acoustic soliton profiles.

pub mod continuum_elasticity;
pub mod flat_bands;

pub use continuum_elasticity::{
    AtomicRelaxationField, BilayerLatticeParams, StackingClassification, StrainTensor,
};
pub use flat_bands::{
    DensityOfStates, DosPoint, HighSymmetryKPoint, LocalizedAcousticSoliton, MoireBandPoint,
    MoireBandStructure,
};
