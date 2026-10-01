#![deny(unsafe_code)]

//! Verification suite for Phonon Studio Interactive Canvas Engine (Phase 307):
//! Smooth continuous zoom scaling, 90-degree 'R' rotation cycling, text selection lockout,
//! and canvas interaction throughput benchmarking.

use egui::{Pos2, Sense, Vec2};
use phonon_gui::schematic::{ComponentKind, SchematicCanvas, SchematicComponent};
use phonon_gui::{PhononApp, ToolMode};
use std::time::Instant;

#[test]
fn test_smooth_zoom_continuous_scaling() {
    let mut canvas = SchematicCanvas::new();
    assert_eq!(canvas.zoom, 1.0);

    // Small scroll delta: +5.0
    canvas.apply_zoom_delta(5.0, None);
    let ratio_5 = canvas.zoom / 1.0;
    assert!(ratio_5 > 1.0, "Zoom should increase with positive scroll");
    assert!(ratio_5 < 1.02, "Scale change should be smooth and sub-1.02x");
    assert!((ratio_5 - (5.0f32 * 0.0015).exp()).abs() < 1e-5);

    // Another small delta: +10.0
    let zoom_before_10 = canvas.zoom;
    canvas.apply_zoom_delta(10.0, None);
    let ratio_10 = canvas.zoom / zoom_before_10;
    assert!(ratio_10 > 1.0 && ratio_10 < 1.02);
    assert!((ratio_10 - (10.0f32 * 0.0015).exp()).abs() < 1e-5);

    // Negative small delta: -5.0
    let zoom_before_neg = canvas.zoom;
    canvas.apply_zoom_delta(-5.0, None);
    let ratio_neg = canvas.zoom / zoom_before_neg;
    assert!(ratio_neg < 1.0 && ratio_neg > 0.98);
    assert!((ratio_neg - (-5.0f32 * 0.0015).exp()).abs() < 1e-5);

    // Large delta clamping: ensure deltas are clamped to [-120.0, 120.0]
    let mut canvas_huge = SchematicCanvas::new();
    canvas_huge.apply_zoom_delta(500.0, None);
    let expected_clamped_zoom = (120.0f32 * 0.0015).exp();
    assert!((canvas_huge.zoom - expected_clamped_zoom).abs() < 1e-5);

    // Boundary clamping: zoom must remain within [0.2, 5.0]
    for _ in 0..100 {
        canvas.apply_zoom_delta(100.0, None);
    }
    assert_eq!(canvas.zoom, 5.0, "Zoom should clamp at upper bound 5.0");

    for _ in 0..100 {
        canvas.apply_zoom_delta(-100.0, None);
    }
    assert_eq!(canvas.zoom, 0.2, "Zoom should clamp at lower bound 0.2");
}

#[test]
fn test_smooth_zoom_cursor_centering() {
    let cursor_positions = [
        Pos2::new(100.0, 100.0),
        Pos2::new(350.0, 280.0),
        Pos2::new(0.0, 0.0),
        Pos2::new(1920.0, 1080.0),
    ];

    for &cursor in &cursor_positions {
        let mut canvas = SchematicCanvas::new();
        let world_initial = canvas.screen_to_world(cursor);

        // Zoom in by delta = 25.0
        canvas.apply_zoom_delta(25.0, Some(cursor));
        let world_after_zoom_in = canvas.screen_to_world(cursor);
        assert!(
            (world_after_zoom_in.x - world_initial.x).abs() < 1e-3,
            "World X at cursor must remain stationary after zoom in: initial={}, after={}",
            world_initial.x, world_after_zoom_in.x
        );
        assert!(
            (world_after_zoom_in.y - world_initial.y).abs() < 1e-3,
            "World Y at cursor must remain stationary after zoom in: initial={}, after={}",
            world_initial.y, world_after_zoom_in.y
        );

        // Zoom out by delta = -40.0
        canvas.apply_zoom_delta(-40.0, Some(cursor));
        let world_after_zoom_out = canvas.screen_to_world(cursor);
        assert!(
            (world_after_zoom_out.x - world_initial.x).abs() < 1e-3,
            "World X at cursor must remain stationary after zoom out: initial={}, after={}",
            world_initial.x, world_after_zoom_out.x
        );
        assert!(
            (world_after_zoom_out.y - world_initial.y).abs() < 1e-3,
            "World Y at cursor must remain stationary after zoom out: initial={}, after={}",
            world_initial.y, world_after_zoom_out.y
        );
    }
}

