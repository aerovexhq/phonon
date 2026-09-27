//! Multi-core parallel logic gate synthesis and transient verification.

pub mod topology_evolver;
pub mod transient_verifier;

pub use topology_evolver::{SynthesisConfig, SynthesizedCandidate, TopologyEvolver};
pub use transient_verifier::{TransientGateVerifier, TransientVerificationResult};
