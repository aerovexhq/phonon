//! Multi-physics solver and parallel benchmark suite for topological
//! non-Abelian Majorana braiding in phononic Josephson metamaterials.

pub mod majorana_braiding_benchmark;
pub mod majorana_braiding_solver;

pub use majorana_braiding_benchmark::{
    MajoranaBraidingBenchmarkResult, MajoranaBraidingBenchmarkRunner,
};
pub use majorana_braiding_solver::TopologicalMajoranaBraidingSolver;
