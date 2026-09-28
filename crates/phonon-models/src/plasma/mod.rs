//! Autonomous high-energy plasma dynamics, tokamak fusion magnetics, and Alfvén wave models.

pub mod alfven_waves;
pub mod constants;
pub mod grad_shafranov;
pub mod kinetic_particles;
pub mod mhd_multi_fluid;

pub use alfven_waves::{AlfvenWaveProperties, ToroidalAlfvenEigenmode};
pub use constants::{
    ALPHA_MASS, BOLTZMANN_CONSTANT, DEUTERON_MASS, DT_ALPHA_ENERGY_JOULES, DT_TOTAL_ENERGY_JOULES,
    ELECTRON_MASS, ELEMENTARY_CHARGE, EV_TO_JOULES, KEV_TO_JOULES, PLASMA_ADIABATIC_INDEX,
    PROTON_MASS, SPEED_OF_LIGHT, TRITON_MASS, VACUUM_PERMEABILITY, VACUUM_PERMITTIVITY,
};
pub use grad_shafranov::{SafetyFactorProfile, SolovevEquilibrium, TokamakBeta, TokamakGeometry};
pub use kinetic_particles::{KineticParticle, PlasmaSpecies};
pub use mhd_multi_fluid::{IcrfHeatingSource, MhdFluidState, ThermonuclearFusion};
