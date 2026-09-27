//! Error types for the Phonon headless CLI engine.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Netlist error: {0}")]
    Netlist(#[from] phonon_netlist::NetlistError),

    #[error("Core graph error: {0}")]
    Core(#[from] phonon_core::CoreError),

    #[error("Solver error: {0}")]
    Solver(#[from] phonon_solver::error::SolverError),

    #[error("Execution error: {0}")]
    Execution(String),
}
