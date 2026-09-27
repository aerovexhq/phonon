//! Error types for SPICE netlist lexing, parsing, and elaboration.

use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum NetlistError {
    #[error("Parse error at line {line}: {message}")]
    SyntaxError { line: usize, message: String },

    #[error("Invalid engineering number literal '{literal}' at line {line}: {detail}")]
    InvalidNumber {
        line: usize,
        literal: String,
        detail: String,
    },

    #[error("Unknown or undefined model '{model_name}' referenced by component '{component_name}' at line {line}")]
    UndefinedModel {
        line: usize,
        component_name: String,
        model_name: String,
    },

    #[error("Unknown or undefined subcircuit '{subcircuit_name}' instantiated by '{instance_name}' at line {line}")]
    UndefinedSubcircuit {
        line: usize,
        instance_name: String,
        subcircuit_name: String,
    },

    #[error("Subcircuit pin count mismatch for instance '{instance_name}': expected {expected}, found {found} at line {line}")]
    PinCountMismatch {
        line: usize,
        instance_name: String,
        expected: usize,
        found: usize,
    },

    #[error("Subcircuit recursion depth limit exceeded (maximum {max_depth}) while expanding '{subcircuit_name}'")]
    RecursionDepthExceeded {
        subcircuit_name: String,
        max_depth: usize,
    },

    #[error("Core graph error during netlist elaboration: {0}")]
    Core(#[from] phonon_core::CoreError),
}
