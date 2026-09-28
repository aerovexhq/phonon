//! 2D Moiré superlattices, Bistritzer-MacDonald continuum physics, and correlated electronic states.

pub mod bistritzer_macdonald;
pub mod correlated_insulator;
pub mod superconducting_dome;

pub use bistritzer_macdonald::{
    BistritzerMacDonaldModel, Complex2x2, MoireLattice, TmdMoireModel, Vector2D, GRAPHENE_A0_NM,
    GRAPHENE_VF_NM_S, HBAR_EV_S, HBAR_VF_EV_NM,
};
pub use correlated_insulator::CorrelatedInsulatorModel;
pub use superconducting_dome::SuperconductingDomeModel;
