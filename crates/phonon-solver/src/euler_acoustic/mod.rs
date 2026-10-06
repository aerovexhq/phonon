#![deny(unsafe_code)]

//! Non-Abelian Euler Class Topological Acoustic Multiband Metamaterial Engine.
//!
//! Provides:
//! - 3-band real symmetric Hamiltonian eigensolver under spacetime inversion C2*T.
//! - Quantized Euler invariant chi in Z across the 2D Brillouin zone.
//! - Patch Euler invariant chi_p and non-Abelian frame rotation across multi-gap nodal braidings.
//! - Topologically protected in-gap acoustic edge states with boundary localization >= 80%.
//! - Domain wall wave packet routing and non-Abelian topological classification.

pub mod euler_lattice;
pub mod edge_transport_engine;

pub use euler_lattice::{
    solve_real_symmetric_3x3, EulerCurvaturePoint, EulerLatticeSolver, EulerParams, EulerPhase,
};
pub use edge_transport_engine::{
    EulerEdgeTransportEngine, EulerRibbonMode, EulerTransportMetrics, RibbonDispersionPoint,
    RibbonParams,
};
