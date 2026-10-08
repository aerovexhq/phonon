#![deny(unsafe_code)]

//! Digital & Arithmetics, Multi-Input Logic Gates, Compressed Wires (Buses),
//! Splitters/Mergers, and Float Operations Test Suite (Phase 440).

use egui::{Pos2, Vec2};
use phonon_gui::schematic::{
    deserialize_project, serialize_project, ComponentKind, SchematicComponent, SchematicWire,
    WireSegment,
};

#[test]
fn test_multi_input_logic_gates_pin_generation_and_grid_snapping() {
    let gate_kinds = [
        ComponentKind::AndGate,
        ComponentKind::OrGate,
        ComponentKind::NandGate,
        ComponentKind::NorGate,
        ComponentKind::XorGate,
        ComponentKind::XnorGate,
    ];

    for &kind in &gate_kinds {
        for n in 2..=16 {
            let mut comp = SchematicComponent::new(1, kind, Pos2::new(100.0, 100.0), 1);
            comp.set_input_count(n);
            assert_eq!(comp.input_count(), n, "Input count must be set to {}", n);

            let pins = comp.dynamic_pin_definitions();
            assert_eq!(
                pins.len(),
                n + 1,
                "Gate {:?} with input_count {} must produce exactly {} pins",
                kind,
                n,
                n + 1
            );

            // Verify the last pin is the output pin
            assert_eq!(pins.last().unwrap().0, "OUT");
            assert_eq!(pins.last().unwrap().1, Vec2::new(40.0, 0.0));

            // Verify all input pins and the output pin have coordinates snapped strictly to the 20.0 px grid
            for &(name, offset) in &pins {
                assert!(
                    (offset.x % 20.0).abs() < 1e-4,
                    "Pin {} of {:?} (N={}) has off-grid local x: {}",
                    name,
                    kind,
                    n,
                    offset.x
                );
                assert!(
                    (offset.y % 20.0).abs() < 1e-4,
                    "Pin {} of {:?} (N={}) has off-grid local y: {}",
                    name,
                    kind,
                    n,
                    offset.y
                );
            }

            // Verify world coordinates across all 4 rotations and 2 mirror configurations
            for rot in 0..4 {
                for &mirrored in &[false, true] {
                    comp.rotation = rot;
                    comp.mirrored = mirrored;
                    for &(pin_name, world_pt) in &comp.all_pins() {
                        let rx = world_pt.x % 20.0;
                        let ry = world_pt.y % 20.0;
                        let x_snapped = rx.abs() < 1e-3 || (rx.abs() - 20.0).abs() < 1e-3;
                        let y_snapped = ry.abs() < 1e-3 || (ry.abs() - 20.0).abs() < 1e-3;
                        assert!(
                            x_snapped && y_snapped,
                            "Pin {} world position {:?} for {:?} (N={}, rot={}, mir={}) not on 20px grid",
                            pin_name,
                            world_pt,
                            kind,
                            n,
                            rot,
                            mirrored
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn test_multi_input_gate_bounding_box_scaling() {
    let mut comp2 = SchematicComponent::new(1, ComponentKind::AndGate, Pos2::new(200.0, 200.0), 1);
    comp2.set_input_count(2);
    let bb2 = comp2.bounding_box();

    let mut comp8 = SchematicComponent::new(2, ComponentKind::AndGate, Pos2::new(200.0, 200.0), 2);
    comp8.set_input_count(8);
    let bb8 = comp8.bounding_box();

    let mut comp16 = SchematicComponent::new(3, ComponentKind::AndGate, Pos2::new(200.0, 200.0), 3);
    comp16.set_input_count(16);
    let bb16 = comp16.bounding_box();

    assert!(
        bb8.height() > bb2.height(),
        "Bounding box height of 8-input gate ({}) must exceed 2-input gate ({})",
        bb8.height(),
        bb2.height()
    );
    assert!(
        bb16.height() > bb8.height(),
        "Bounding box height of 16-input gate ({}) must exceed 8-input gate ({})",
        bb16.height(),
        bb8.height()
    );
}

#[test]
fn test_high_level_arithmetic_blocks_configuration() {
    let arith_kinds = [
        ComponentKind::Adder,
        ComponentKind::Subtractor,
        ComponentKind::Multiplier,
        ComponentKind::Divider,
        ComponentKind::ArithmeticLogicUnit,
    ];

    for &kind in &arith_kinds {
        let mut comp = SchematicComponent::new(1, kind, Pos2::new(300.0, 300.0), 1);

        // Test bit-width setting
        comp.set_bit_width(16);
        assert_eq!(comp.bit_width(), 16);
        comp.set_bit_width(64);
        assert_eq!(comp.bit_width(), 64);

        // Test clamping (1..=128)
        comp.set_bit_width(0);
        assert_eq!(comp.bit_width(), 1);
        comp.set_bit_width(256);
        assert_eq!(comp.bit_width(), 128);

        // Test input partitions
        comp.set_input_partitions("8,8");
        assert_eq!(comp.parsed_input_partitions(), vec![8, 8]);

        comp.set_input_partitions("16, 16, 16, 16");
        assert_eq!(comp.parsed_input_partitions(), vec![16, 16, 16, 16]);

        // Verify pins are on grid
        let pins = comp.pin_definitions();
        for &(name, offset) in &pins {
            assert!(
                (offset.x % 20.0).abs() < 1e-4,
                "Pin {} on {:?} has off-grid x: {}",
                name,
                kind,
                offset.x
            );
            assert!(
                (offset.y % 20.0).abs() < 1e-4,
                "Pin {} on {:?} has off-grid y: {}",
                name,
                kind,
                offset.y
            );
        }
    }
}

#[test]
fn test_bit_splitter_and_merger_partitions_and_pins() {
    // 1. BitSplitter
    let mut splitter = SchematicComponent::new(1, ComponentKind::BitSplitter, Pos2::new(400.0, 400.0), 1);
    splitter.set_bit_width(16);
    splitter.set_input_partitions("4,4,4,4");
    assert_eq!(splitter.parsed_input_partitions(), vec![4, 4, 4, 4]);

    let split_pins = splitter.dynamic_pin_definitions();
    // 1 input + 4 outputs = 5 pins
    assert_eq!(split_pins.len(), 5);
    assert_eq!(split_pins[0].0, "IN");
    assert_eq!(split_pins[0].1, Vec2::new(-40.0, 0.0));
    assert_eq!(split_pins[1].0, "OUT0");
    assert_eq!(split_pins[2].0, "OUT1");
    assert_eq!(split_pins[3].0, "OUT2");
    assert_eq!(split_pins[4].0, "OUT3");

    for &(name, offset) in &split_pins {
        assert!(
            (offset.x % 20.0).abs() < 1e-4 && (offset.y % 20.0).abs() < 1e-4,
            "Splitter pin {} has off-grid coordinates {:?}",
            name,
            offset
        );
    }

    // 2. BitMerger
    let mut merger = SchematicComponent::new(2, ComponentKind::BitMerger, Pos2::new(500.0, 500.0), 2);
    merger.set_bit_width(8);
    merger.set_input_partitions("2,2,2,2");
    assert_eq!(merger.parsed_input_partitions(), vec![2, 2, 2, 2]);

    let merge_pins = merger.dynamic_pin_definitions();
    // 4 inputs + 1 output = 5 pins
    assert_eq!(merge_pins.len(), 5);
    assert_eq!(merge_pins[0].0, "IN0");
    assert_eq!(merge_pins[1].0, "IN1");
    assert_eq!(merge_pins[2].0, "IN2");
    assert_eq!(merge_pins[3].0, "IN3");
    assert_eq!(merge_pins[4].0, "OUT");
    assert_eq!(merge_pins[4].1, Vec2::new(40.0, 0.0));

    for &(name, offset) in &merge_pins {
        assert!(
            (offset.x % 20.0).abs() < 1e-4 && (offset.y % 20.0).abs() < 1e-4,
            "Merger pin {} has off-grid coordinates {:?}",
            name,
            offset
        );
    }

    // 3. BusTap
    let tap = SchematicComponent::new(3, ComponentKind::BusTap, Pos2::new(600.0, 600.0), 3);
    let tap_pins = tap.pin_definitions();
    assert_eq!(tap_pins.len(), 3);
    for &(name, offset) in &tap_pins {
        assert!(
            (offset.x % 20.0).abs() < 1e-4 && (offset.y % 20.0).abs() < 1e-4,
            "BusTap pin {} has off-grid coordinates {:?}",
            name,
            offset
        );
    }
}

#[test]
fn test_floating_point_blocks() {
    let float_kinds = [
        ComponentKind::FloatAdder,
        ComponentKind::FloatSubtractor,
        ComponentKind::FloatMultiplier,
        ComponentKind::FloatDivider,
        ComponentKind::FloatComparator,
    ];

    for &kind in &float_kinds {
        let mut comp = SchematicComponent::new(1, kind, Pos2::new(200.0, 200.0), 1);
        assert_eq!(comp.float_precision(), "FP32");

        comp.set_float_precision("FP64");
        assert_eq!(comp.float_precision(), "FP64");

        comp.set_float_precision("FP16");
        assert_eq!(comp.float_precision(), "FP16");

        let pins = comp.pin_definitions();
        assert!(!pins.is_empty());
        for &(name, offset) in &pins {
            assert!(
                (offset.x % 20.0).abs() < 1e-4 && (offset.y % 20.0).abs() < 1e-4,
                "Float component {:?} pin {} has off-grid coords {:?}",
                kind,
                name,
                offset
            );
        }
    }
}

#[test]
fn test_compressed_bus_wire_properties_and_serialization() {
    // 1. Single scalar wire (bit_width = 1)
    let w1 = SchematicWire::new(
        1,
        vec![WireSegment::new(Pos2::new(100.0, 100.0), Pos2::new(200.0, 100.0))],
    );
    assert_eq!(w1.bit_width(), 1);
    assert!(!w1.is_bus());

    // 2. Compressed multi-bit bus wire (bit_width = 8)
    let w2 = SchematicWire::new(
        2,
        vec![
            WireSegment::new(Pos2::new(200.0, 100.0), Pos2::new(200.0, 200.0)),
            WireSegment::new(Pos2::new(200.0, 200.0), Pos2::new(300.0, 200.0)),
        ],
    )
    .with_bit_width(8)
    .with_net_name("DATA_BUS[7:0]");

    assert_eq!(w2.bit_width(), 8);
    assert!(w2.is_bus());
    assert_eq!(w2.net_name.as_deref(), Some("DATA_BUS[7:0]"));

    // 3. Binary serialization round-trip
    let comp = SchematicComponent::new(1, ComponentKind::Adder, Pos2::new(100.0, 100.0), 1);
    let title = "Bus Architecture Test Project";
    let wires = vec![w1, w2];

    let bytes = serialize_project(title, &[comp], &wires);
    let deserialized = deserialize_project(&bytes).expect("Deserialization must succeed");

    assert_eq!(deserialized.wires.len(), 2);
    assert_eq!(deserialized.wires[0].bit_width, 1);
    assert!(!deserialized.wires[0].is_bus());

    assert_eq!(deserialized.wires[1].bit_width, 8);
    assert!(deserialized.wires[1].is_bus());
    assert_eq!(deserialized.wires[1].net_name.as_deref(), Some("DATA_BUS[7:0]"));
}
