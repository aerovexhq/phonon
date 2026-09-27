//! Phonon headless CLI engine library.

#![deny(unsafe_code)]

pub mod commands;
pub mod distributed;
pub mod error;
pub mod telemetry;

pub use commands::*;
pub use distributed::{
    ColumnStatistics, ColumnarRecordBatch, DistributedCoordinator, SimulationTask, SweepParameter,
    SweepResultSummary,
};
pub use error::CliError;
pub use telemetry::{CsvTelemetryWriter, JsonLinesTelemetryWriter, OutputFormat, TelemetryWriter};
