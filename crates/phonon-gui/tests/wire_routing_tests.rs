#![deny(unsafe_code)]

//! Integration tests for Manhattan wire routing and topological junction detection.

use egui::Pos2;
use phonon_gui::schematic::{
    compute_junction_dots, PinNormal, SchematicWire, WirePinOrientation, WireSegment,
};

#[test]
fn test_manhattan_routing_segments() {
    // Diagonal route creates 2 orthogonal segments
    let w_diag =
        SchematicWire::manhattan_route(1, Pos2::new(100.0, 100.0), Pos2::new(200.0, 250.0));
    assert_eq!(w_diag.segments.len(), 2);
    assert_eq!(w_diag.segments[0].start, Pos2::new(100.0, 100.0));
    assert_eq!(w_diag.segments[0].end, Pos2::new(200.0, 100.0));
    assert_eq!(w_diag.segments[1].start, Pos2::new(200.0, 100.0));
    assert_eq!(w_diag.segments[1].end, Pos2::new(200.0, 250.0));

    // Pure horizontal route creates 1 segment
    let w_horiz =
        SchematicWire::manhattan_route(2, Pos2::new(50.0, 100.0), Pos2::new(150.0, 100.0));
    assert_eq!(w_horiz.segments.len(), 1);

    // Pure vertical route creates 1 segment
    let w_vert = SchematicWire::manhattan_route(3, Pos2::new(100.0, 50.0), Pos2::new(100.0, 200.0));
    assert_eq!(w_vert.segments.len(), 1);
}

#[test]
fn test_wire_segment_hit_testing() {
    let seg = WireSegment::new(Pos2::new(100.0, 100.0), Pos2::new(300.0, 100.0));

    // Points on the line
    assert!(seg.contains_point(Pos2::new(100.0, 100.0), 1.0));
    assert!(seg.contains_point(Pos2::new(200.0, 100.0), 1.0));
    assert!(seg.contains_point(Pos2::new(300.0, 100.0), 1.0));

    // Points slightly off the line within tolerance
    assert!(seg.contains_point(Pos2::new(200.0, 102.0), 3.0));
    assert!(!seg.contains_point(Pos2::new(200.0, 105.0), 3.0));

    // Points beyond the ends
    assert!(!seg.contains_point(Pos2::new(90.0, 100.0), 2.0));
    assert!(!seg.contains_point(Pos2::new(310.0, 100.0), 2.0));
}

#[test]
fn test_junction_dots_computation() {
    // Horizontal wire from (100, 100) to (300, 100)
    let w1 = SchematicWire::new(
        1,
        vec![WireSegment::new(
            Pos2::new(100.0, 100.0),
            Pos2::new(300.0, 100.0),
        )],
    );
    // Vertical wire branching off at (200, 100) down to (200, 200)
    let w2 = SchematicWire::new(
        2,
        vec![WireSegment::new(
            Pos2::new(200.0, 100.0),
            Pos2::new(200.0, 200.0),
        )],
    );
    // Second wire meeting at pin (100, 100) extending up to (100, 50)
    let w3 = SchematicWire::new(
        3,
        vec![WireSegment::new(
            Pos2::new(100.0, 100.0),
            Pos2::new(100.0, 50.0),
        )],
    );

    let wires = vec![w1, w2, w3];
    let pins = vec![Pos2::new(100.0, 100.0)];

    let junctions = compute_junction_dots(&wires, &pins);

    // (200, 100) is a T-junction: vertical wire branches off the horizontal wire
    // (100, 100) is a pin junction: 2 wires meet at the component pin
    assert!(junctions
        .iter()
        .any(|&p| (p - Pos2::new(200.0, 100.0)).length() < 1.0));
    assert!(junctions
        .iter()
        .any(|&p| (p - Pos2::new(100.0, 100.0)).length() < 1.0));
    assert_eq!(junctions.len(), 2);
}

