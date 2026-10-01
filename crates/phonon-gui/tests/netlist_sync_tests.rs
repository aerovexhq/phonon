#![deny(unsafe_code)]

//! Analytical verification and high-throughput benchmark test suite for
//! Real-Time Interactive Multi-Tier Netlist Synchronization Engine.

use egui::Pos2;
use phonon_gui::schematic::{
    ComponentKind, NetlistSyncEngine, NetlistSyncError, SchematicCanvas, SchematicComponent,
    SchematicWire,
};

#[test]
fn test_canvas_to_spice_netlist_export_and_structure() {
    let mut canvas = SchematicCanvas::new();
    let v1 = SchematicComponent::new(1, ComponentKind::VoltageSource, Pos2::new(100.0, 100.0), 1);
    let r1 = SchematicComponent::new(2, ComponentKind::Resistor, Pos2::new(200.0, 100.0), 1);
    let r2 = SchematicComponent::new(3, ComponentKind::Resistor, Pos2::new(200.0, 200.0), 2);
    let gnd = SchematicComponent::new(4, ComponentKind::Ground, Pos2::new(100.0, 200.0), 1);

    canvas.add_component(v1);
    canvas.add_component(r1);
    canvas.add_component(r2);
    canvas.add_component(gnd);

    let w1 = SchematicWire::manhattan_route(1, Pos2::new(100.0, 60.0), Pos2::new(200.0, 60.0));
    let w2 = SchematicWire::manhattan_route(2, Pos2::new(200.0, 140.0), Pos2::new(200.0, 160.0));
    let w3 = SchematicWire::manhattan_route(3, Pos2::new(200.0, 240.0), Pos2::new(100.0, 140.0));
    let w4 = SchematicWire::manhattan_route(4, Pos2::new(100.0, 140.0), Pos2::new(100.0, 180.0));

    canvas.add_wire(w1);
    canvas.add_wire(w2);
    canvas.add_wire(w3);
    canvas.add_wire(w4);

    let mut engine = NetlistSyncEngine::new();
    let netlist = engine.sync_from_canvas(&canvas);

    assert!(netlist.contains("V1"), "Netlist must contain V1 component");
    assert!(netlist.contains("R1"), "Netlist must contain R1 component");
    assert!(netlist.contains("R2"), "Netlist must contain R2 component");
    assert!(netlist.contains(".TEMP 27.0"), "Netlist must contain .TEMP header");
}

#[test]
fn test_incremental_delta_update_when_netlist_values_change() {
    let mut engine = NetlistSyncEngine::new();
    let mut canvas = SchematicCanvas::new();

    let netlist1 = "* Initial Schematic\nV1 net1 0 10.0\nR1 net1 net2 2.2k\nR2 net2 0 4.7k\n";
    let delta1 = engine.sync_to_canvas(netlist1, &mut canvas).expect("sync failed");
    assert_eq!(delta1.added_count, 3);
    assert_eq!(delta1.updated_count, 0);
    assert_eq!(delta1.removed_count, 0);
    assert_eq!(canvas.components.len(), 3);
    assert_eq!(canvas.components[0].name, "V1");
    assert_eq!(canvas.components[1].name, "R1");
    assert_eq!(canvas.components[2].name, "R2");
    assert_eq!(canvas.components[1].value_str, "2.2k");
    assert_eq!(canvas.components[2].value_str, "4.7k");

    let orig_pos_r1 = canvas.components[1].pos;
    let orig_pos_r2 = canvas.components[2].pos;

    let netlist2 = "* Updated Parameters\nV1 net1 0 12.0\nR1 net1 net2 2.2k\nR2 net2 0 10k\n";
    let delta2 = engine.sync_to_canvas(netlist2, &mut canvas).expect("sync failed");
    assert_eq!(delta2.added_count, 0);
    assert_eq!(delta2.updated_count, 2);
    assert_eq!(delta2.removed_count, 0);
    assert_eq!(canvas.components[0].value_str, "12.0");
    assert_eq!(canvas.components[1].value_str, "2.2k");
    assert_eq!(canvas.components[2].value_str, "10k");
    assert_eq!(canvas.components[1].pos, orig_pos_r1, "R1 coordinates must be preserved");
    assert_eq!(canvas.components[2].pos, orig_pos_r2, "R2 coordinates must be preserved");

    // Remove R2 and add C1
    let netlist3 = "* Mod Delta\nV1 net1 0 12.0\nR1 net1 net2 2.2k\nC1 net2 0 100n\n";
    let delta3 = engine.sync_to_canvas(netlist3, &mut canvas).expect("sync failed");
    assert_eq!(delta3.added_count, 1);
    assert_eq!(delta3.updated_count, 0);
    assert_eq!(delta3.removed_count, 1);
    assert_eq!(canvas.components.len(), 3);
    assert!(canvas.components.iter().any(|c| c.name == "C1" && c.kind == ComponentKind::Capacitor));
    assert!(!canvas.components.iter().any(|c| c.name == "R2"));
}

#[test]
fn test_hashing_debounce_no_recompile_if_canvas_unchanged() {
    let mut engine = NetlistSyncEngine::new();
    let mut canvas = SchematicCanvas::new();
    let r1 = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(100.0, 100.0), 1);
    canvas.add_component(r1);

    let netlist_str1 = engine.sync_from_canvas(&canvas);
    let ptr1 = netlist_str1.as_ptr();

    let netlist_str2 = engine.sync_from_canvas(&canvas);
    let ptr2 = netlist_str2.as_ptr();

    assert_eq!(ptr1, ptr2, "Cached netlist string pointer must be reused without reallocation");
    assert!(engine.is_up_to_date(&canvas));

    // Mutate canvas component value to invalidate cache
    canvas.components[0].value_str = "4.7k".to_string();
    assert!(!engine.is_up_to_date(&canvas));
    let netlist_str3 = engine.sync_from_canvas(&canvas);
    assert!(netlist_str3.contains("4.7k"));
}

#[test]
fn test_netlist_sync_error_handling() {
    let mut engine = NetlistSyncEngine::new();
    let mut canvas = SchematicCanvas::new();

    // Invalid short line
    let invalid_line = "R1 1\n";
    let err = engine.sync_to_canvas(invalid_line, &mut canvas).unwrap_err();
    assert!(matches!(err, NetlistSyncError::InvalidComponent(_)));

    // Unknown designator prefix
    let unknown_line = "Z1 1 2 100\n";
    let err2 = engine.sync_to_canvas(unknown_line, &mut canvas).unwrap_err();
    assert!(matches!(err2, NetlistSyncError::UnknownComponentType(_)));
}

#[test]
fn test_throughput_benchmark_exceeds_50k_sync_operations_per_second() {
    let mut engine = NetlistSyncEngine::new();
    let mut canvas = SchematicCanvas::new();
    let r1 = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(100.0, 100.0), 1);
    let r2 = SchematicComponent::new(2, ComponentKind::Resistor, Pos2::new(200.0, 100.0), 2);
    canvas.add_component(r1);
    canvas.add_component(r2);

    // Warm-up cache
    engine.sync_from_canvas(&canvas);

    let iterations = 100_000;
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _s = engine.sync_from_canvas(&canvas);
    }
    let elapsed = start.elapsed();
    let throughput = iterations as f64 / elapsed.as_secs_f64();

    println!(
        "Netlist Sync Throughput: {:.2} ops/sec ({:?} for {} iterations)",
        throughput, elapsed, iterations
    );

    assert!(
        throughput > 50_000.0,
        "Netlist sync throughput must exceed 50,000 ops/sec, achieved {:.2}",
        throughput
    );
}
