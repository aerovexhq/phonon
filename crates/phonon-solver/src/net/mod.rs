//! End-to-End CPU-to-Router Network Co-Simulation & Discrete Packet Switching Solvers
//!
//! Synchronizes CPU execution pipelines, MMIO NIC controllers, Router switches,
//! and multi-tier physical RF channel propagation models.

pub mod network_benchmark;
pub mod network_cosim_solver;

pub use network_benchmark::{NetworkBenchmarkReport, NetworkBenchmarkRunner};
pub use network_cosim_solver::{CoSimStepReport, NetworkCoSimulator};
