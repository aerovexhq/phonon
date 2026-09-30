#![deny(unsafe_code)]

//! Multi-physics solver and parallel benchmark suite for autonomous acoustically driven
//! topological valley-Hall photonic waveguide & chiral quantum network router engine.

pub mod router_benchmark;
pub mod router_solver;

pub use router_benchmark::*;
pub use router_solver::*;
