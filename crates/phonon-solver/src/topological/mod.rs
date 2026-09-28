//! Topological quantum computing solver engines, non-Abelian braiding, and benchmark runners.

pub mod braiding_solver;
pub mod parity_tracker;
pub mod surface_decoder;
pub mod threshold_simulator;
pub mod topological_benchmark;

pub use braiding_solver::{BraidingStepResult, MajoranaBraidingSolver};
pub use parity_tracker::{FermionParitySolver, ParityTrackingReport};
pub use surface_decoder::{BeliefPropagationDecoder, MwpmDecoder, SyndromeDefect};
pub use threshold_simulator::{ThresholdDataPoint, ThresholdSimulationReport, ThresholdSimulator};
pub use topological_benchmark::{
    BenchmarkComparisonReport, ComprehensiveQecBenchmarkRunner, ComprehensiveQecReport,
    TopologicalBenchmarkRunner,
};
