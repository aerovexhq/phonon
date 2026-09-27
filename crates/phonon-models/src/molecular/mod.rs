//! Direct physical-chemistry molecular wire networks and quantum-interference logic.
//!
//! Provides:
//! - Tight-binding Hückel Hamiltonian for conjugated molecular junctions.
//! - Non-Equilibrium Green's Function (NEGF) transport and Landauer-Büttiker formalism.
//! - Quantum interference (QI) logic primitives (Inverter, NAND2, NOR2, XOR2, Full Adder).
//! - Sub-100 meV switching energy accounting and carbon nanoribbon interconnect modeling.

pub mod molecular_junction;
pub mod molecular_primitives;
pub mod negf_transport;

pub use molecular_junction::{MolecularGraphType, MolecularJunction};
pub use molecular_primitives::{
    MolecularFullAdderCell, MolecularGateMetrics, MolecularInverter, MolecularNand2, MolecularNor2,
    MolecularXor2, MultiBitMolecularAdder,
};
pub use negf_transport::{
    invert_complex_matrix, solve_complex_linear_system, NegfTransportSolver, CONDUCTANCE_QUANTUM_G0,
};
