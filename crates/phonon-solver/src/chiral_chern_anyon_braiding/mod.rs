#![deny(unsafe_code)]

//! Non-Abelian anyon braiding in chiral acoustic Chern metamaterials & fault-tolerant phononic topological qubits.

mod braiding_benchmark;
mod braiding_solver;

pub use braiding_benchmark::{BraidingBenchmarkResult, BraidingBenchmarkRunner};
pub use braiding_solver::ChiralChernAnyonBraidingSolver;
