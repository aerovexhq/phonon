#![deny(unsafe_code)]

//! Schematic capture, infinite canvas, component library, and netlist compiler.

pub mod canvas;
pub mod categories;
pub mod circuit_compiler;
pub mod components;
pub mod history;
pub mod wire;

pub use canvas::SchematicCanvas;
pub use categories::ComponentCategory;
pub use circuit_compiler::{compile_schematic, CompiledCircuit};
pub use components::{ComponentKind, SchematicComponent};
pub use history::{CanvasCommand, HistoryStack};
pub use wire::{compute_junction_dots, SchematicWire, WireSegment};
