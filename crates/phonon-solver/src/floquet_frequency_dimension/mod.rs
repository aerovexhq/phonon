#![deny(unsafe_code)]

//! Topological Acoustic Floquet-Bloch Synthetic Frequency Dimension & Frequency-Lattice Soliton Engine.
//!
//! Provides models for synthetic frequency dimensions, dynamic boundary phase modulations,
//! Floquet-Bloch mode expansions, synthetic frequency lattice hopping dynamics,
//! chiral frequency-space edge currents, and non-linear frequency-lattice solitons.

pub mod frequency_lattice;
pub mod soliton_transport;

pub use frequency_lattice::{
    solve_jacobi_symmetric, BoundaryModulationParams, Complex, FloquetBandPoint,
    SyntheticFrequencyLattice, SyntheticLatticeKind,
};
pub use soliton_transport::{
    FloquetFrequencyEngine, FrequencyConversionMetrics, FrequencyModeState, FrequencySolitonParams,
    FrequencyWavepacketProfile, SolitonRegime,
};
