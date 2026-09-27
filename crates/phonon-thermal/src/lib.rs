//! # Phonon Thermal Engine
//!
//! Multi-physics electro-thermal solver engine, spatial Fourier heat diffusion grids,
//! lumped Cauer/Foster ladder networks, and dynamic self-heating simulators.

pub mod boundary;
pub mod cauer;
pub mod foster;
pub mod grid2d;
pub mod grid3d;
pub mod monolithic;
pub mod relaxation;
pub mod runaway;

pub use boundary::{ThermalBoundary, STEFAN_BOLTZMANN};
pub use cauer::{CauerNetwork, CauerStage};
pub use foster::{FosterNetwork, FosterStage};
pub use grid2d::{silicon_thermal_conductivity, ThermalGrid2D, CV_SI_300K, KAPPA_SI_300K};
pub use grid3d::ThermalGrid3D;
pub use monolithic::{solve_electrothermal_dc, ElectroThermalBinding, ElectroThermalSolution};
pub use relaxation::{ElectroThermalTracePoint, MultirateElectroThermalSimulator};
pub use runaway::{assess_diode_thermal_stability, ThermalStabilityAssessment};
