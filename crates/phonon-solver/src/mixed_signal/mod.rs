#![deny(unsafe_code)]

//! Mixed-signal co-simulation synchronization kernel.

pub mod co_solver;
pub mod digital_engine;
pub mod hdl_parser;
pub mod synchronizer;

pub use co_solver::{
    solve_mixed_signal, DigitalTraceStep, MixedSignalCircuit, MixedSignalOptions,
    MixedSignalSolution,
};
pub use digital_engine::{DigitalEngine, DigitalEvent, DigitalLogicGate, GateKind, LogicState};
pub use hdl_parser::{parse_verilog_module, HdlParseError, ParsedHdlModule};
pub use synchronizer::{
    BoundaryAdc, BoundaryDac, DacSmoothing, MixedSignalStepReport, MixedSignalSynchronizer,
};
