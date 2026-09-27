//! High-Performance Electromagnetic Wave & Space Link Solvers.

pub mod antenna_solver;
pub mod em_benchmark;
pub mod em_wave_solver;
pub mod emitter_benchmark;
pub mod rf_transceiver_solver;
pub mod space_link_solver;

pub use antenna_solver::{AntennaElectrodynamicSolver, ObservationPoint, RadiationSphereResult};
pub use em_benchmark::{EmBenchmarkReport, EmBenchmarkRunner};
pub use em_wave_solver::{EmLinkResult, EmPropagationScene, EmWaveSolver};
pub use emitter_benchmark::{EmitterBenchmarkReport, EmitterBenchmarkRunner};
pub use rf_transceiver_solver::{RfTransceiverSolver, TransceiverLinkResult, TransientBurstResult};
pub use space_link_solver::{GroundStation, SatelliteNode, SpaceLinkBudgetResult, SpaceLinkSolver};
