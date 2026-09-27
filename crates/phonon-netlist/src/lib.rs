//! SPICE netlist lexer, parser, hierarchical subcircuit flattener, and model elaborator.

#![deny(unsafe_code)]

pub mod ast;
pub mod elaborator;
pub mod error;
pub mod geometry;
pub mod lexer;
pub mod parser;
pub mod pdk;

pub use ast::*;
pub use elaborator::{elaborate_netlist, ElaboratedCircuit, SimulationPlan};
pub use error::NetlistError;
pub use geometry::{LefLibrary, LefMacro, LefPin, LefRect, PinDirection, TransistorLayout};
pub use lexer::{parse_spice_number, preprocess_netlist};
pub use parser::parse_netlist;
pub use pdk::{Gf180McuPdk, Sky130Pdk};
