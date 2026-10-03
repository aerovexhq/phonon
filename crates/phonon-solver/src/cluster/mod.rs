#![deny(unsafe_code)]

//! Distributed Cloud Parameter Sweep Cluster Engine for Phonon Visual Studio.
//!
//! Provides distributed RPC protocol framing, work-stealing scheduling across
//! heterogeneous compute topologies, fault-tolerant batch re-queueing upon node failure,
//! and real-time yield harvesting.

pub mod protocol;
pub mod scheduler;
pub mod worker;

pub use protocol::{
    NodeStatus, ProtocolError, RpcMessage, SimulationType, SweepSample, WorkerNodeInfo,
    PROTOCOL_MAGIC, PROTOCOL_VERSION,
};
pub use scheduler::{ClusterDispatchQueue, ClusterSweepResult};
pub use worker::SimulatedClusterWorker;