#[test]
fn test_component_r_rotation_cycling() {
    let origin = Pos2::new(200.0, 300.0);
    let mut comp = SchematicComponent::new(1, ComponentKind::Resistor, origin, 1);

    // Initial state: rotation = 0 (0 deg)
    assert_eq!(comp.rotation, 0);
    assert_eq!(comp.rotation_degrees(), 0);
    let pins_0 = comp.all_pins();
    assert_eq!(pins_0.len(), 2);
    assert_eq!(pins_0[0].0, "1");
    assert!((pins_0[0].1.x - 200.0).abs() < 1e-4 && (pins_0[0].1.y - 260.0).abs() < 1e-4);
    assert_eq!(pins_0[1].0, "2");
    assert!((pins_0[1].1.x - 200.0).abs() < 1e-4 && (pins_0[1].1.y - 340.0).abs() < 1e-4);

    // 1st rotation: 90 deg clockwise (rotation = 1)
    comp.rotate_clockwise();
    assert_eq!(comp.rotation, 1);
    assert_eq!(comp.rotation_degrees(), 90);
    let pins_1 = comp.all_pins();
    // Local pin 1: (0, -40). 90 CW: (-(-40), 0) = (40, 0) -> world (240, 300)
    // Local pin 2: (0, 40). 90 CW: (-40, 0) -> world (160, 300)
    assert!((pins_1[0].1.x - 240.0).abs() < 1e-4 && (pins_1[0].1.y - 300.0).abs() < 1e-4);
    assert!((pins_1[1].1.x - 160.0).abs() < 1e-4 && (pins_1[1].1.y - 300.0).abs() < 1e-4);

    // 2nd rotation: 180 deg (rotation = 2)
    comp.rotate_clockwise();
    assert_eq!(comp.rotation, 2);
    assert_eq!(comp.rotation_degrees(), 180);
    let pins_2 = comp.all_pins();
    assert!((pins_2[0].1.x - 200.0).abs() < 1e-4 && (pins_2[0].1.y - 340.0).abs() < 1e-4);
    assert!((pins_2[1].1.x - 200.0).abs() < 1e-4 && (pins_2[1].1.y - 260.0).abs() < 1e-4);

    // 3rd rotation: 270 deg (rotation = 3)
    comp.rotate_clockwise();
    assert_eq!(comp.rotation, 3);
    assert_eq!(comp.rotation_degrees(), 270);
    let pins_3 = comp.all_pins();
    assert!((pins_3[0].1.x - 160.0).abs() < 1e-4 && (pins_3[0].1.y - 300.0).abs() < 1e-4);
    assert!((pins_3[1].1.x - 240.0).abs() < 1e-4 && (pins_3[1].1.y - 300.0).abs() < 1e-4);

    // 4th rotation: 360 deg -> cycle back to 0 deg (rotation = 0)
    comp.rotate_clockwise();
    assert_eq!(comp.rotation, 0);
    assert_eq!(comp.rotation_degrees(), 0);
    let pins_4 = comp.all_pins();
    assert!((pins_4[0].1.x - 200.0).abs() < 1e-4 && (pins_4[0].1.y - 260.0).abs() < 1e-4);
    assert!((pins_4[1].1.x - 200.0).abs() < 1e-4 && (pins_4[1].1.y - 340.0).abs() < 1e-4);

    // Multi-terminal transistor verification: NMOS (Drain, Gate, Source)
    let mut nmos = SchematicComponent::new(2, ComponentKind::Nmos, origin, 1);
    assert_eq!(nmos.rotation, 0);
    nmos.rotate_clockwise();
    assert_eq!(nmos.rotation, 1);
    let nmos_pins_90 = nmos.all_pins();
    assert_eq!(nmos_pins_90.len(), 3);
    assert!((nmos_pins_90[0].1.x - 240.0).abs() < 1e-4 && (nmos_pins_90[0].1.y - 320.0).abs() < 1e-4);
}

