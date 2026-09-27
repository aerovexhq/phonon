//! Integration tests for schematic DSU net extraction and circuit compilation.

use egui::Pos2;
use phonon_gui::schematic::{compile_schematic, ComponentKind, SchematicComponent, SchematicWire};
use phonon_solver::{solve_dc_non_linear, NewtonOptions};

#[test]
fn test_voltage_divider_compilation_and_dc_solve() {
    // 1. Instantiate components
    let v1 = SchematicComponent::new(1, ComponentKind::VoltageSource, Pos2::new(200.0, 300.0), 1);
    let r1 = SchematicComponent::new(2, ComponentKind::Resistor, Pos2::new(360.0, 240.0), 1);
    let r2 = SchematicComponent::new(3, ComponentKind::Resistor, Pos2::new(360.0, 360.0), 2);
    let gnd = SchematicComponent::new(4, ComponentKind::Ground, Pos2::new(200.0, 440.0), 1);

    let components = vec![v1, r1, r2, gnd];

    // 2. Wire connections
    // V1(+) (200, 260) -> R1(1) (360, 200)
    let w1 = SchematicWire::manhattan_route(1, Pos2::new(200.0, 260.0), Pos2::new(360.0, 200.0));
    // R1(2) (360, 280) -> R2(1) (360, 320)
    let w2 = SchematicWire::manhattan_route(2, Pos2::new(360.0, 280.0), Pos2::new(360.0, 320.0));
    // R2(2) (360, 400) -> V1(-) (200, 340)
    let w3 = SchematicWire::manhattan_route(3, Pos2::new(360.0, 400.0), Pos2::new(200.0, 340.0));
    // V1(-) (200, 340) -> GND (200, 420)
    let w4 = SchematicWire::manhattan_route(4, Pos2::new(200.0, 340.0), Pos2::new(200.0, 420.0));

    let wires = vec![w1, w2, w3, w4];

    // 3. Compile schematic
    let compiled = compile_schematic(&components, &wires).expect("Compilation must succeed");

    assert!(compiled.net_names.contains(&"0".to_string()));
    assert_eq!(compiled.graph.num_components(), 3); // V1, R1, R2

    // 4. Solve DC Operating Point
    let sol = solve_dc_non_linear(
        &compiled.graph,
        &compiled.model_ctx,
        &NewtonOptions::default(),
    )
    .expect("DC solve must converge");

    let n_mid = compiled
        .pin_to_net
        .get(&("R1".to_string(), "2".to_string()))
        .expect("R1 pin 2 must be connected");
    let n_in = compiled
        .pin_to_net
        .get(&("V1".to_string(), "+".to_string()))
        .expect("V1 pin + must be connected");

    let node_mid_id = compiled.graph.get_node(n_mid).unwrap();
    let node_in_id = compiled.graph.get_node(n_in).unwrap();

    let v_in = sol.node_voltages[node_in_id.0 as usize];
    let v_mid = sol.node_voltages[node_mid_id.0 as usize];

    assert!(
        (v_in - 5.0).abs() < 1e-6,
        "Input node must be 5.0V, got {}",
        v_in
    );
    assert!(
        (v_mid - 2.5).abs() < 1e-6,
        "Midpoint node must be 2.5V, got {}",
        v_mid
    );
}

#[test]
fn test_component_rotation_geometry() {
    let mut r = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(100.0, 100.0), 1);
    assert_eq!(r.rotation, 0);

    // Initial pin positions: (100, 60) and (100, 140)
    let p1 = r.pin_world_pos(0).unwrap();
    let p2 = r.pin_world_pos(1).unwrap();
    assert_eq!(p1, Pos2::new(100.0, 60.0));
    assert_eq!(p2, Pos2::new(100.0, 140.0));

    // Rotate 90° clockwise: (x, y) offset (0, -40) -> (-(-40), 0) = (40, 0)
    r.rotate_clockwise();
    assert_eq!(r.rotation, 1);
    let p1_rot = r.pin_world_pos(0).unwrap();
    let p2_rot = r.pin_world_pos(1).unwrap();
    assert_eq!(p1_rot, Pos2::new(140.0, 100.0));
    assert_eq!(p2_rot, Pos2::new(60.0, 100.0));
}
