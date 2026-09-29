#![deny(unsafe_code)]

//! Quantum Acoustic Models: SAW cavities, interdigital transducers, transmon cQAD coupling,
//! and virtual phonon-mediated entanglement.

pub mod saw_cavity;
pub mod saw_qubit;

pub use saw_cavity::{
    BraggAcousticMirror, InterdigitalTransducer, SawCavity, SawSubstrateMaterial,
    ELEMENTARY_CHARGE, EPSILON_0, HBAR,
};
pub use saw_qubit::{SawBeamSplitter, SawQubitCoupling, TransmonQubit, VirtualPhononBus};
