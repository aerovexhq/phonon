//! Relativistic laser-plasma wakefield acceleration solvers:
//! Boris particle pusher, bubble-regime bunch tracking, and parallel Rayon benchmarks.

pub mod pwfa_benchmark;
pub mod relativistic_boris_pusher;
pub mod wakefield_accelerator;

pub use pwfa_benchmark::{PwfaBenchmarkReport, PwfaBenchmarkRunner};
pub use relativistic_boris_pusher::{
    BorisPusher, RelativisticParticle, ELECTRON_MASS_KG, ELEMENTARY_CHARGE, SPEED_OF_LIGHT,
    VACUUM_PERMITTIVITY,
};
pub use wakefield_accelerator::{BeamBunch, TrackingSummary, WakefieldAccelerator};
