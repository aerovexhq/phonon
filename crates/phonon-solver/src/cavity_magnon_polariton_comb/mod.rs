//! Multi-physics solver and parallel benchmark suite for cavity quantum
//! magnon-polariton frequency combs, Kerr non-linearities, and squeezed halometry.

pub mod magnon_polariton_comb_benchmark;
pub mod magnon_polariton_comb_solver;

pub use magnon_polariton_comb_benchmark::{
    MagnonPolaritonCombBenchmarkResult, MagnonPolaritonCombBenchmarkRunner,
};
pub use magnon_polariton_comb_solver::CavityMagnonPolaritonCombSolver;
