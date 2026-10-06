#![deny(unsafe_code)]

//! Quantum Spin-Torque Oscillator & Magnetic Skyrmion Reservoir Computing Module.
//!
//! Exposes the Landau-Lifshitz-Gilbert-Slonczewski (LLGS) dynamical engine,
//! 2D magnetic chiral skyrmion textures with artificial synaptic pinning,
//! and the virtual-node spintronic reservoir computing framework.

pub mod llgs_engine;
pub mod reservoir_computer;
pub mod skyrmion_texture;

pub use llgs_engine::{
    EffectiveField, LlgsParams, SpinTorqueOscillator, Vector3, ELEMENTARY_CHARGE,
    GYROMAGNETIC_RATIO, HBAR, MU_0,
};
pub use reservoir_computer::{
    SpintronicReservoir, SpintronicReservoirParams,
};
pub use skyrmion_texture::{
    MagneticSkyrmionTexture, PinningSite, SkyrmionGridParams,
};
