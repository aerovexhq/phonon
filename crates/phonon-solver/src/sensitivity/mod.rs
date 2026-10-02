#![deny(unsafe_code)]

//! Non-linear transient sensitivity analysis, adjoint engine, and worst-case circuit optimization.

pub mod adjoint_engine;
pub mod worst_case_optimizer;

pub use adjoint_engine::{
    AdjointSensitivityEngine, CircuitParameter, ObjectiveKind, SensitivityResult,
};
pub use worst_case_optimizer::{
    CornerEvaluation, CornerType, WorstCaseOptimizer, WorstCaseSummary,
};
