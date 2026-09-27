//! Multi-threaded circuit partitioning, Data-Oriented Design (DoD), and Diakoptics.

pub mod diakoptics;
pub mod node_tearing;
pub mod soa_group;
pub mod torn_solver;

pub use diakoptics::{DiakopticsSolver, SubcircuitLocalSolution};
pub use node_tearing::PartitionedCircuit;
pub use soa_group::{CircuitSoA, DiodeSoA, MosfetSoA, ResistorSoA};
pub use torn_solver::solve_torn_dc;
