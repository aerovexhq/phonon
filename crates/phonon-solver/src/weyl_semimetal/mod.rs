#![deny(unsafe_code)]

//! Universal Topological Dirac and Weyl Semimetal Metamaterial Simulator.
//!
//! Provides 3D Weyl and Dirac dispersion solvers, Berry curvature monopole metrics,
//! open surface Fermi arc contours on (001) surfaces, and chiral anomaly magnetotransport.

pub mod dispersion;
pub mod fermi_arc;

pub use dispersion::{BandPoint, WeylNode, WeylNodeType, WeylSemimetalModel};
pub use fermi_arc::{BeamSplitterResult, ChiralAnomalyTransport, FermiArcPoint, FermiArcSurface};
