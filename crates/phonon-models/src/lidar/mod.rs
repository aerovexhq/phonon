//! LiDAR Time-of-Flight Models, Atmospheric Scattering & Scanning Architectures
//!
//! Provides physically rigorous laser beam propagation, Gaussian divergence,
//! surface BRDF reflectance, multi-echo pulse return, Mie atmospheric extinction,
//! backscatter clutter, and 360-degree/MEMS/Flash scanning mechanisms.

pub mod atmosphere;
pub mod beam;
pub mod scanner;

pub use atmosphere::{AtmosphericCondition, FogType};
pub use beam::{EchoReturn, LaserPulseConfig, LaserRay, WAVELENGTH_1550_NM, WAVELENGTH_905_NM};
pub use scanner::{LidarScannerConfig, ScanningArchitecture};
