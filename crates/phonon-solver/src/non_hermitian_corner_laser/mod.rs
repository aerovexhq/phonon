#![deny(unsafe_code)]

//! Non-Hermitian Higher-Order Topological (HOT) Corner Laser & Spectral Singularity Engine.
//!
//! Provides models for non-Hermitian gain-loss metamaterials, higher-order topological
//! corner lasing, exceptional points and spectral singularities, single-mode emission,
//! and backscattering immunity.

pub mod hot_laser_lattice;
pub mod laser_dynamics;

pub use hot_laser_lattice::{
    Complex as CornerLaserComplex, HotLaserEigenmode, HotLaserModeKind, LaserLatticeKind,
    LaserLatticeParams, NonHermitianHotLattice,
};
pub use laser_dynamics::{LaserEmissionMetrics, NonHermitianCornerLaserEngine};

pub type CornerLaserLatticeKind = LaserLatticeKind;
pub type CornerLaserLatticeParams = LaserLatticeParams;
