//! Mixed-signal co-simulation boundary bridges and behavioral digital primitives.

pub mod bridges;
pub mod logic_gates;

pub use bridges::{A2dBridge, D2aBridge, D2aCompanion};
pub use logic_gates::{
    DFlipFlop, DigitalNetwork, LogicGate, LogicGateType, PeriodicClock, SarController,
};
