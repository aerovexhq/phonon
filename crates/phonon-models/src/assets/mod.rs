//! Unified Multi-Physics 3D Asset Ecosystem, Dielectric Material Library & Component Catalog
//!
//! Provides authoritative physical material models (EM, Optical, Acoustic, Thermal, Mechanical),
//! 100+ standard materials database, 3D triangular meshes with automated mass properties,
//! and procedural aerospace & RF component generators.

pub mod catalog;
pub mod library;
pub mod material;
pub mod mesh;

pub use catalog::{
    create_cubesat_chassis, create_dipole_antenna, create_finned_heatsink, create_patch_antenna,
    create_quadrotor_frame, create_tactile_landing_gear,
};
pub use library::MaterialLibrary;
pub use material::{
    AcousticProperties, ElectromagneticProperties, MaterialCategory, MaterialRecord,
    OpticalProperties, ThermalMechanicalProperties,
};
pub use mesh::{Aabb3D, Mesh3D, Submesh, Triangle3D, Vertex3D};
