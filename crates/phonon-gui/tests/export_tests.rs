//! Integration tests for schematic SPICE netlist export and round-trip simulation.

use egui::Pos2;
use phonon_gui::schematic::{compile_schematic, ComponentKind, SchematicComponent, SchematicWire};
use phonon_netlist::parse_netlist;
use phonon_solver::{solve_dc_non_linear, NewtonOptions};

#[test]
fn test_spice_export_round_trip() {
    let v1 = SchematicComponent::new(1, ComponentKind::VoltageSource, Pos2::new(200.0, 300.0), 1);
    let r1 = SchematicComponent::new(2, ComponentKind::Resistor, Pos2::new(360.0, 240.0), 1);
    let r2 = SchematicComponent::new(3, ComponentKind::Resistor, Pos2::new(360.0, 360.0), 2);
    let gnd = SchematicComponent::new(4, ComponentKind::Ground, Pos2::new(200.0, 440.0), 1);

    let components = vec![v1, r1, r2, gnd];

    let w1 = SchematicWire::manhattan_route(1, Pos2::new(200.0, 260.0), Pos2::new(360.0, 200.0));
    let w2 = SchematicWire::manhattan_route(2, Pos2::new(360.0, 280.0), Pos2::new(360.0, 320.0));
    let w3 = SchematicWire::manhattan_route(3, Pos2::new(360.0, 400.0), Pos2::new(200.0, 340.0));
    let w4 = SchematicWire::manhattan_route(4, Pos2::new(200.0, 340.0), Pos2::new(200.0, 420.0));

    let wires = vec![w1, w2, w3, w4];

    let compiled = compile_schematic(&components, &wires).expect("Schematic must compile");

    // Check exported netlist format
    assert!(compiled.spice_netlist.contains("V1"));
    assert!(compiled.spice_netlist.contains("R1"));
    assert!(compiled.spice_netlist.contains("R2"));
    assert!(compiled.spice_netlist.contains(".OP"));
    assert!(compiled.spice_netlist.contains(".END"));

    // Parse the generated netlist back using phonon-netlist
    let parsed = parse_netlist(&compiled.spice_netlist).expect("Exported SPICE must parse cleanly");
    assert_eq!(parsed.components.len(), 3); // V1, R1, R2

    // Solve DC directly on the compiled graph
    let sol = solve_dc_non_linear(
        &compiled.graph,
        &compiled.model_ctx,
        &NewtonOptions::default(),
    )
    .expect("DC solution must converge");

    assert!(sol.node_voltages.len() >= 3);
}
