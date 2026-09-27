//! Distributed sweep orchestration, multi-core worker dispatch, and columnar telemetry.

pub mod coordinator;
pub mod telemetry_arrow;

pub use coordinator::{DistributedCoordinator, SimulationTask, SweepParameter, SweepResultSummary};
pub use telemetry_arrow::{ColumnStatistics, ColumnarRecordBatch};
