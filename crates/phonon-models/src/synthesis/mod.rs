//! Unconstrained Gate Topology Synthesis, Non-CMOS Logic & Direct Material Actions.
//!
//! Provides circuit topology representations, truth table specifications, noise margin evaluators,
//! and non-transistor direct material switching actions (NDR/RTD and MIT VO2).

pub mod circuit_metrics;
pub mod material_switches;
pub mod topology_graph;
pub mod truth_table;

pub use circuit_metrics::{CircuitMetricsEvaluator, GateMetrics};
pub use material_switches::{
    MitDeviceModel, MitEvaluation, MitParameters, NdrDeviceModel, NdrEvaluation, NdrParameters,
};
pub use topology_graph::{CircuitTopology, GateElement, GateNode};
pub use truth_table::{TruthTable, TruthTableRow};
