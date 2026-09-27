//! Multi-Valued Logic (MVL) solver and benchmarking engine.
//!
//! Submodules:
//! - `ternary_solver`: Damped Newton-Raphson 3-state DC transfer curve solver and balanced ternary adder engine.
//! - `mvl_benchmark`: Parallel multi-threaded comparative benchmark comparing ternary datapaths against 64-bit binary baselines.

pub mod mvl_benchmark;
pub mod ternary_solver;

pub use mvl_benchmark::{MvlBenchmarkReport, MvlBenchmarkRunner};
pub use ternary_solver::{TernaryAdderEngine, TernaryCircuitSolver, TernarySolverError};
