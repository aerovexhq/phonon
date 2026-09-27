//! Phonon Core: fundamental data structures, graph topology, physical constants,
//! and error definitions for the Phonon electro-thermal simulator.

pub mod constants;
pub mod digital;
pub mod error;
pub mod graph;
pub mod types;

pub use constants::*;
pub use digital::{DigitalEvent, DigitalNodeId, EventQueue, LogicLevel};
pub use error::CoreError;
pub use graph::{AtomisticChannelType, CircuitGraph, ComponentRecord};

pub use types::{
    format_si, BranchId, ComponentId, MemristorState, NeuronSpike, NodeId, OpticalPortId,
    OpticalSignal, RadiationEvent, TmrOutput,
};
