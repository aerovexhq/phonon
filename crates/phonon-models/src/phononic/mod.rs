//! Phononic crystal metamaterials, acoustic wave logic, and hypersonic nanoresonator physics.

pub mod crystal;
pub mod logic;
pub mod piezoelectric;
pub mod resonator;

pub use crystal::{AcousticLayer, DefectPhononicWaveguide, PhononicCrystal1D};
pub use logic::{
    AcousticAnd, AcousticInverter, AcousticOr, AcousticWave, AcousticXor, PhononicFullAdder,
};
pub use piezoelectric::{
    ElectricDisplacement, ElectricField, PiezoelectricMaterial, VoigtStrain, VoigtStress, EPSILON_0,
};
pub use resonator::{BawResonator, MbvdParameters, SawResonator};
