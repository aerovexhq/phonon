//! Multi-physics solvers for non-Abelian Majorana zero mode braiding,
//! time-dependent Bogoliubov-de Gennes equations, and topological quantum logic.

pub mod hexagonal_majorana_benchmark;
pub mod majorana_braid_simulator;
pub mod tdbdg_solver;

pub use hexagonal_majorana_benchmark::{
    run_parallel_majorana_braid_benchmark, MajoranaBenchmarkReport,
};
pub use majorana_braid_simulator::{
    MajoranaBraidResult, MajoranaBraidSimulator, MajoranaNoiseEnvironment,
};
pub use tdbdg_solver::{Complex, TdBdGLatticeConfig, TdBdGSolver};
