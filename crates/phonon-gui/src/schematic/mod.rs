//! Schematic capture, infinite canvas, component library, and netlist compiler.

pub mod canvas;
pub mod circuit_compiler;
pub mod components;
pub mod wire;

pub use canvas::SchematicCanvas;
pub use circuit_compiler::{compile_schematic, CompiledCircuit};
pub use components::{ComponentKind, SchematicComponent};
pub use wire::{compute_junction_dots, SchematicWire, WireSegment};
