#![deny(unsafe_code)]

//! Test suite for reusable component packaging (.phnc binary format, port discovery, and SPICE export).

use egui::Pos2;
use phonon_gui::schematic::{
    discover_boundary_ports, ComponentKind, PortDirection, PortEdge, SchematicComponent,
    SchematicWire, SubcircuitPackage, SubcircuitPort, SubcircuitRegistry, WireSegment,
};

#[test]
fn test_subcircuit_port_properties() {
    let port_left = SubcircuitPort::new("IN", PortDirection::Input, PortEdge::Left, "net_in");
    assert_eq!(port_left.name, "IN");
    assert_eq!(port_left.direction, PortDirection::Input);
    assert_eq!(port_left.edge, PortEdge::Left);
    assert_eq!(port_left.internal_net, "net_in");

    let port_right = SubcircuitPort::new("OUT", PortDirection::Output, PortEdge::Right, "net_out");
    assert_eq!(port_right.name, "OUT");
    assert_eq!(port_right.direction, PortDirection::Output);
    assert_eq!(port_right.edge, PortEdge::Right);
    assert_eq!(port_right.internal_net, "net_out");

    assert_eq!(PortEdge::Left.display_name(), "Left");
    assert_eq!(PortEdge::Right.display_name(), "Right");
    assert_eq!(PortEdge::Top.display_name(), "Top");
    assert_eq!(PortEdge::Bottom.display_name(), "Bottom");
}

#[test]
fn test_discover_boundary_ports_with_labeled_wires() {
    let mut comps = Vec::new();
    let mut r1 = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(100.0, 100.0), 0);
    r1.name = "R1".to_string();
    comps.push(r1);

    let mut r2 = SchematicComponent::new(2, ComponentKind::Resistor, Pos2::new(100.0, 200.0), 1);
    r2.name = "R2".to_string();
    comps.push(r2);

    let wires = vec![
        SchematicWire::new(1, vec![WireSegment::new(Pos2::new(50.0, 100.0), Pos2::new(100.0, 100.0))])
            .with_net_name("VIN"),
        SchematicWire::new(2, vec![WireSegment::new(Pos2::new(100.0, 140.0), Pos2::new(100.0, 200.0))])
            .with_net_name("VMID"),
        SchematicWire::new(3, vec![WireSegment::new(Pos2::new(100.0, 240.0), Pos2::new(150.0, 240.0))])
            .with_net_name("VOUT"),
    ];

    let discovered_ports = discover_boundary_ports(&comps, &wires);
    assert!(!discovered_ports.is_empty());
    let port_names: Vec<String> = discovered_ports.iter().map(|p| p.name.clone()).collect();
    assert!(port_names.contains(&"VIN".to_string()));
    assert!(port_names.contains(&"VOUT".to_string()));
}

#[test]
fn test_spice_subckt_generation() {
    let mut comps = Vec::new();
    let mut r1 = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(0.0, 0.0), 0);
    r1.name = "R1".to_string();
    comps.push(r1);

    let ports = vec![
        SubcircuitPort::new("IN", PortDirection::Input, PortEdge::Left, "net_in"),
        SubcircuitPort::new("OUT", PortDirection::Output, PortEdge::Right, "net_out"),
    ];

    let pkg = SubcircuitPackage::new("ATTENUATOR", "4.7k Attenuator block", ports, comps, Vec::new());
    let subckt = pkg.to_spice_subckt();

    assert!(subckt.contains(".SUBCKT ATTENUATOR"));
    assert!(subckt.contains(".ENDS ATTENUATOR"));
    assert!(subckt.contains("4.7k Attenuator block"));
}

#[test]
fn test_binary_phnc_serialization_and_deserialization() {
    let mut comps = Vec::new();
    let mut d1 = SchematicComponent::new(1, ComponentKind::Diode, Pos2::new(10.0, 20.0), 1);
    d1.name = "D1".to_string();
    comps.push(d1);

    let wires = vec![SchematicWire::new(
        1,
        vec![WireSegment::new(Pos2::new(0.0, 0.0), Pos2::new(10.0, 20.0))],
    )];

    let ports = vec![
        SubcircuitPort::new("ANODE", PortDirection::Input, PortEdge::Left, "net_anode"),
        SubcircuitPort::new("CATHODE", PortDirection::Output, PortEdge::Right, "net_cathode"),
    ];

    let mut pkg = SubcircuitPackage::new("CLIPPING_DIODE", "Signal clipping diode subcircuit", ports, comps, wires);
    pkg.author = "Phonon Team".to_string();

    let bytes = pkg.serialize();

    // Check header
    assert_eq!(&bytes[0..4], b"PHNC");
    assert_eq!(bytes[4], 1); // Version 1

    // Deserialize
    let restored = SubcircuitPackage::deserialize(&bytes).expect("Deserialization from .phnc should succeed");
    assert_eq!(restored.name, "CLIPPING_DIODE");
    assert_eq!(restored.description, "Signal clipping diode subcircuit");
    assert_eq!(restored.author, "Phonon Team");
    assert_eq!(restored.ports.len(), 2);
    assert_eq!(restored.components.len(), 1);
    assert_eq!(restored.wires.len(), 1);
    assert_eq!(restored.components[0].name, "D1");
}

#[test]
fn test_phnc_corruption_detection() {
    let pkg = SubcircuitPackage::new("TEST_PKG", "Description", Vec::new(), Vec::new(), Vec::new());
    let mut bytes = pkg.serialize();

    // Test corrupted magic header
    bytes[0] = b'X';
    assert!(SubcircuitPackage::deserialize(&bytes).is_err());

    // Restore magic and corrupt checksum
    bytes[0] = b'P';
    let len = bytes.len();
    bytes[len - 1] ^= 0xFF;
    assert!(SubcircuitPackage::deserialize(&bytes).is_err());
}

#[test]
fn test_subcircuit_registry_operations() {
    let mut registry = SubcircuitRegistry::new();
    // Initially empty by default - no phantom subcircuits
    assert!(registry.list().is_empty());
    assert!(registry.get("VoltageDivider").is_none());

    // Explicit registration of demo defaults
    registry.register_built_in_defaults();
    assert!(!registry.list().is_empty());
    assert!(registry.get("VoltageDivider").is_some());

    let pkg = SubcircuitPackage::new("RC_FILTER", "Lowpass Filter", Vec::new(), Vec::new(), Vec::new());
    registry.register(pkg);

    assert!(registry.get("RC_FILTER").is_some());
    assert_eq!(registry.get("RC_FILTER").unwrap().description, "Lowpass Filter");

    let removed = registry.remove("RC_FILTER");
    assert!(removed.is_some());
    assert!(registry.get("RC_FILTER").is_none());
}
