#![deny(unsafe_code)]

//! Analytical verification and high-throughput benchmark test suite for
//! Visual Electrical Rules Check (ERC) Diagnostic Overlay Engine.

use egui::Pos2;
use phonon_gui::schematic::{
    ComponentKind, ErcCode, ErcEngine, ErcSeverity, SchematicCanvas, SchematicComponent,
    SchematicWire,
};

#[test]
fn test_clean_voltage_divider_circuit_has_zero_errors() {
    let mut canvas = SchematicCanvas::new();

    let v1 =
        SchematicComponent::new(1, ComponentKind::VoltageSource, Pos2::new(200.0, 300.0), 1);
    let r1 = SchematicComponent::new(2, ComponentKind::Resistor, Pos2::new(360.0, 240.0), 1);
    let r2 = SchematicComponent::new(3, ComponentKind::Resistor, Pos2::new(360.0, 360.0), 2);
    let gnd = SchematicComponent::new(4, ComponentKind::Ground, Pos2::new(200.0, 440.0), 1);

    canvas.add_component(v1);
    canvas.add_component(r1);
    canvas.add_component(r2);
    canvas.add_component(gnd);

    let w1 =
        SchematicWire::manhattan_route(1, Pos2::new(200.0, 260.0), Pos2::new(360.0, 200.0));
    let w2 =
        SchematicWire::manhattan_route(2, Pos2::new(360.0, 280.0), Pos2::new(360.0, 320.0));
    let w3 =
        SchematicWire::manhattan_route(3, Pos2::new(360.0, 400.0), Pos2::new(200.0, 340.0));
    let w4 =
        SchematicWire::manhattan_route(4, Pos2::new(200.0, 340.0), Pos2::new(200.0, 420.0));

    canvas.add_wire(w1);
    canvas.add_wire(w2);
    canvas.add_wire(w3);
    canvas.add_wire(w4);

    let diagnostics = ErcEngine::evaluate_canvas(&canvas);
    let error_count = diagnostics
        .iter()
        .filter(|d| d.severity == ErcSeverity::Error)
        .count();
    let warn_count = diagnostics
        .iter()
        .filter(|d| d.severity == ErcSeverity::Warning)
        .count();

    assert_eq!(error_count, 0, "Clean voltage divider must have 0 errors");
    assert_eq!(warn_count, 0, "Clean voltage divider must have 0 warnings");
    assert_eq!(
        diagnostics.len(),
        0,
        "Clean voltage divider must have strictly 0 diagnostic findings"
    );
}

#[test]
fn test_detection_of_floating_pin_or_hanging_wire() {
    let mut canvas = SchematicCanvas::new();

    let r1 = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(100.0, 100.0), 1);
    let gnd = SchematicComponent::new(2, ComponentKind::Ground, Pos2::new(100.0, 200.0), 1);
    canvas.add_component(r1);
    canvas.add_component(gnd);

    // Connect R1 pin 2 to Ground, leave pin 1 floating
    let w1 =
        SchematicWire::manhattan_route(1, Pos2::new(100.0, 140.0), Pos2::new(100.0, 180.0));
    canvas.add_wire(w1);

    // Add standalone hanging wire with no pin connections
    let hanging =
        SchematicWire::manhattan_route(2, Pos2::new(300.0, 300.0), Pos2::new(300.0, 350.0));
    canvas.add_wire(hanging);

    let diagnostics = ErcEngine::evaluate_canvas(&canvas);

    let floating_pin_diag = diagnostics
        .iter()
        .find(|d| d.code == ErcCode::FloatingNode && d.component_ids.contains(&1));
    assert!(
        floating_pin_diag.is_some(),
        "Must detect floating pin 1 on resistor R1"
    );
    assert_eq!(
        floating_pin_diag.unwrap().pin_positions[0],
        Pos2::new(100.0, 60.0),
        "Pin position must match R1 pin 1 coordinate"
    );

    let hanging_wire_diag = diagnostics
        .iter()
        .find(|d| d.code == ErcCode::FloatingNode && d.message.contains("Wire #2"));
    assert!(
        hanging_wire_diag.is_some(),
        "Must detect unconnected hanging wire #2"
    );
}

#[test]
fn test_detection_of_missing_ground_reference() {
    let mut canvas = SchematicCanvas::new();

    let v1 =
        SchematicComponent::new(1, ComponentKind::VoltageSource, Pos2::new(100.0, 100.0), 1);
    let r1 = SchematicComponent::new(2, ComponentKind::Resistor, Pos2::new(200.0, 100.0), 1);
    canvas.add_component(v1);
    canvas.add_component(r1);

    let w1 =
        SchematicWire::manhattan_route(1, Pos2::new(100.0, 60.0), Pos2::new(200.0, 60.0));
    let w2 =
        SchematicWire::manhattan_route(2, Pos2::new(100.0, 140.0), Pos2::new(200.0, 140.0));
    canvas.add_wire(w1);
    canvas.add_wire(w2);

    let diagnostics = ErcEngine::evaluate_canvas(&canvas);
    let ground_diag = diagnostics
        .iter()
        .find(|d| d.code == ErcCode::UnreferencedGround);

    assert!(ground_diag.is_some(), "Must report unreferenced ground when GND component missing");
    assert_eq!(
        ground_diag.unwrap().severity,
        ErcSeverity::Warning,
        "Missing ground must be a warning"
    );
}

