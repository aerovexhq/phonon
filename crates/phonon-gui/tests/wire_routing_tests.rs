//! Integration tests for Manhattan wire routing and topological junction detection.

use egui::Pos2;
use phonon_gui::schematic::{compute_junction_dots, SchematicWire, WireSegment};

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
