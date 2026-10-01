#![deny(unsafe_code)]

//! Phonon Core: fundamental data structures, graph topology, physical constants,
//! error definitions, abstract physics dynamics interfaces, and Aerovex presence / SHM connectors.

pub mod constants;
pub mod digital;
pub mod dynamics;
pub mod dynamics_reference;
pub mod dynamics_shm;
pub mod error;
pub mod graph;
pub mod probe;
pub mod types;

pub use constants::*;
pub use digital::{DigitalEvent, DigitalNodeId, EventQueue, LogicLevel};
pub use dynamics::{
    ActuatorInputs, BackendInfo, DynamicsError, DynamicsTelemetry, PhysicsDynamicsBackend,
};
pub use dynamics_reference::{ReferenceDynamicsBackend, ReferenceDynamicsParams};
pub use dynamics_shm::{
    create_mock_shm_buffer, safe_read_shm_slot, AerovexShmBackend, AutoSelectingDynamicsBackend,
    ShmSlotData, AVSM_DEFAULT_SLOT_STRIDE, AVSM_ENTITY_SIZE, AVSM_HEADER_SIZE, AVSM_SLOT_META_SIZE,
};
pub use error::CoreError;
pub use graph::{AtomisticChannelType, CircuitGraph, ComponentRecord};
pub use probe::{
    AerovexPresenceProbe, ProbeStatus, AVSM_MAGIC, DEFAULT_SHM_PATH, MAX_HEARTBEAT_AGE_MS,
};
pub use types::{
    format_si, BranchId, ComponentId, MemristorState, NeuronSpike, NodeId, OpticalPortId,
    OpticalSignal, RadiationEvent, TmrOutput,
};
