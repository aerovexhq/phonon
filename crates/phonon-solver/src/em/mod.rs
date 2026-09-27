//! High-Performance Electromagnetic Wave & Space Link Solvers.

pub mod antenna_solver;
pub mod em_benchmark;
pub mod em_wave_solver;
pub mod emitter_benchmark;
pub mod protocol_benchmark;
pub mod rf_tier_engine;
pub mod rf_transceiver_solver;
pub mod space_link_solver;
pub mod wifi_protocol_solver;

pub use antenna_solver::{AntennaElectrodynamicSolver, ObservationPoint, RadiationSphereResult};
pub use em_benchmark::{EmBenchmarkReport, EmBenchmarkRunner};
pub use em_wave_solver::{EmLinkResult, EmPropagationScene, EmWaveSolver};
pub use emitter_benchmark::{EmitterBenchmarkReport, EmitterBenchmarkRunner};
pub use protocol_benchmark::{ProtocolBenchmarkReport, ProtocolBenchmarkRunner};
pub use rf_tier_engine::{LinkEvaluationParams, RfRealismTier, RfTierEngine, TierChannelResult};
pub use rf_transceiver_solver::{RfTransceiverSolver, TransceiverLinkResult, TransientBurstResult};
pub use space_link_solver::{GroundStation, SatelliteNode, SpaceLinkBudgetResult, SpaceLinkSolver};
pub use wifi_protocol_solver::{PacketTransmissionResult, WifiLinkSimulator};
