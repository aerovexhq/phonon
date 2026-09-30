#![deny(unsafe_code)]

//! Multi-physics solver and benchmark suite for autonomous photonic-phononic-spintronic
//! tripartite quantum router engine.

pub mod router_benchmark;
pub mod router_solver;

pub use router_benchmark::*;
pub use router_solver::*;
