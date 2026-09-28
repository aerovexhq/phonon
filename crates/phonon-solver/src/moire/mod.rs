//! 2D Moiré superlattices, continuum bandstructure eigensolvers, and correlated insulator benchmarks.

pub mod hartree_fock_solver;
pub mod moire_flatband_benchmark;
pub mod moire_hamiltonian_solver;

pub use hartree_fock_solver::{HartreeFockSolution, HartreeFockSolver};
pub use moire_flatband_benchmark::{run_10k_moire_benchmark, MoireBenchmarkReport};
pub use moire_hamiltonian_solver::{MoireBandSolution, MoireHamiltonianSolver};
