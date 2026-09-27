//! Error representations across the core circuit modeling domain.

use crate::types::NodeId;
use thiserror::Error;

/// Core errors arising from circuit construction, graph topology, and physical constraints.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum CoreError {
    #[error("Node index {0} is out of bounds")]
    InvalidNodeIndex(usize),

    #[error("Node with name '{0}' was not found in the circuit")]
    NodeNotFound(String),

    #[error("Component with name '{0}' was not found in the circuit")]
    ComponentNotFound(String),

    #[error("Component with name '{0}' is already registered")]
    DuplicateComponent(String),

    #[error("Node {node_id} ('{name}') has no DC path to ground or reference (floating node)")]
    FloatingNode { node_id: NodeId, name: String },

    #[error(
        "Closed loop of ideal voltage sources detected connecting node {node_a} and node {node_b}"
    )]
    VoltageSourceLoop { node_a: NodeId, node_b: NodeId },

    #[error("Component '{0}' has zero or invalid resistance ({1} Ohms), creating a short circuit")]
    InvalidResistance(String, f64),

    #[error("Circuit is empty: no components or nodes are defined")]
    EmptyCircuit,

    #[error("Circuit contains disconnected subnetworks ({count} isolated partitions)")]
    DisconnectedSubnetworks { count: usize },

    #[error("Component '{0}' has invalid value ({1})")]
    InvalidValue(String, f64),

    #[error("Numerical anomaly detected: {detail}")]
    NumericalAnomaly { detail: String },
}