#[test]
fn test_detection_of_short_circuited_voltage_source_and_shorted_resistor() {
    let mut canvas = SchematicCanvas::new();

    let v1 =
        SchematicComponent::new(1, ComponentKind::VoltageSource, Pos2::new(100.0, 100.0), 1);
    let r1 = SchematicComponent::new(2, ComponentKind::Resistor, Pos2::new(250.0, 100.0), 1);
    let gnd = SchematicComponent::new(3, ComponentKind::Ground, Pos2::new(100.0, 250.0), 1);
    canvas.add_component(v1);
    canvas.add_component(r1);
    canvas.add_component(gnd);

    // Short-circuit V1: connect positive terminal to negative terminal
    let w_short_v =
        SchematicWire::manhattan_route(1, Pos2::new(100.0, 60.0), Pos2::new(100.0, 140.0));
    // Short-circuit R1: connect pin 1 to pin 2
    let w_short_r =
        SchematicWire::manhattan_route(2, Pos2::new(250.0, 60.0), Pos2::new(250.0, 140.0));
    // Connect to GND
    let w_gnd =
        SchematicWire::manhattan_route(3, Pos2::new(100.0, 140.0), Pos2::new(100.0, 230.0));

    canvas.add_wire(w_short_v);
    canvas.add_wire(w_short_r);
    canvas.add_wire(w_gnd);

    let diagnostics = ErcEngine::evaluate_canvas(&canvas);

    let short_v_diag = diagnostics
        .iter()
        .find(|d| d.code == ErcCode::ShortCircuitedSource);
    assert!(
        short_v_diag.is_some(),
        "Must detect short-circuited voltage source"
    );
    assert_eq!(
        short_v_diag.unwrap().severity,
        ErcSeverity::Error,
        "Short-circuited source must be an Error"
    );

    let short_r_diag = diagnostics
        .iter()
        .find(|d| d.code == ErcCode::ShortCircuitedPassive);
    assert!(
        short_r_diag.is_some(),
        "Must detect short-circuited passive resistor"
    );
    assert_eq!(
        short_r_diag.unwrap().severity,
        ErcSeverity::Warning,
        "Short-circuited passive must be a Warning"
    );
}

#[test]
fn test_detection_of_duplicate_designators() {
    let mut canvas = SchematicCanvas::new();

    let mut r1_a = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(100.0, 100.0), 1);
    r1_a.name = "R1".to_string();
    let mut r1_b = SchematicComponent::new(2, ComponentKind::Resistor, Pos2::new(200.0, 100.0), 1);
    r1_b.name = "R1".to_string();
    let gnd = SchematicComponent::new(3, ComponentKind::Ground, Pos2::new(100.0, 200.0), 1);

    canvas.add_component(r1_a);
    canvas.add_component(r1_b);
    canvas.add_component(gnd);

    let diagnostics = ErcEngine::evaluate_canvas(&canvas);
    let dup_diag = diagnostics
        .iter()
        .find(|d| d.code == ErcCode::DuplicateDesignator);

    assert!(dup_diag.is_some(), "Must detect duplicate R1 designator");
    assert_eq!(
        dup_diag.unwrap().severity,
        ErcSeverity::Error,
        "Duplicate designator must be an Error"
    );
    assert_eq!(
        dup_diag.unwrap().component_ids.len(),
        2,
        "Must list both duplicate component IDs"
    );
}

