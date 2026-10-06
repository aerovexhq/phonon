#![deny(unsafe_code)]

//! Topological acoustic moire quasicrystal and Aubry-Andre-Harper (AAH) mobility edge engine.
//!
//! Models quasiperiodic acoustic metamaterials with incommensurate modulation,
//! self-dual Hofstadter butterfly energy spectra, energy-dependent localization-delocalization
//! mobility edges, inverse participation ratio (IPR) fractal scaling, and topologically
//! protected phason edge modes.

pub mod aah_hamiltonian;
pub mod quasicrystal_mobility_engine;

pub use aah_hamiltonian::*;
pub use quasicrystal_mobility_engine::*;