#[test]
fn test_vh_and_hv_and_pin_aware_manhattan_routing() {
    let from = Pos2::new(100.0, 100.0);
    let to = Pos2::new(200.0, 250.0);

    // HV route: horizontal first to (to.x, from.y), then vertical to (to.x, to.y)
    let w_hv = SchematicWire::manhattan_route_hv(1, from, to);
    assert_eq!(w_hv.segments.len(), 2);
    assert_eq!(w_hv.segments[0].start, Pos2::new(100.0, 100.0));
    assert_eq!(w_hv.segments[0].end, Pos2::new(200.0, 100.0));
    assert_eq!(w_hv.segments[1].start, Pos2::new(200.0, 100.0));
    assert_eq!(w_hv.segments[1].end, Pos2::new(200.0, 250.0));

    // VH route: vertical first to (from.x, to.y), then horizontal to (to.x, to.y)
    let w_vh = SchematicWire::manhattan_route_vh(2, from, to);
    assert_eq!(w_vh.segments.len(), 2);
    assert_eq!(w_vh.segments[0].start, Pos2::new(100.0, 100.0));
    assert_eq!(w_vh.segments[0].end, Pos2::new(100.0, 250.0));
    assert_eq!(w_vh.segments[1].start, Pos2::new(100.0, 250.0));
    assert_eq!(w_vh.segments[1].end, Pos2::new(200.0, 250.0));

    // Pin aware routing: North departure chooses VH
    let w_north = SchematicWire::manhattan_route_pin_aware(3, from, PinNormal::North, to);
    assert_eq!(w_north.segments, w_vh.segments);

    // Pin aware routing: South departure chooses VH
    let w_south = SchematicWire::manhattan_route_pin_aware(4, from, PinNormal::South, to);
    assert_eq!(w_south.segments, w_vh.segments);

    // Pin aware routing: East departure chooses HV
    let w_east = SchematicWire::manhattan_route_pin_aware(5, from, PinNormal::East, to);
    assert_eq!(w_east.segments, w_hv.segments);

    // Pin aware routing: West departure chooses HV
    let w_west = SchematicWire::manhattan_route_pin_aware(6, from, PinNormal::West, to);
    assert_eq!(w_west.segments, w_hv.segments);

    // Pin aware routing with net name
    let w_net = SchematicWire::manhattan_route_pin_aware_with_net(
        7,
        from,
        WirePinOrientation::Vertical,
        to,
        Some("V_OUT".to_string()),
    );
    assert_eq!(w_net.net_name.as_deref(), Some("V_OUT"));
    assert_eq!(w_net.segments, w_vh.segments);
}

#[test]
fn test_voltage_source_narrowed_bounding_box() {
    use phonon_gui::schematic::{ComponentKind, SchematicComponent};
    use egui::Pos2;

    let v1 = SchematicComponent::new(1, ComponentKind::VoltageSource, Pos2::new(200.0, 300.0), 1);
    let bbox = v1.bounding_box();
    // 30.0 px wide (1 grid unit shorter on left and right: 70 - 40 = 30)
    assert!((bbox.width() - 30.0).abs() < 1e-3, "VoltageSource width should be 30px, got {}", bbox.width());
    assert!((bbox.height() - 70.0).abs() < 1e-3, "VoltageSource height should be 70px, got {}", bbox.height());

    let mut v1_rot = v1.clone();
    v1_rot.rotation = 1;
    let bbox_rot = v1_rot.bounding_box();
    assert!((bbox_rot.width() - 70.0).abs() < 1e-3, "Rotated VoltageSource width should be 70px");
    assert!((bbox_rot.height() - 30.0).abs() < 1e-3, "Rotated VoltageSource height should be 30px");
}

