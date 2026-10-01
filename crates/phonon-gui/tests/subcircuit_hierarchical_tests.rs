#![deny(unsafe_code)]

//! Verification test suite for hierarchical subcircuit macro-modeling and multi-sheet canvas management.

use egui::Pos2;
use phonon_gui::schematic::{
    flatten_hierarchical_netlist, ComponentKind, MultiSheetManager, PinDirection,
    SchematicCanvas, SchematicComponent, SchematicWire, SubcircuitDefinition,
    SubcircuitInstance, SubcircuitPin, WireSegment,
};
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn test_subcircuit_creation_pin_registration_and_instance_placement() {
    let mut internal_canvas = SchematicCanvas::new();
    let r1 = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(0.0, 0.0), 1)
        .with_property("net:1", "in")
        .with_property("net:2", "mid");
    let c1 = SchematicComponent::new(2, ComponentKind::Capacitor, Pos2::new(40.0, 0.0), 1)
        .with_property("net:1", "mid")
        .with_property("net:2", "0");
    internal_canvas.add_component(r1);
    internal_canvas.add_component(c1);

    let pins = vec![
        SubcircuitPin::new("IN", PinDirection::Input, "in", Pos2::new(-40.0, 0.0)),
        SubcircuitPin::new("OUT", PinDirection::Output, "mid", Pos2::new(40.0, 0.0)),
        SubcircuitPin::new("GND", PinDirection::Inout, "0", Pos2::new(0.0, 30.0)),
    ];

    let def = SubcircuitDefinition::new("RC_LOWPASS", pins, internal_canvas);

    assert_eq!(def.name, "RC_LOWPASS");
    assert_eq!(def.pins.len(), 3);
    assert!(def.pin_by_name("IN").is_some());
    assert_eq!(def.pin_by_name("IN").unwrap().direction, PinDirection::Input);
    assert_eq!(def.pin_by_name("OUT").unwrap().local_net, "mid");
    assert!(def.pin_by_name("NONEXISTENT").is_none());

    // Create placed instance
    let mut inst = SubcircuitInstance::new(1, "RC_LOWPASS", Pos2::new(100.0, 200.0));
    inst.map_pin("IN", "sig_in");
    inst.map_pin("OUT", "sig_out");
    inst.map_pin("GND", "0");

    assert_eq!(inst.id, 1);
    assert_eq!(inst.def_name, "RC_LOWPASS");
    assert_eq!(inst.pin_nets.get("IN").unwrap(), "sig_in");
    assert_eq!(inst.pin_nets.get("OUT").unwrap(), "sig_out");

    // Pin coordinate transformations
    let pin_in = def.pin_by_name("IN").unwrap();
    let pos_r0 = inst.pin_world_pos(pin_in);
    assert_eq!(pos_r0, Pos2::new(60.0, 200.0));

    inst.rotation = 1; // 90 deg clockwise: (-40, 0) -> (0, -40)
    let pos_r1 = inst.pin_world_pos(pin_in);
    assert_eq!(pos_r1, Pos2::new(100.0, 160.0));

    // Bounds and hit-testing
    assert!(inst.contains(Pos2::new(100.0, 200.0), Some(&def)));
    assert!(!inst.contains(Pos2::new(500.0, 500.0), Some(&def)));
}

