//! Subcommand execution modules for the Phonon CLI.

pub mod mc;
pub mod run;
pub mod sweep;
pub mod validate;

pub use mc::{execute_monte_carlo, MonteCarloStats};
pub use run::execute_run;
pub use sweep::execute_sweep;
pub use validate::{execute_validate, ValidationReport};
