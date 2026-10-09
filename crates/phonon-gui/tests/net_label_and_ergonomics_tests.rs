#![deny(unsafe_code)]

//! Comprehensive test suite for Net Labels, Wire Crossing Hop Bridges, and Schematic Ergonomics.

use egui::Pos2;
use phonon_gui::schematic::{
    compile_schematic_with_labels, compute_wire_crossings, CanvasCommand, ComponentKind,
    ErcCode, ErcEngine, HistoryStack, NetLabel, NetLabelOrientation, SchematicCanvas,
    SchematicComponent, SchematicWire, WireSegment,
};

#[test]
fn test_net_label_creation_and_orientation_rotation() {
    let lbl = NetLabel::new(1, "VCC", Pos2::new(100.0, 200.0));
    assert_eq!(lbl.id, 1);
    assert_eq!(lbl.name, "VCC");
    assert_eq!(lbl.pos, Pos2::new(100.0, 200.0));
    assert_eq!(lbl.orientation, NetLabelOrientation::East);

    // Quarter-turn cycle tests
    assert_eq!(NetLabelOrientation::from_quarter_turns(0), NetLabelOrientation::East);
    assert_eq!(NetLabelOrientation::from_quarter_turns(1), NetLabelOrientation::South);
    assert_eq!(NetLabelOrientation::from_quarter_turns(2), NetLabelOrientation::West);
    assert_eq!(NetLabelOrientation::from_quarter_turns(3), NetLabelOrientation::North);
    assert_eq!(NetLabelOrientation::from_quarter_turns(4), NetLabelOrientation::East);

    assert_eq!(NetLabelOrientation::East.to_quarter_turns(), 0);
    assert_eq!(NetLabelOrientation::South.to_quarter_turns(), 1);
    assert_eq!(NetLabelOrientation::West.to_quarter_turns(), 2);
    assert_eq!(NetLabelOrientation::North.to_quarter_turns(), 3);

    let rot1 = lbl.orientation.rotate_clockwise();
    assert_eq!(rot1, NetLabelOrientation::South);
    let rot2 = rot1.rotate_clockwise();
    assert_eq!(rot2, NetLabelOrientation::West);
    let rot3 = rot2.rotate_clockwise();
    assert_eq!(rot3, NetLabelOrientation::North);
    let rot4 = rot3.rotate_clockwise();
    assert_eq!(rot4, NetLabelOrientation::East);
}

#[test]
fn test_net_label_bounding_box_and_hit_testing() {
    let lbl_east = NetLabel::new(1, "CLK", Pos2::new(50.0, 50.0));
    let bbox_east = lbl_east.bounding_box();
    assert!(bbox_east.min.x >= 50.0);
    assert!(lbl_east.hit_test(Pos2::new(50.0, 50.0), 2.0));
    assert!(lbl_east.hit_test(bbox_east.center(), 2.0));
    assert!(!lbl_east.hit_test(Pos2::new(10.0, 10.0), 2.0));

    let lbl_west = NetLabel::new(2, "DATA0", Pos2::new(50.0, 50.0))
        .with_orientation(NetLabelOrientation::West);
    let bbox_west = lbl_west.bounding_box();
    assert!(bbox_west.max.x <= 50.0);
    assert!(lbl_west.hit_test(bbox_west.center(), 2.0));

    let lbl_north = NetLabel::new(3, "RESET", Pos2::new(50.0, 50.0))
        .with_orientation(NetLabelOrientation::North);
    let bbox_north = lbl_north.bounding_box();
    assert!(bbox_north.max.y <= 50.0);
    assert!(lbl_north.hit_test(bbox_north.center(), 2.0));

    let lbl_south = NetLabel::new(4, "ENABLE", Pos2::new(50.0, 50.0))
        .with_orientation(NetLabelOrientation::South);
    let bbox_south = lbl_south.bounding_box();
    assert!(bbox_south.min.y >= 50.0);
    assert!(lbl_south.hit_test(bbox_south.center(), 2.0));
}

#[test]
fn test_net_label_electrical_unioning_without_wires() {
    // Create R1 at (100, 100) and R2 at (300, 100)
    let r1 = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(100.0, 100.0), 1);
    let r2 = SchematicComponent::new(2, ComponentKind::Resistor, Pos2::new(300.0, 100.0), 2);

    let r1_pin2_pos = r1.pin_world_pos(1).expect("R1 pin 2 exists");
    let r2_pin1_pos = r2.pin_world_pos(0).expect("R2 pin 1 exists");

    // Add NetLabels named "CLK" to R1 pin 2 and R2 pin 1 without any physical wires
    let lbl1 = NetLabel::new(1, "CLK", r1_pin2_pos);
    let lbl2 = NetLabel::new(2, "CLK", r2_pin1_pos);

    let components = vec![r1, r2];
    let wires = Vec::new();
    let labels = vec![lbl1, lbl2];

    let compiled = compile_schematic_with_labels(&components, &wires, &labels)
        .expect("Schematic should compile cleanly");

    let net_r1_p2 = compiled
        .pin_to_net
        .get(&("R1".to_string(), "2".to_string()))
        .expect("R1 pin 2 mapped");
    let net_r2_p1 = compiled
        .pin_to_net
        .get(&("R2".to_string(), "1".to_string()))
        .expect("R2 pin 1 mapped");

    // Both pins must belong to the exact same net named "CLK"
    assert_eq!(net_r1_p2, net_r2_p1);
    assert_eq!(net_r1_p2, "CLK");
}