#[test]
fn test_hierarchical_flattening_into_unified_flat_spice_netlist_with_prefixing() {
    let mut filter_canvas = SchematicCanvas::new();
    let r1 = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(0.0, 0.0), 1)
        .with_property("net:1", "in_node")
        .with_property("net:2", "internal_node");
    let c1 = SchematicComponent::new(2, ComponentKind::Capacitor, Pos2::new(40.0, 0.0), 1)
        .with_property("net:1", "internal_node")
        .with_property("net:2", "0");
    let r2 = SchematicComponent::new(3, ComponentKind::Resistor, Pos2::new(80.0, 0.0), 2)
        .with_property("net:1", "internal_node")
        .with_property("net:2", "out_node");

    filter_canvas.add_component(r1);
    filter_canvas.add_component(c1);
    filter_canvas.add_component(r2);

    let pins = vec![
        SubcircuitPin::new("IN", PinDirection::Input, "in_node", Pos2::new(-40.0, 0.0)),
        SubcircuitPin::new("OUT", PinDirection::Output, "out_node", Pos2::new(40.0, 0.0)),
    ];

    let def = SubcircuitDefinition::new("FILTER_STAGE", pins, filter_canvas);
    let mut defs = HashMap::new();
    defs.insert("FILTER_STAGE".to_string(), def);

    // Build top-level canvas with two cascaded subcircuit instances and a voltage source
    let mut top_canvas = SchematicCanvas::new();
    let v_src = SchematicComponent::new(10, ComponentKind::VoltageSource, Pos2::new(-100.0, 0.0), 1)
        .with_property("net:+", "vin")
        .with_property("net:-", "0");
    top_canvas.add_component(v_src);

    let mut inst1 = SubcircuitInstance::new(1, "FILTER_STAGE", Pos2::new(0.0, 0.0));
    inst1.map_pin("IN", "vin");
    inst1.map_pin("OUT", "vmid");
    top_canvas.add_subcircuit_instance(inst1);

    let mut inst2 = SubcircuitInstance::new(2, "FILTER_STAGE", Pos2::new(120.0, 0.0));
    inst2.map_pin("IN", "vmid");
    inst2.map_pin("OUT", "vout");
    top_canvas.add_subcircuit_instance(inst2);

    let netlist = flatten_hierarchical_netlist(&top_canvas, &defs).expect("Flattening failed");

    // Verification checks
    assert!(netlist.contains("* Exported from Phonon CAD Schematic"));
    assert!(netlist.contains(".TEMP 27.0"));
    assert!(netlist.contains(".OP"));
    assert!(netlist.contains(".END"));

    // Check component inlining and naming
    assert!(netlist.contains("R1_X1"));
    assert!(netlist.contains("C1_X1"));
    assert!(netlist.contains("R2_X1"));
    assert!(netlist.contains("R1_X2"));
    assert!(netlist.contains("C1_X2"));
    assert!(netlist.contains("R2_X2"));

    // Check internal node prefixing: internal_node prefixed with X<id>_
    assert!(netlist.contains("X1_internal_node"));
    assert!(netlist.contains("X2_internal_node"));

    // Check boundary pin nets mapped to top-level signals
    assert!(netlist.contains("vin"));
    assert!(netlist.contains("vmid"));
    assert!(netlist.contains("vout"));

    // Check ground remains 0
    assert!(netlist.contains(" 0 "));
}

#[test]
fn test_multisheet_tab_creation_switching_and_last_sheet_deletion_protection() {
    let mut manager = MultiSheetManager::new("Main Sheet");

    assert_eq!(manager.sheets.len(), 1);
    assert_eq!(manager.active_sheet_idx, 0);
    assert_eq!(manager.active_sheet().name, "Main Sheet");

    let idx2 = manager.add_sheet("Power Supply");
    let idx3 = manager.add_sheet("Microcontroller");

    assert_eq!(manager.sheets.len(), 3);
    assert_eq!(idx2, 1);
    assert_eq!(idx3, 2);

    // Switch sheets
    assert!(manager.switch_sheet(1));
    assert_eq!(manager.active_sheet().name, "Power Supply");

    assert!(manager.switch_sheet(2));
    assert_eq!(manager.active_sheet().name, "Microcontroller");

    assert!(!manager.switch_sheet(99)); // Out of bounds
    assert_eq!(manager.active_sheet().name, "Microcontroller");

    // Remove sheet
    assert!(manager.remove_sheet(1).is_ok());
    assert_eq!(manager.sheets.len(), 2);

    // Last sheet deletion protection
    assert!(manager.remove_sheet(1).is_ok());
    assert_eq!(manager.sheets.len(), 1);

    let err = manager.remove_sheet(0);
    assert!(err.is_err());
    assert_eq!(err.unwrap_err(), "Cannot remove the last remaining sheet");
    assert_eq!(manager.sheets.len(), 1);
}

