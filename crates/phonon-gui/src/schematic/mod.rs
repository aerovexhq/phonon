#![deny(unsafe_code)]

//! Schematic capture, infinite canvas, component library, and netlist compiler.

pub mod binary_format;
pub mod canvas;
pub mod categories;
pub mod circuit_compiler;
pub mod components;
pub mod erc;
pub mod history;
pub mod netlist_sync;
pub mod wire;

pub use binary_format::{
    component_category_to_discriminant, component_kind_from_discriminant,
    component_kind_to_discriminant, compute_adler32, deserialize_project, load_project_from_file,
    save_project_to_file, serialize_project, BinaryFormatError, DeserializedProject,
    CURRENT_VERSION, PHONON_MAGIC,
};
pub use canvas::SchematicCanvas;
pub use categories::ComponentCategory;
pub use circuit_compiler::{compile_schematic, CompiledCircuit};
pub use components::{ComponentKind, SchematicComponent};
pub use erc::{ErcCode, ErcDiagnostic, ErcEngine, ErcSeverity};
pub use history::{CanvasCommand, HistoryStack};
pub use netlist_sync::{NetlistSyncEngine, NetlistSyncError, SyncDelta};
pub use wire::{compute_junction_dots, SchematicWire, WireSegment};