#[test]
fn test_ground_reference_via_gnd_net_label() {
    let r1 = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(100.0, 100.0), 1);
    let r1_pin2_pos = r1.pin_world_pos(1).expect("R1 pin 2 exists");

    let gnd_lbl = NetLabel::new(1, "GND", r1_pin2_pos);

    let components = vec![r1];
    let wires = Vec::new();
    let labels = vec![gnd_lbl];

    let compiled = compile_schematic_with_labels(&components, &wires, &labels)
        .expect("Compilation should succeed");

    let net_r1_p2 = compiled
        .pin_to_net
        .get(&("R1".to_string(), "2".to_string()))
        .expect("R1 pin 2 mapped");

    // Ground reference should resolve to "0"
    assert_eq!(net_r1_p2, "0");

    // Check ERC engine evaluation with labels
    let diagnostics = ErcEngine::evaluate_with_labels(&components, &wires, &labels);
    // Pin 2 is connected to GND, so unreferenced ground should not trigger
    let has_ground_missing = diagnostics.iter().any(|d| d.code == ErcCode::UnreferencedGround);
    assert!(!has_ground_missing, "GND NetLabel must satisfy ground reference requirement");
}

#[test]
fn test_wire_crossing_detection_and_bridge_hop_isolation() {
    // Wire 1: Horizontal from (40, 100) to (160, 100)
    let wire_h = SchematicWire {
        id: 1,
        net_name: Some("NET1".to_string()),
        segments: vec![WireSegment::new(Pos2::new(40.0, 100.0), Pos2::new(160.0, 100.0))],
        bit_width: 1,
    };

    // Wire 2: Vertical from (100, 40) to (100, 160)
    let wire_v = SchematicWire {
        id: 2,
        net_name: Some("NET2".to_string()),
        segments: vec![WireSegment::new(Pos2::new(100.0, 40.0), Pos2::new(100.0, 160.0))],
        bit_width: 1,
    };

    let wires = vec![wire_h, wire_v];

    // Case 1: No solder junction dot at crossing point (100, 100) -> crossing hop must be generated
    let crossings_without_junction = compute_wire_crossings(&wires, &[]);
    assert_eq!(crossings_without_junction.len(), 1);
    assert_eq!(crossings_without_junction[0].point, Pos2::new(100.0, 100.0));
    assert!(crossings_without_junction[0].is_horizontal);

    // Case 2: Solder junction dot exists at (100, 100) -> wires are electrically joined, NO bridge hop
    let junctions = vec![Pos2::new(100.0, 100.0)];
    let crossings_with_junction = compute_wire_crossings(&wires, &junctions);
    assert_eq!(crossings_with_junction.len(), 0);

    // Case 3: Parallel wires -> NO crossings
    let parallel_wire = SchematicWire {
        id: 3,
        net_name: Some("NET3".to_string()),
        segments: vec![WireSegment::new(Pos2::new(40.0, 120.0), Pos2::new(160.0, 120.0))],
        bit_width: 1,
    };
    let parallel_crossings = compute_wire_crossings(&[wires[0].clone(), parallel_wire], &[]);
    assert_eq!(parallel_crossings.len(), 0);
}

#[test]
fn test_history_stack_undo_redo_with_net_labels() {
    let mut history = HistoryStack::new();
    let mut components = Vec::new();
    let mut wires = Vec::new();
    let mut labels = Vec::new();

    // 1. Add NetLabel
    let lbl = NetLabel::new(1, "CLK", Pos2::new(100.0, 100.0));
    labels.push(lbl.clone());
    history.record(CanvasCommand::AddNetLabel(lbl.clone()));
    assert_eq!(labels.len(), 1);

    // Undo -> labels empty
    assert!(history.undo_with_labels(&mut components, &mut wires, &mut labels));
    assert_eq!(labels.len(), 0);

    // Redo -> labels restored
    assert!(history.redo_with_labels(&mut components, &mut wires, &mut labels));
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0].name, "CLK");

    // 2. Move NetLabel
    let moved_to = Pos2::new(140.0, 160.0);
    history.record(CanvasCommand::MoveNetLabel {
        id: 1,
        from: labels[0].pos,
        to: moved_to,
    });
    labels[0].pos = moved_to;

    // Undo move
    assert!(history.undo_with_labels(&mut components, &mut wires, &mut labels));
    assert_eq!(labels[0].pos, Pos2::new(100.0, 100.0));

    // Redo move
    assert!(history.redo_with_labels(&mut components, &mut wires, &mut labels));
    assert_eq!(labels[0].pos, moved_to);

    // 3. Delete NetLabel
    let deleted_lbl = labels.remove(0);
    history.record(CanvasCommand::DeleteNetLabel(deleted_lbl));
    assert_eq!(labels.len(), 0);

    // Undo delete
    assert!(history.undo_with_labels(&mut components, &mut wires, &mut labels));
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0].id, 1);
}

#[test]
fn test_schematic_canvas_net_label_selection_and_bounding_box() {
    let mut canvas = SchematicCanvas::new();
    assert!(canvas.is_empty());

    let lbl = NetLabel::new(1, "VOUT", Pos2::new(200.0, 150.0));
    canvas.add_net_label(lbl);
    assert!(!canvas.is_empty());
    assert_eq!(canvas.net_labels.len(), 1);

    // Selection
    assert!(!canvas.is_label_selected(1));
    canvas.select_label(1, false);
    assert!(canvas.is_label_selected(1));

    canvas.toggle_label_selection(1);
    assert!(!canvas.is_label_selected(1));

    canvas.toggle_label_selection(1);
    assert!(canvas.is_label_selected(1));

    canvas.clear_selection();
    assert!(!canvas.is_label_selected(1));

    // Canvas bounding box includes net label
    let bbox = canvas.bounding_box();
    assert!(bbox.is_some());
    assert!(bbox.unwrap().contains(Pos2::new(200.0, 150.0)));

    canvas.clear();
    assert!(canvas.is_empty());
    assert_eq!(canvas.net_labels.len(), 0);
}