#[test]
fn test_cross_sheet_global_net_resolution() {
    let mut manager = MultiSheetManager::new("Power");
    let _ = manager.add_sheet("Logic");
    let _ = manager.add_sheet("Sensors");

    // Sheet 0 (Power): Wires for VCC, GND, VDD_5V
    manager.sheets[0].canvas.add_wire(
        SchematicWire::new(
            1,
            vec![WireSegment::new(Pos2::new(0.0, 0.0), Pos2::new(100.0, 0.0))],
        )
        .with_net_name("VCC"),
    );
    manager.sheets[0].canvas.add_wire(
        SchematicWire::new(
            2,
            vec![WireSegment::new(Pos2::new(0.0, 20.0), Pos2::new(100.0, 20.0))],
        )
        .with_net_name("VDD_5V"),
    );

    // Sheet 1 (Logic): Component with property net: VCC, local wire LOCAL_CLK
    let comp = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(50.0, 50.0), 1)
        .with_property("net", "VCC");
    manager.sheets[1].canvas.add_component(comp);
    manager.sheets[1].canvas.add_wire(
        SchematicWire::new(
            3,
            vec![WireSegment::new(Pos2::new(0.0, 0.0), Pos2::new(50.0, 0.0))],
        )
        .with_net_name("LOCAL_CLK"),
    );

    // Sheet 2 (Sensors): Subcircuit instance with pin mapped to VCC and wire for VDD_5V
    let mut inst = SubcircuitInstance::new(1, "SENSOR_BLOCK", Pos2::new(0.0, 0.0));
    inst.map_pin("PWR", "VCC");
    manager.sheets[2].canvas.add_subcircuit_instance(inst);

    manager.sheets[2].canvas.add_wire(
        SchematicWire::new(
            4,
            vec![WireSegment::new(Pos2::new(0.0, 0.0), Pos2::new(80.0, 0.0))],
        )
        .with_net_name("VDD_5V"),
    );

    let cross_nets = manager.resolve_cross_sheet_nets();

    // VCC appears across all 3 sheets
    assert!(cross_nets.contains_key("VCC"));
    let vcc_locations = cross_nets.get("VCC").unwrap();
    assert_eq!(vcc_locations.len(), 3);

    // VDD_5V appears across 2 sheets (Power and Sensors)
    assert!(cross_nets.contains_key("VDD_5V"));
    let vdd_locations = cross_nets.get("VDD_5V").unwrap();
    assert_eq!(vdd_locations.len(), 2);

    // LOCAL_CLK appears on only 1 sheet (Logic), so it must NOT be in cross-sheet nets
    assert!(!cross_nets.contains_key("LOCAL_CLK"));
}

#[test]
fn test_throughput_benchmark_flattening_1000_subcircuit_instances() {
    let mut internal_canvas = SchematicCanvas::new();
    let r1 = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(0.0, 0.0), 1)
        .with_property("net:1", "in")
        .with_property("net:2", "tap");
    let c1 = SchematicComponent::new(2, ComponentKind::Capacitor, Pos2::new(20.0, 0.0), 1)
        .with_property("net:1", "tap")
        .with_property("net:2", "0");
    let r2 = SchematicComponent::new(3, ComponentKind::Resistor, Pos2::new(40.0, 0.0), 2)
        .with_property("net:1", "tap")
        .with_property("net:2", "out");
    internal_canvas.add_component(r1);
    internal_canvas.add_component(c1);
    internal_canvas.add_component(r2);

    let pins = vec![
        SubcircuitPin::new("IN", PinDirection::Input, "in", Pos2::new(-20.0, 0.0)),
        SubcircuitPin::new("OUT", PinDirection::Output, "out", Pos2::new(20.0, 0.0)),
    ];
    let def = SubcircuitDefinition::new("CELL", pins, internal_canvas);
    let mut defs = HashMap::new();
    defs.insert("CELL".to_string(), def);

    let count = 1000;
    let mut canvas = SchematicCanvas::new();
    for i in 1..=count {
        let mut inst = SubcircuitInstance::new(i, "CELL", Pos2::new(i as f32 * 20.0, 0.0));
        inst.map_pin("IN", format!("node_{}", i - 1));
        inst.map_pin("OUT", format!("node_{}", i));
        canvas.add_subcircuit_instance(inst);
    }

    let start = Instant::now();
    let netlist = flatten_hierarchical_netlist(&canvas, &defs).expect("Flattening 1000 instances failed");
    let elapsed = start.elapsed();

    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
    let ops_per_sec = count as f64 / elapsed.as_secs_f64();

    println!(
        "Flattening 1,000 instances: {:.3} ms ({:.0} inst/sec)",
        elapsed_ms, ops_per_sec
    );

    assert!(
        elapsed_ms < 10.0,
        "Hierarchical flattening of 1,000 instances took {:.3} ms (threshold < 10 ms)",
        elapsed_ms
    );
    assert!(
        ops_per_sec > 100_000.0,
        "Throughput {:.0} inst/sec below 100,000 threshold",
        ops_per_sec
    );

    // Verify presence of first and last instance elements
    assert!(netlist.contains("R1_X1"));
    assert!(netlist.contains("X1_tap"));
    assert!(netlist.contains("R1_X1000"));
    assert!(netlist.contains("X1000_tap"));
    assert!(netlist.contains("node_0"));
    assert!(netlist.contains("node_1000"));
}
