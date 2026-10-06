#![deny(unsafe_code)]

//! Higher-order topological acoustic octupole insulator and 3D corner state nanocavity engine.
//!
//! Models 3D Benalcazar-Bernevig-Hughes (BBH) octupole tight-binding metamaterial lattices,
//! quantized octupole moment O_xyz = 1/2, bulk pi-flux cubic cells, surface/hinge gapping,
//! and 8 localized zero-dimensional acoustic corner states at mid-gap.

pub mod corner_nanocavity;
pub mod octupole_hamiltonian;

pub use corner_nanocavity::*;
pub use octupole_hamiltonian::*;
