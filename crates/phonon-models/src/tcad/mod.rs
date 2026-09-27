//! Microscopic Technology Computer-Aided Design (TCAD) semiconductor physics and synthesis engine.

pub mod material;
pub mod mesh;
pub mod parameter_extraction;
pub mod poisson_dd;
pub mod tcad_companion;
pub mod tcad_device;

pub use material::{MaterialProperties, SemiconductorMaterial};
pub use mesh::{Contact, ContactType, Mesh1D};
pub use parameter_extraction::{extract_diode_model, extract_mosfet_model};
pub use poisson_dd::{bernoulli, solve_tridiagonal, PoissonDriftDiffusionSolver, TcadState1D};
pub use tcad_companion::{TcadDiodeCompanion, TcadMosfetCompanion};
pub use tcad_device::{TcadDevice, TcadDeviceBuilder};
