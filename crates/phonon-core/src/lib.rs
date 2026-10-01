#![deny(unsafe_code)]

//! Phonon Core: fundamental data structures, graph topology, physical constants,
//! error definitions, and abstract physics dynamics interfaces.

pub mod constants;
pub mod digital;
pub mod dynamics;
pub mod dynamics_reference;
pub mod error;
pub mod graph;
pub mod types;

pub use constants::*;
pub use digital::{DigitalEvent, DigitalNodeId, EventQueue, LogicLevel};
pub use dynamics::{
    ActuatorInputs, BackendInfo, DynamicsError, DynamicsTelemetry, PhysicsDynamicsBackend,
};
pub use dynamics_reference::{ReferenceDynamicsBackend, ReferenceDynamicsParams};
pub use error::CoreError;
pub use graph::{AtomisticChannelType, CircuitGraph, ComponentRecord};

pub use types::{
    format_si, BranchId, ComponentId, MemristorState, NeuronSpike, NodeId, OpticalPortId,
    OpticalSignal, RadiationEvent, TmrOutput,
};
