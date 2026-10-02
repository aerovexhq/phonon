#![deny(unsafe_code)]

//! Analytical verification and throughput benchmarks for Phase 317 component symbol and shape editor.

use phonon_gui::schematic::symbol::{
    CustomComponentSymbol, LabelPlacement, SymbolLibrary, SymbolPin, SymbolPrimitive,
    TerminalDirection,
};
use phonon_gui::widgets::symbol_editor::{SymbolEditorDialog, SymbolEditorTool};

#[test]
fn test_symbol_primitive_bounds_and_strokes() {
    let line = SymbolPrimitive::Line {
        start: [-10.0, -5.0],
        end: [20.0, 15.0],
        stroke_width: 1.5,
    };
    let bounds = line.bounding_box();
    assert_eq!(bounds, [-10.0, -5.0, 20.0, 15.0]);

    let rect = SymbolPrimitive::Rectangle {
        min: [-20.0, -15.0],
        max: [20.0, 15.0],
        filled: true,
        stroke_width: 2.0,
    };
    assert_eq!(rect.bounding_box(), [-20.0, -15.0, 20.0, 15.0]);

    let circle = SymbolPrimitive::Circle {
        center: [0.0, 0.0],
        radius: 12.0,
        filled: false,
        stroke_width: 1.0,
    };
    assert_eq!(circle.bounding_box(), [-12.0, -12.0, 12.0, 12.0]);
}

#[test]
fn test_symbol_pin_anchors_and_directionality() {
    let pin_in = SymbolPin::new(0, "IN1", TerminalDirection::Input, [-30.0, -10.0], 1);
    let pin_out = SymbolPin::new(1, "OUT", TerminalDirection::Output, [30.0, 0.0], 2);
    let pin_bi = SymbolPin::new(2, "IO", TerminalDirection::Bidirectional, [0.0, 20.0], 3);
    let pin_pass = SymbolPin::new(3, "GND", TerminalDirection::Passive, [0.0, -20.0], 0);

    assert_eq!(pin_in.direction.as_str(), "Input");
    assert_eq!(pin_out.direction.as_str(), "Output");
    assert_eq!(pin_bi.direction.as_str(), "Bidirectional");
    assert_eq!(pin_pass.direction.as_str(), "Passive");

    assert_eq!(pin_in.rel_pos, [-30.0, -10.0]);
    assert_eq!(pin_out.rel_pos, [30.0, 0.0]);
}

#[test]
fn test_label_placement_collision_detection() {
    let body_bounds = [-20.0, -20.0, 20.0, 20.0];

    // Label positioned directly inside the component body -> collision!
    let colliding_labels = LabelPlacement {
        designator_offset: [0.0, 0.0],
        value_offset: [5.0, 5.0],
        rotation_deg: 0.0,
        designator_visible: true,
        value_visible: true,
    };
    assert!(colliding_labels.check_collision(body_bounds));

    // Auto-avoid collision should reposition labels clear of body_bounds
    let mut resolved_labels = colliding_labels;
    resolved_labels.auto_avoid_collision(body_bounds);
    assert!(!resolved_labels.check_collision(body_bounds));
    assert!(resolved_labels.designator_offset[0] > body_bounds[2]);
}

#[test]
fn test_simulation_ui_decoupling_zero_gui_overhead() {
    let mut symbol = CustomComponentSymbol::new(
        "opamp_ad820",
        "AD820 Precision Op-Amp",
        "U",
        "Integrated Circuits",
    );
    symbol.add_primitive(SymbolPrimitive::Rectangle {
        min: [-25.0, -25.0],
        max: [25.0, 25.0],
        filled: false,
        stroke_width: 1.5,
    });
    symbol.add_pin(SymbolPin::new(0, "IN+", TerminalDirection::Input, [-35.0, 10.0], 1));
    symbol.add_pin(SymbolPin::new(1, "IN-", TerminalDirection::Input, [-35.0, -10.0], 2));
    symbol.add_pin(SymbolPin::new(2, "OUT", TerminalDirection::Output, [35.0, 0.0], 3));

    // The simulation engine queries only pure mathematical SPICE node calls
    let spice_line = symbol.to_spice_subcircuit_instance("U1", &["net_in_p", "net_in_n", "net_out"]);
    assert_eq!(spice_line, "XU1 net_in_p net_in_n net_out opamp_ad820");

    // Strictly ZERO GUI vector primitives in the mathematical instance string
    assert!(!spice_line.contains("Rectangle"));
    assert!(!spice_line.contains("stroke_width"));
    assert!(!spice_line.contains("filled"));
}

