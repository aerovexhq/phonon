//! Multi-physics solver and parallel benchmark suite for cavity acoustomagnonic
//! dark matter haloscopes, axion-magnon hybridization, and sub-Kelvin quantum readout.

pub mod acoustomagnonic_benchmark;
pub mod acoustomagnonic_solver;

pub use acoustomagnonic_benchmark::{
    AcoustomagnonicBenchmarkResult, AcoustomagnonicBenchmarkRunner,
};
pub use acoustomagnonic_solver::CavityAcoustomagnonicSolver;
