#![deny(unsafe_code)]

//! Quantum Plasmonic Models: non-local hydrodynamic electron gas, surface plasmon
//! polaritons, MIM slot waveguides, and all-optical single-photon transistors.

pub mod drude_hydrodynamic;
pub mod plasmonic_transistor;

pub use drude_hydrodynamic::{
    NobleMetal, SppHydrodynamicModel, ELEMENTARY_CHARGE, EPSILON_0, HBAR, SPEED_OF_LIGHT,
};
pub use plasmonic_transistor::{PlasmonicSlotWaveguide, QuantumEmitter, SinglePhotonTransistor};
