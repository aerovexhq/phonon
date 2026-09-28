//! Spintronic, nanomagnetic, and magnetoresistive device models.
//!
//! Modules:
//! - `nanomagnet`: Single-domain ferromagnets, demagnetization tensors, and dipole fields.
//! - `llgs_solver`: Landau-Lifshitz-Gilbert-Slonczewski equation with STT, SOT, and Langevin thermal fields.
//! - `nml_primitives`: Nanomagnetic Logic (NML) digital gates, Majority-3 logic, and non-volatile adders.
//! - `ciss`: Chiral-Induced Spin Selectivity (CISS) and helical tight-binding quantum transport.
//! - `smm`: Single-Molecule Magnets (SMM), giant anisotropy, and quantum tunneling of magnetization (QTM).
//! - `molecular_spintronics`: Coupled CISS-SMM logic and non-volatile molecular memory cells.

pub mod ciss;
pub mod llgs_solver;
pub mod molecular_spintronics;
pub mod nanomagnet;
pub mod nml_primitives;
pub mod smm;

pub use ciss::{ChiralHelixGeometry, Chirality, CissHamiltonian, CissTransmission, ComplexMatrix};
pub use llgs_solver::{LlgsConfig, LlgsSolver};
pub use molecular_spintronics::{
    MolecularInverter, MolecularMajority3, MolecularSpinState, MolecularSpintronicCell,
};
pub use nanomagnet::{MagneticMaterial, Nanomagnet, Vec3};
pub use nml_primitives::{
    MultiBitNmlAdder, NmlAnd2, NmlFullAdderCell, NmlGateMetrics, NmlInverter, NmlMajority3, NmlOr2,
};
pub use smm::{SingleMoleculeMagnet, SpinValue, BOHR_MAGNETON_EV, BOHR_MAGNETON_JOULES};
