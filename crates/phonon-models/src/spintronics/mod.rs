//! Spintronic, nanomagnetic, and magnetoresistive device models.
//!
//! Modules:
//! - `nanomagnet`: Single-domain ferromagnets, demagnetization tensors, and dipole fields.
//! - `llgs_solver`: Landau-Lifshitz-Gilbert-Slonczewski equation with STT, SOT, and Langevin thermal fields.
//! - `nml_primitives`: Nanomagnetic Logic (NML) digital gates, Majority-3 logic, and non-volatile adders.

pub mod llgs_solver;
pub mod nanomagnet;
pub mod nml_primitives;

pub use llgs_solver::{LlgsConfig, LlgsSolver};
pub use nanomagnet::{MagneticMaterial, Nanomagnet, Vec3};
pub use nml_primitives::{
    MultiBitNmlAdder, NmlAnd2, NmlFullAdderCell, NmlGateMetrics, NmlInverter, NmlMajority3, NmlOr2,
};
