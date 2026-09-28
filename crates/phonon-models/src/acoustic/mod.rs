//! Acoustic Wave Propagation, Atmospheric Sound Transduction & Physical Microphone Synthesis
//!
//! Provides multi-medium wave equations, atmospheric attenuation, structural wall
//! transmission loss, physical condenser/piezoelectric microphone transducers,
//! and kinematic Doppler shift modeling.

pub mod medium;
pub mod microphone;
pub mod wave;

pub use medium::{
    speed_of_sound_in_air, AcousticMedium, AcousticWall, MediumType, ADIABATIC_INDEX_AIR,
    GAS_CONSTANT_R, MOLAR_MASS_AIR, P_ATM_SEA_LEVEL, P_REF_AIR, T_REF_KELVIN,
};
pub use microphone::{
    CondenserMicrophone, MicrophonePolarPattern, MicrophoneSignal, PiezoelectricMicrophone,
};
pub use wave::{
    compute_acoustic_doppler, evaluate_acoustic_field, AcousticDopplerResult, AcousticFieldPoint,
    AcousticObserver, AcousticSource,
};
