//! Atomic relay and electrochemical logic multi-physics solver, autonomous synthesis, and benchmark engine.
//!
//! Provides:
//! - Multi-physics coupled electro-mechanical-thermal transient solver (`coupled_solver`).
//! - Autonomous minimal-component logic synthesizer (`relay_synthesis`).
//! - Multi-threaded Rayon comparative benchmark engine (`relay_benchmark`).

pub mod coupled_solver;
pub mod relay_benchmark;
pub mod relay_synthesis;

pub use coupled_solver::{CoupledRelaySolver, CoupledRelayTransientResult, CoupledSolverConfig};
pub use relay_benchmark::{RelayBenchmarkReport, RelayBenchmarkRunner};
pub use relay_synthesis::{AutonomousRelaySynthesizer, RelaySynthesisTarget, SynthesizedRelayGate};
