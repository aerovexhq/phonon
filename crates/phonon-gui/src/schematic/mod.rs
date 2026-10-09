#![deny(unsafe_code)]

//! Schematic capture, infinite canvas, component library, and netlist compiler.

pub mod binary_format;
pub mod binary_history;
pub mod bom;
pub mod bus;
pub mod canvas;
pub mod categories;
pub mod circuit_compiler;
pub mod components;
pub mod erc;
pub mod history;
pub mod net_label;
pub mod netlist_sync;
pub mod sheet;
pub mod subcircuit;
pub mod subcircuit_package;
pub mod symbol;
pub mod wire;

pub use bom::{component_to_part_key_and_desc, generate_bom_from_canvas};

pub use binary_format::{
    component_category_to_discriminant, component_kind_from_discriminant,
    component_kind_to_discriminant, compute_adler32, deserialize_project, load_project_from_file,
    read_component, read_wire, save_project_to_file, save_project_to_file_with_pricing,
    serialize_project, serialize_project_with_pricing, write_component, write_wire,
    BinaryFormatError, DeserializedProject, CURRENT_VERSION, FLAG_HAS_PRICING, PHONON_MAGIC,
};
pub use binary_history::{
    adler32, deserialize_history, read_command, serialize_history, write_command, ActionOpcode,
    BinaryHistoryError, HISTORY_MAGIC, HISTORY_VERSION,
};
pub use bus::{BusSignal, BusTapOff, SchematicBus};
pub use canvas::SchematicCanvas;
pub use categories::ComponentCategory;
pub use circuit_compiler::{
    compile_schematic, compile_schematic_with_labels, compute_wire_telemetry, CompiledCircuit,
};
pub use components::{ComponentKind, SchematicComponent};
pub use erc::{ErcCode, ErcDiagnostic, ErcEngine, ErcSeverity};
pub use history::{CanvasCommand, HistoryStack};
pub use net_label::{NetLabel, NetLabelOrientation};
pub use netlist_sync::{NetlistSyncEngine, NetlistSyncError, SyncDelta};
pub use sheet::{MultiSheetManager, SchematicSheet};
pub use subcircuit::{
    flatten_hierarchical_netlist, flatten_hierarchical_netlist_with_instances, PinDirection,
    SubcircuitDefinition, SubcircuitInstance, SubcircuitPin,
};
pub use subcircuit_package::{
    discover_boundary_ports, PortDirection, PortEdge, SubcircuitPackage, SubcircuitPort,
    SubcircuitRegistry,
};
pub use symbol::{
    CustomComponentSymbol, LabelPlacement, SymbolLibrary, SymbolPin, SymbolPinDirection,
    SymbolPrimitive, TerminalDirection,
};
pub use wire::{
    compute_junction_dots, compute_wire_crossings, render_wire_crossings, PinNormal, SchematicWire,
    WireCrossing, WirePinOrientation, WireSegment,
};