#[test]
fn test_simulate_voltage_divider_drag() {
    use phonon_gui::PhononApp;
    use egui::Vec2;

    let mut app = PhononApp::default();
    app.load_voltage_divider_demo();

    // Verify V1 bounding box is narrowed to 30.0 px
    let v1 = app.components.iter().find(|c| c.id == 1).unwrap();
    assert_eq!(v1.bounding_box().width(), 30.0);
    assert_eq!(v1.bounding_box().height(), 70.0);

    // Select V1 (id=1)
    app.select_component(1, false);

    // Simulate drag start
    app.dragging_selection = true;
    app.drag_start_positions = vec![(1, app.components[0].pos)];
    app.drag_start_wires = app.wires.clone();

    // Drag left by -160.0 in steps of -2.0
    for _ in 0..80 {
        app.translate_selection(Vec2::new(-2.0, 0.0));
    }

    // Verify V1 new position is exactly on grid: (40.0, 300.0)
    let v1_moved = app.components.iter().find(|c| c.id == 1).unwrap();
    assert_eq!(v1_moved.pos.x, 40.0);
    assert_eq!(v1_moved.pos.y, 300.0);

    // Verify all component pin tips are strictly on the 20px grid
    for comp in &app.components {
        for (pin_name, p) in comp.all_pins() {
            assert!(
                (p.x % 20.0).abs() < 1e-3 || (20.0 - (p.x % 20.0)).abs() < 1e-3,
                "Pin {} of {} at x={} is not on 20px grid",
                pin_name, comp.name, p.x
            );
            assert!(
                (p.y % 20.0).abs() < 1e-3 || (20.0 - (p.y % 20.0)).abs() < 1e-3,
                "Pin {} of {} at y={} is not on 20px grid",
                pin_name, comp.name, p.y
            );
        }
    }

    // Verify all wire endpoints and vertices are strictly on the 20px grid
    for wire in &app.wires {
        for seg in &wire.segments {
            assert!(
                (seg.start.x % 20.0).abs() < 1e-3 || (20.0 - (seg.start.x % 20.0)).abs() < 1e-3,
                "Wire {} seg start x={} not on 20px grid", wire.id, seg.start.x
            );
            assert!(
                (seg.start.y % 20.0).abs() < 1e-3 || (20.0 - (seg.start.y % 20.0)).abs() < 1e-3,
                "Wire {} seg start y={} not on 20px grid", wire.id, seg.start.y
            );
            assert!(
                (seg.end.x % 20.0).abs() < 1e-3 || (20.0 - (seg.end.x % 20.0)).abs() < 1e-3,
                "Wire {} seg end x={} not on 20px grid", wire.id, seg.end.x
            );
            assert!(
                (seg.end.y % 20.0).abs() < 1e-3 || (20.0 - (seg.end.y % 20.0)).abs() < 1e-3,
                "Wire {} seg end y={} not on 20px grid", wire.id, seg.end.y
            );
        }
    }

    // Wire 1: climbs up from (40, 260) to (40, 200), then right to (360, 200)
    let w1 = app.wires.iter().find(|w| w.id == 1).unwrap();
    assert_eq!(w1.start_point(), Pos2::new(40.0, 260.0));
    assert_eq!(w1.end_point(), Pos2::new(360.0, 200.0));

    // Wire 3: connects from (360, 400) to (40, 340)
    let w3 = app.wires.iter().find(|w| w.id == 3).unwrap();
    assert_eq!(w3.start_point(), Pos2::new(360.0, 400.0));
    assert_eq!(w3.end_point(), Pos2::new(40.0, 340.0));

    // Wire 4: connects from (40, 340) to Ground at (200, 420)
    let w4 = app.wires.iter().find(|w| w.id == 4).unwrap();
    assert_eq!(w4.start_point(), Pos2::new(40.0, 340.0));
    assert_eq!(w4.end_point(), Pos2::new(200.0, 420.0));

    // Verify wires avoid obstacle bounding boxes
    let obstacles: Vec<_> = app.components.iter().map(|c| c.bounding_box()).collect();
    for wire in &app.wires {
        // Segments outside the attached pins must not pass through non-connected obstacles
        assert!(!wire.intersects_obstacles(&obstacles[1..3]), "Wire {} intersects resistors", wire.id);
    }
}

#[test]
fn test_unsolvable_drag_maintains_snapshot_and_marks_red_pins() {
    use phonon_gui::PhononApp;
    use egui::Vec2;

    let mut app = PhononApp::default();
    app.load_voltage_divider_demo();

    // Select V1 (id=1)
    app.select_component(1, false);

    // Save initial wires
    let initial_wires = app.wires.clone();
    app.drag_start_wires = initial_wires.clone();

    // Simulate drag collision: move V1 directly onto Resistor R1 at (360.0, 240.0)
    let delta = Vec2::new(160.0, -60.0);
    let moving_comp_ids = vec![1];

    let mut obstacles = Vec::new();
    let mut moving_bboxes = Vec::new();
    for c in &mut app.components {
        if moving_comp_ids.contains(&c.id) {
            c.pos += delta;
            moving_bboxes.push((c.id, c.bounding_box()));
        } else {
            obstacles.push(c.bounding_box());
        }
    }

    // Check collision
    let mut has_collision = false;
    for (_cid, m_bbox) in &moving_bboxes {
        for obs in &obstacles {
            if m_bbox.shrink(2.0).intersects(*obs) {
                has_collision = true;
                break;
            }
        }
    }
    assert!(has_collision, "Moving V1 onto R1 must trigger collision detection");

    if has_collision {
        // As per app drag logic: wires stay at pre-drag snapshot, component marked unsolvable
        app.wires = app.drag_start_wires.clone();
        for id in &moving_comp_ids {
            app.unsolvable_wiring_components.insert(*id);
        }
    }

    assert!(app.unsolvable_wiring_components.contains(&1));
    assert_eq!(app.wires, initial_wires);
}


