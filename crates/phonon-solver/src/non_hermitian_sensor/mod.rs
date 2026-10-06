#![deny(unsafe_code)]

//! Non-Hermitian Floquet Skin-Effect Sensor & Exceptional Point Magnetometer Module.
//!
//! Provides:
//! - Non-Hermitian skin effect (NHSE) eigensolver with Generalized Brillouin Zone (GBZ) analysis,
//!   point-gap spectral winding number calculation, and exponential boundary localization.
//! - Non-Hermitian Exceptional Point (EP2 & EP3) sensor engine with power-law frequency splitting
//!   and divergent responsivity enhancement (> 100x over linear sensors).
//! - Sub-picotesla acoustic-magnonic magnetometer demonstrating B_min < 1.0 pT / sqrt(Hz)
//!   and dynamic range >= 60 dB.

pub mod exceptional_point;
pub mod magnetometer;
pub mod skin_effect;

pub use exceptional_point::{
    EpSensor, EpSensorParams, ExceptionalPointOrder,
};
pub use magnetometer::{
    AcousticMagnonicMagnetometer, MagnetoacousticParams, MagnetometerTelemetry,
};
pub use skin_effect::{
    Complex, NonHermitianLatticeParams, SkinEffectSolver,
};