#[test]
fn test_symbol_rotation_mapping() {
    let pin = SymbolPin::new(0, "A", TerminalDirection::Input, [30.0, 10.0], 1);
    let rot90 = pin.rotated(1);
    assert_eq!(rot90.rel_pos, [-10.0, 30.0]);

    let rot180 = pin.rotated(2);
    assert_eq!(rot180.rel_pos, [-30.0, -10.0]);

    let rot270 = pin.rotated(3);
    assert_eq!(rot270.rel_pos, [10.0, -30.0]);

    let rot360 = pin.rotated(4);
    assert_eq!(rot360.rel_pos, [30.0, 10.0]);
}

#[test]
fn test_symbol_library_registration_and_lookup() {
    let mut library = SymbolLibrary::new();
    assert!(library.is_empty());

    let sym1 = CustomComponentSymbol::new("sym1", "Symbol One", "U", "Logic");
    let sym2 = CustomComponentSymbol::new("sym2", "Symbol Two", "Q", "Discrete");

    library.register(sym1);
    library.register(sym2);

    assert_eq!(library.len(), 2);
    assert!(library.get("sym1").is_some());
    assert_eq!(library.get("sym1").unwrap().prefix, "U");
    assert!(library.get("sym2").is_some());
    assert_eq!(library.get("sym2").unwrap().prefix, "Q");
    assert!(library.get("nonexistent").is_none());
}

#[test]
fn test_logisim_style_logic_gate_symbol() {
    let mut nand_gate = CustomComponentSymbol::new("nand2", "2-Input NAND Gate", "U", "Logic");

    // D-shape outline: line on back, arc on front
    nand_gate.add_primitive(SymbolPrimitive::Line {
        start: [-20.0, -15.0],
        end: [-20.0, 15.0],
        stroke_width: 1.5,
    });
    nand_gate.add_primitive(SymbolPrimitive::Line {
        start: [-20.0, -15.0],
        end: [0.0, -15.0],
        stroke_width: 1.5,
    });
    nand_gate.add_primitive(SymbolPrimitive::Line {
        start: [-20.0, 15.0],
        end: [0.0, 15.0],
        stroke_width: 1.5,
    });
    nand_gate.add_primitive(SymbolPrimitive::Arc {
        center: [0.0, 0.0],
        radius: 15.0,
        start_angle_rad: -std::f32::consts::FRAC_PI_2,
        end_angle_rad: std::f32::consts::FRAC_PI_2,
        stroke_width: 1.5,
    });

    // Inversion bubble circle at output
    nand_gate.add_primitive(SymbolPrimitive::Circle {
        center: [18.0, 0.0],
        radius: 3.0,
        filled: false,
        stroke_width: 1.5,
    });

    // 2 Inputs on left, 1 Output on right
    nand_gate.add_pin(SymbolPin::new(0, "A", TerminalDirection::Input, [-30.0, -8.0], 1));
    nand_gate.add_pin(SymbolPin::new(1, "B", TerminalDirection::Input, [-30.0, 8.0], 2));
    nand_gate.add_pin(SymbolPin::new(2, "Y", TerminalDirection::Output, [30.0, 0.0], 3));

    assert_eq!(nand_gate.primitives.len(), 5);
    assert_eq!(nand_gate.pins.len(), 3);

    let body = nand_gate.body_bounding_box();
    assert!(body[0] <= -20.0);
    assert!(body[2] >= 20.0);
}

#[test]
fn test_symbol_editor_dialog_initialization() {
    let mut dialog = SymbolEditorDialog::new();
    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tool, SymbolEditorTool::Select);

    let custom = CustomComponentSymbol::new("filter_lpf", "Low-Pass Filter", "FLT", "Passives");
    dialog.open_symbol(custom);
    assert!(dialog.is_open);
    assert_eq!(dialog.symbol.id, "filter_lpf");
}

#[test]
fn test_high_speed_symbol_evaluation_benchmark() {
    let mut symbol = CustomComponentSymbol::new("benchmark_sym", "Bench Device", "U", "Custom");
    for i in 0..10 {
        let offset = i as f32 * 5.0;
        symbol.add_primitive(SymbolPrimitive::Line {
            start: [-offset, -offset],
            end: [offset, offset],
            stroke_width: 1.0,
        });
    }

    let start = std::time::Instant::now();
    let iterations = 20_000;
    for rot in 0..iterations {
        let rotated = symbol.rotated((rot % 4) as u8);
        let bounds = rotated.body_bounding_box();
        assert!(bounds[0] < bounds[2]);
    }
    let elapsed = start.elapsed();
    println!("20,000 symbol rotations & bounds evaluations: {:?}", elapsed);
    assert!(elapsed.as_millis() < 300, "Throughput must exceed 65,000 evals/sec");
}