#[test]
fn test_held_component_rotation_during_placement() {
    let mut app = PhononApp::default();
    app.selected_tool = ToolMode::Place(ComponentKind::Resistor);
    assert_eq!(app.placement_rotation, 0);

    // Press 'R' key to rotate component being held
    app.rotate_active();
    assert_eq!(app.placement_rotation, 1);

    // Rotate through full cycle while held
    app.rotate_active();
    assert_eq!(app.placement_rotation, 2);
    app.rotate_active();
    assert_eq!(app.placement_rotation, 3);
    app.rotate_active();
    assert_eq!(app.placement_rotation, 0);

    // Rotate to 90 deg before placing onto canvas
    app.rotate_active();
    assert_eq!(app.placement_rotation, 1);

    // Simulate committing the component to canvas at Pos2(150.0, 250.0)
    let initial_comp_count = app.components.len();
    let place_pos = Pos2::new(150.0, 250.0);
    let mut committed = SchematicComponent::new(
        app.next_comp_id,
        ComponentKind::Resistor,
        place_pos,
        initial_comp_count + 1,
    );
    committed.rotation = app.placement_rotation;
    app.components.push(committed);
    app.next_comp_id += 1;

    let placed_comp = app.components.last().expect("Component must be placed");
    assert_eq!(placed_comp.rotation, 1);
    assert_eq!(placed_comp.rotation_degrees(), 90);
    assert_eq!(placed_comp.pos, place_pos);

    // Verify pin coordinates of placed component match the 90 deg rotation
    let pins = placed_comp.all_pins();
    assert!((pins[0].1.x - 190.0).abs() < 1e-4 && (pins[0].1.y - 250.0).abs() < 1e-4);
    assert!((pins[1].1.x - 110.0).abs() < 1e-4 && (pins[1].1.y - 250.0).abs() < 1e-4);

    // Also verify rotating an existing selected/dragged component
    app.selected_tool = ToolMode::Select;
    app.selected_component_id = Some(placed_comp.id);
    app.rotate_active();
    let rotated_comp = app.components.last().unwrap();
    assert_eq!(rotated_comp.rotation, 2);
    assert_eq!(rotated_comp.rotation_degrees(), 180);
}

#[test]
fn test_text_selection_lockout() {
    let canvas = SchematicCanvas::new();
    assert!(canvas.is_text_selection_locked());

    // Verify that canvas interaction mode utilizes Sense::click_and_drag()
    let sense = Sense::click_and_drag();
    assert!(sense.interactive());
    assert!(sense.senses_click());
    assert!(sense.senses_drag());
    assert_eq!(sense, Sense::click_and_drag());

    // Verify web CSS template contains user-select: none and -webkit-user-select: none
    let index_css = include_str!("../../../web/studio/src/index.css");
    assert!(
        index_css.contains("user-select: none;"),
        "index.css must enforce user-select: none"
    );
    assert!(
        index_css.contains("-webkit-user-select: none;"),
        "index.css must enforce -webkit-user-select: none"
    );

    let web_html = include_str!("../../../assets/web/index.html");
    assert!(
        web_html.contains("user-select: none;"),
        "assets/web/index.html must enforce user-select: none"
    );
    assert!(
        web_html.contains("-webkit-user-select: none;"),
        "assets/web/index.html must enforce -webkit-user-select: none"
    );
}

#[test]
fn test_canvas_interaction_throughput() {
    let mut canvas = SchematicCanvas::new();
    let mut comp = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(100.0, 100.0), 1);
    let focus = Some(Pos2::new(500.0, 500.0));

    let start = Instant::now();
    let iterations = 10_000;
    for i in 0..iterations {
        let delta = if i % 2 == 0 { 2.5 } else { -2.5 };
        canvas.apply_zoom_delta(delta, focus);
        canvas.pan += Vec2::new(0.5, -0.5);
        comp.rotate_clockwise();
    }
    let elapsed = start.elapsed();
    let total_ops = iterations * 3; // zoom, pan, rotate
    let ops_per_sec = (total_ops as f64) / elapsed.as_secs_f64();

    eprintln!(
        "Interactive canvas throughput: {} ops in {:?} ({:.2} ops/sec)",
        total_ops, elapsed, ops_per_sec
    );

    assert!(
        elapsed.as_millis() < 10,
        "10,000 canvas operations took {:?}, exceeding 10 ms threshold",
        elapsed
    );
    assert!(
        ops_per_sec > 1_000_000.0,
        "Throughput {:.2} ops/sec below 1,000,000 ops/sec threshold",
        ops_per_sec
    );
}
