#![deny(unsafe_code)]

//! Schematic capture, infinite canvas, component library, and netlist compiler.

pub mod binary_format;
pub mod binary_history;
pub mod bus;
pub mod canvas;
pub mod categories;
pub mod circuit_compiler;
pub mod components;
pub mod erc;
pub mod history;
pub mod netlist_sync;
pub mod sheet;
pub mod subcircuit;
pub mod subcircuit_package;
pub mod symbol;
pub mod wire;

pub use binary_format::{
    component_category_to_discriminant, component_kind_from_discriminant,
    component_kind_to_discriminant, compute_adler32, deserialize_project, load_project_from_file,
    read_component, read_wire, save_project_to_file, serialize_project, write_component,
    write_wire, BinaryFormatError, DeserializedProject, CURRENT_VERSION, PHONON_MAGIC,
};
pub use binary_history::{
    adler32, deserialize_history, read_command, serialize_history, write_command, ActionOpcode,
    BinaryHistoryError, HISTORY_MAGIC, HISTORY_VERSION,
};
pub use bus::{BusSignal, BusTapOff, SchematicBus};
pub use canvas::SchematicCanvas;
pub use categories::ComponentCategory;
pub use circuit_compiler::{compile_schematic, compute_wire_telemetry, CompiledCircuit};
pub use components::{ComponentKind, SchematicComponent};
pub use erc::{ErcCode, ErcDiagnostic, ErcEngine, ErcSeverity};
pub use history::{CanvasCommand, HistoryStack};
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
    compute_junction_dots, PinNormal, SchematicWire, WirePinOrientation, WireSegment,
};

