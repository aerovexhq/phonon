//! Error types for sparse linear algebra and MNA circuit solving.

use phonon_core::CoreError;
use thiserror::Error;

/// Numerical and topological errors encountered during circuit solving.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum SolverError {
    #[error("Singular matrix detected at step {step}: zero pivot ({pivot_value:e}) at [{row}, {col}]. Offending circuit entity: {entity_diagnostic}")]
    SingularMatrix {
        step: usize,
        row: usize,
        col: usize,
        pivot_value: f64,
        entity_diagnostic: String,
    },

    #[error("Matrix dimension mismatch: expected {expected}x{expected}, found {found_rows}x{found_cols}")]
    DimensionMismatch {
        expected: usize,
        found_rows: usize,
        found_cols: usize,
    },

    #[error("Vector dimension mismatch: expected length {expected}, found {found}")]
    VectorDimensionMismatch { expected: usize, found: usize },

    #[error("Numerical anomaly during solver execution: {detail}")]
    NumericalAnomaly { detail: String },

    #[error("Ill-conditioned matrix: condition ratio {ratio:e} exceeds maximum allowable threshold ({max_ratio:e})")]
    IllConditionedMatrix { ratio: f64, max_ratio: f64 },

    #[error("Core circuit error: {0}")]
    Core(#[from] CoreError),
}
