#![deny(unsafe_code)]

//! Topological Acoustic Valley-Hall Vortex Pumping & Synthetic Chiral Gauge Field Engine.
//!
//! Provides models for:
//! - Valley-Hall topological acoustic metamaterials with mass detuning Delta = m_A - m_B.
//! - Valley Berry curvature and quantized valley Chern difference Delta C_v = 1.
//! - Synthetic pseudomagnetic gauge field A_s and discrete pseudo-Landau levels E_n proportional to sqrt(n).
//! - Inverted-mass domain wall waveguides supporting valley-locked gapless edge modes.
//! - Acoustic vortex phase winding exp(i * l * theta) driving quantized topological charge pumping.
//! - Valley router S-parameters with high directivity and defect-immune transmission around sharp corners.

pub mod valley_hamiltonian;
pub mod vortex_pumping_engine;

pub use valley_hamiltonian::{
    AcousticValley, PseudoLandauLevel, ValleyDispersionPoint, ValleyHallParams,
    ValleyHallPhase, ValleyHamiltonian,
};

pub use vortex_pumping_engine::{
    DomainWallKind, PumpingCyclePoint, ValleyRibbonMode, ValleyRibbonParams,
    ValleyRouterMetrics, VortexPumpingEngine,
};