#[test]
fn test_detection_of_floating_mosfet_bulk_terminal() {
    let mut canvas = SchematicCanvas::new();

    let fet = SchematicComponent::new(1, ComponentKind::FinFet, Pos2::new(200.0, 200.0), 1);
    let v1 =
        SchematicComponent::new(2, ComponentKind::VoltageSource, Pos2::new(100.0, 200.0), 1);
    let gnd = SchematicComponent::new(3, ComponentKind::Ground, Pos2::new(100.0, 300.0), 1);

    canvas.add_component(fet);
    canvas.add_component(v1);
    canvas.add_component(gnd);

    // Connect Gate (180, 200) to V1+ (100, 160)
    let w1 =
        SchematicWire::manhattan_route(1, Pos2::new(180.0, 200.0), Pos2::new(100.0, 160.0));
    // Connect Source (220, 240) to V1- (100, 240)
    let w2 =
        SchematicWire::manhattan_route(2, Pos2::new(220.0, 240.0), Pos2::new(100.0, 240.0));
    // Connect Drain (220, 160) to V1+ (100, 160)
    let w3 =
        SchematicWire::manhattan_route(3, Pos2::new(220.0, 160.0), Pos2::new(100.0, 160.0));
    // Connect V1- to GND (100, 280)
    let w4 =
        SchematicWire::manhattan_route(4, Pos2::new(100.0, 240.0), Pos2::new(100.0, 280.0));

    canvas.add_wire(w1);
    canvas.add_wire(w2);
    canvas.add_wire(w3);
    canvas.add_wire(w4);
    // Bulk pin 'B' at (220, 200) is left completely unconnected!

    let diagnostics = ErcEngine::evaluate_canvas(&canvas);
    let substrate_diag = diagnostics
        .iter()
        .find(|d| d.code == ErcCode::InvalidSubstrate);

    assert!(
        substrate_diag.is_some(),
        "Must detect unconnected 4-terminal FET substrate bulk pin"
    );
    assert_eq!(
        substrate_diag.unwrap().severity,
        ErcSeverity::Warning,
        "Unconnected substrate bulk must be a Warning"
    );
    assert_eq!(
        substrate_diag.unwrap().pin_positions[0],
        Pos2::new(220.0, 200.0),
        "Pin position must match FinFet bulk terminal coordinates"
    );
}

#[test]
fn test_pin_coordinate_calculations_match_canvas_pin_locations() {
    let mut canvas = SchematicCanvas::new();
    let mut r1 = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(150.0, 250.0), 1);
    r1.rotation = 0;
    canvas.add_component(r1);

    // Initial rotation 0
    let diags_0 = ErcEngine::evaluate_canvas(&canvas);
    let pin1_diag_0 = diags_0
        .iter()
        .find(|d| d.code == ErcCode::FloatingNode && d.message.contains("Pin '1'"))
        .expect("Pin 1 must be floating");
    assert_eq!(
        pin1_diag_0.pin_positions[0],
        canvas.components[0].pin_world_pos(0).unwrap(),
        "Diagnostic pin position must match component pin_world_pos at rotation 0"
    );

    // Rotate component 90 degrees CW
    canvas.components[0].rotate_clockwise();
    assert_eq!(canvas.components[0].rotation, 1);

    let diags_90 = ErcEngine::evaluate_canvas(&canvas);
    let pin1_diag_90 = diags_90
        .iter()
        .find(|d| d.code == ErcCode::FloatingNode && d.message.contains("Pin '1'"))
        .expect("Pin 1 must be floating after rotation");
    assert_eq!(
        pin1_diag_90.pin_positions[0],
        canvas.components[0].pin_world_pos(0).unwrap(),
        "Diagnostic pin position must match component pin_world_pos at rotation 90 deg"
    );
}

#[test]
fn test_throughput_benchmark_exceeds_100k_erc_evaluations_per_second() {
    let mut canvas = SchematicCanvas::new();

    let v1 =
        SchematicComponent::new(1, ComponentKind::VoltageSource, Pos2::new(200.0, 300.0), 1);
    let r1 = SchematicComponent::new(2, ComponentKind::Resistor, Pos2::new(360.0, 240.0), 1);
    let r2 = SchematicComponent::new(3, ComponentKind::Resistor, Pos2::new(360.0, 360.0), 2);
    let gnd = SchematicComponent::new(4, ComponentKind::Ground, Pos2::new(200.0, 440.0), 1);

    canvas.add_component(v1);
    canvas.add_component(r1);
    canvas.add_component(r2);
    canvas.add_component(gnd);

    let w1 =
        SchematicWire::manhattan_route(1, Pos2::new(200.0, 260.0), Pos2::new(360.0, 200.0));
    let w2 =
        SchematicWire::manhattan_route(2, Pos2::new(360.0, 280.0), Pos2::new(360.0, 320.0));
    let w3 =
        SchematicWire::manhattan_route(3, Pos2::new(360.0, 400.0), Pos2::new(200.0, 340.0));
    let w4 =
        SchematicWire::manhattan_route(4, Pos2::new(200.0, 340.0), Pos2::new(200.0, 420.0));

    canvas.add_wire(w1);
    canvas.add_wire(w2);
    canvas.add_wire(w3);
    canvas.add_wire(w4);

    let iterations = 100_000;
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _diags = ErcEngine::evaluate_canvas(&canvas);
    }
    let elapsed = start.elapsed();
    let throughput = iterations as f64 / elapsed.as_secs_f64();

    println!(
        "ERC Engine Throughput: {:.2} evals/sec ({:?} for {} iterations)",
        throughput, elapsed, iterations
    );

    assert!(
        throughput > 100_000.0,
        "ERC engine throughput must exceed 100,000 evals/sec, achieved {:.2}",
        throughput
    );
}
