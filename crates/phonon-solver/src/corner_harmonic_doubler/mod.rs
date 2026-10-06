#![deny(unsafe_code)]

//! Corner-to-Edge Non-Linear Coupling, Frequency Doubler & Topological Routing Engine.
//!
//! Provides:
//! - 2D quadrupole SOTI tight-binding real-space lattice with 0D localized corner states and 1D boundary modes.
//! - Non-linear hierarchical overlap integral I_corner_edge between fundamental corner and second-harmonic edge mode.
//! - Resonant acoustic frequency doubler (second-harmonic generation) cavity rate equation solver.
//! - Dynamic transient buildup, steady-state conversion efficiency (>= 30%), and harmonic spectral purity (>= 30 dB).
//! - Backscattering-immune 90-degree corner router with scattering matrix [S] (T >= 95%, return loss <= -25 dB).
//! - Defect immunity around vacancy obstacles and multi-port reconfigurable beam steering (isolation >= 25 dB).

pub mod corner_coupling;
pub mod frequency_doubler;
pub mod topological_router;

pub use corner_coupling::{
    CornerCouplingParams, CornerEigenstate, CornerId, CornerToEdgeLattice,
    CornerToEdgeLatticeResult, EdgeEigenstate,
};
pub use frequency_doubler::{
    DoublerParams, DoublerSteadyState, DoublerTransientPoint, HarmonicSpectrumPoint,
    NonlinearFrequencyDoubler,
};
pub use topological_router::{
    CornerBendAngle, CornerTopologicalRouter, PortTelemetry, RouterParams,
    RouterTargetPort, RoutingWaveField, ScatteringMatrix,
};
