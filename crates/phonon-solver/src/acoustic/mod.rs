//! Multi-Tier Acoustic Propagation Engine & Physical Room Solvers
//!
//! Provides multi-tier acoustic solvers (FDTD full-wave, raytracing multipath,
//! Sabine reverberation time $T_{60}$, and accelerated path loss) and high-throughput
//! parallel Rayon benchmark suites.

pub mod acoustic_benchmark;
pub mod acoustic_tier_engine;

pub use acoustic_benchmark::{AcousticBenchmarkReport, AcousticBenchmarkRunner};
pub use acoustic_tier_engine::{
    AcousticLinkSimulator, AcousticRealismTier, AcousticRoom, AcousticStepResult, FdtdResult,
};
