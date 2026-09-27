//! First-principles semiconductor material chemistry, crystallography, bandstructure, and interfaces.

pub mod bandstructure;
pub mod builder;
pub mod contact;
pub mod crystal;
pub mod dielectric;
pub mod dopant;
pub mod heterostructure;
pub mod mobility;
pub mod presets;

pub use bandstructure::Bandstructure;
pub use builder::ChemicalMaterialBuilder;
pub use contact::{ContactMaterial, ContactSpecies};
pub use crystal::{CrystalStructure, Crystallography};
pub use dielectric::{DielectricMaterial, DielectricSpecies};
pub use dopant::{DopantSpecies, DopantType};
pub use heterostructure::{BandAlignmentType, HeteroInterface};
pub use mobility::{CarrierMobilityParams, ChemicalMobility};
pub use presets::{ChemicalMaterial, Silicon};
