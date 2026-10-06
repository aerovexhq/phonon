#![deny(unsafe_code)]

//! Topological Acoustic Higher-Order Skyrmion-Lattice Beam Deflector & Chiral Spin-Orbit Angle Router.
//!
//! Provides mathematical models, real-space spin vector textures, topological winding density,
//! and anomalous acoustic Hall beam deflection simulation engines.

pub mod skyrmion_texture;
pub mod deflector_engine;

pub use skyrmion_texture::{
    SkyrmionProfileKind, SkyrmionTexture, SkyrmionTextureParams, SpinVector,
};
pub use deflector_engine::{
    AcousticPseudoSpin, DeflectedBeamResult, DeflectorParams, SkyrmionDeflectorEngine,
    SkyrmionDeflectorMetrics,
};
