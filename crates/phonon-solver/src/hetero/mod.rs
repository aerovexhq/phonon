//! Heterogeneous CPU macro-architecture synthesis, timing closure, and optimization solver.
//!
//! Provides:
//! - Multi-material static timing closure and F_max frequency determination (`timing_closure`).
//! - Rayon parallel combinatorial optimization across all CPU cores (`hetero_optimizer`).
//! - Comprehensive benchmark runner comparing uniform Silicon vs heterogeneous CPU architectures (`cpu_benchmark`).

pub mod cpu_benchmark;
pub mod hetero_optimizer;
pub mod timing_closure;

pub use cpu_benchmark::{HeteroCpuBenchmarkResult, HeteroCpuBenchmarkRunner};
pub use hetero_optimizer::{HeteroCpuOptimizer, HeteroOptimizationCandidate};
pub use timing_closure::{PipelineTimingReport, TimingPathAnalyzer};
