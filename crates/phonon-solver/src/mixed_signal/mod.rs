//! Mixed-signal co-simulation synchronization kernel.

pub mod co_solver;

pub use co_solver::{
    solve_mixed_signal, DigitalTraceStep, MixedSignalCircuit, MixedSignalOptions,
    MixedSignalSolution,
};
