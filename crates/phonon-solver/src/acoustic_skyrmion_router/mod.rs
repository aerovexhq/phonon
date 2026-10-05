#![deny(unsafe_code)]

//! Topological Acoustic Skyrmion Vortex Lattice & Chiral Domain Wall Router.
//!
//! Provides:
//! - 2D pseudospin vector fields n(x, y) with |n| = 1.0 for single skyrmions,
//!   hexagonal vortex crystals (triple-q state), square lattices, and domain walls.
//! - Continuous and discrete topological charge density calculators and invariant verification.
//! - Thiele skyrmion center-of-mass dynamics under acoustic gradient driving force with Hall deflection.
//! - Chiral domain wall acoustic router with unidirectional edge transmission and defect immunity.

pub mod chiral_domain_wall;
pub mod skyrmion_lattice;

pub use chiral_domain_wall::{
    ChiralDomainWallRouter, DefectTransmissionResult, DomainWallDefect, SParameterPoint,
    SParameterSpectrum, ThieleDynamics, TrajectoryPoint,
};
pub use skyrmion_lattice::{
    SkyrmionLatticeParams, SkyrmionLatticeType, TopologicalChargeCalculator, Vector3Field,
};
