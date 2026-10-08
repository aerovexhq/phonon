#![deny(unsafe_code)]

//! Test suite for empty canvas origin normalization.
//!
//! Validates:
//! 1. `SchematicCanvas::is_empty()` query across components, wires, subcircuits, and buses.
//! 2. `SchematicCanvas::reset_origin_at_screen()` coordinate transformation accuracy.
//! 3. Placing the first component on an empty canvas sets its world coordinates to (0.0, 0.0)
//!    and aligns the canvas pan so that (0, 0) is rendered at the exact click position.
//! 4. Subsequent component placements preserve the established origin.
//! 5. Starting a wire on an empty canvas establishes (0.0, 0.0) as the wire start and origin.
//! 6. Clearing the canvas re-enables origin normalization for future placements.

use egui::{pos2, vec2, Context, PointerButton, Pos2, Rect};
use phonon_gui::app::{PhononApp, ToolMode};
use phonon_gui::schematic::{ComponentKind, SchematicCanvas, SchematicComponent, SchematicWire};

#[test]
fn test_schematic_canvas_is_empty_and_reset_origin() {
    let mut canvas = SchematicCanvas::new();
    assert!(canvas.is_empty(), "Fresh canvas must be empty");

    // Add component
    let comp = SchematicComponent::new(1, ComponentKind::Resistor, pos2(100.0, 100.0), 1);
    canvas.add_component(comp);
    assert!(!canvas.is_empty(), "Canvas with component is not empty");

    canvas.clear();
    assert!(canvas.is_empty(), "Canvas must be empty after clear()");

    // Add wire
    let wire = SchematicWire::manhattan_route(1, pos2(0.0, 0.0), pos2(100.0, 0.0));
    canvas.add_wire(wire);
    assert!(!canvas.is_empty(), "Canvas with wire is not empty");

    canvas.clear();
    assert!(canvas.is_empty());

    // Test reset_origin_at_screen
    canvas.pan = vec2(-1200.0, 850.0);
    canvas.zoom = 1.0;
    let target_screen = pos2(640.0, 480.0);

    canvas.reset_origin_at_screen(target_screen);
    assert_eq!(canvas.pan, vec2(640.0, 480.0));
    assert_eq!(canvas.world_to_screen(Pos2::ZERO), target_screen);
    assert_eq!(canvas.screen_to_world(target_screen), Pos2::ZERO);
}

#[test]
fn test_app_is_canvas_empty_state() {
    let mut app = PhononApp::default();
    app.clear_canvas_state();
    assert!(app.is_canvas_empty(), "App canvas must be empty after clear_canvas_state");

    let comp = SchematicComponent::new(1, ComponentKind::Capacitor, pos2(0.0, 0.0), 1);
    app.components.push(comp);
    assert!(!app.is_canvas_empty());

    app.clear_canvas_state();
    assert!(app.is_canvas_empty());
}

fn simulate_click(ctx: &Context, app: &mut PhononApp, viewport: Rect, pos: Pos2) {
    let base_time = ctx.input(|i| i.time);
    let t0 = base_time + 0.05;
    let t1 = base_time + 0.10;
    let t2 = base_time + 0.15;

    // Frame 0: Warmup pointer hover so egui registers widget under cursor
    let mut input0 = egui::RawInput::default();
    input0.screen_rect = Some(viewport);
    input0.time = Some(t0);
    input0.events.push(egui::Event::PointerMoved(pos));
    let mut output0 = ctx.run_ui(input0, |ui| {
        egui::CentralPanel::default().show(ui, |ui| {
            app.render_canvas(ui);
        });
    });
    output0.textures_delta.clear();

    // Frame 1: Press pointer
    let mut input1 = egui::RawInput::default();
    input1.screen_rect = Some(viewport);
    input1.time = Some(t1);
    input1.events.push(egui::Event::PointerMoved(pos));
    input1.events.push(egui::Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Default::default(),
    });
    let mut output1 = ctx.run_ui(input1, |ui| {
        egui::CentralPanel::default().show(ui, |ui| {
            app.render_canvas(ui);
        });
    });
    output1.textures_delta.clear();

    // Frame 2: Release pointer (triggers click)
    let mut input2 = egui::RawInput::default();
    input2.screen_rect = Some(viewport);
    input2.time = Some(t2);
    input2.events.push(egui::Event::PointerMoved(pos));
    input2.events.push(egui::Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Default::default(),
    });
    let mut output2 = ctx.run_ui(input2, |ui| {
        egui::CentralPanel::default().show(ui, |ui| {
            app.render_canvas(ui);
        });
    });
    output2.textures_delta.clear();
}

#[test]
fn test_empty_canvas_places_first_component_at_origin() {
    let ctx = Context::default();
    let viewport = Rect::from_min_size(pos2(0.0, 0.0), vec2(1280.0, 800.0));

    let mut app = PhononApp::default();
    app.clear_canvas_state();
    assert!(app.is_canvas_empty());

    // Simulate panning far away before placing anything
    app.canvas.pan = vec2(-2500.0, 1800.0);
    app.canvas.zoom = 1.0;

    // Select Resistor tool
    app.selected_tool = ToolMode::PlaceComponent(ComponentKind::Resistor);

    // Target click screen position inside canvas
    let click_screen = pos2(500.0, 400.0);
    simulate_click(&ctx, &mut app, viewport, click_screen);

    // Verify component was placed
    assert_eq!(app.components.len(), 1, "Exactly one component must be placed");
    let placed = &app.components[0];

    // Must be placed at world origin (0.0, 0.0)
    assert_eq!(
        placed.pos,
        Pos2::ZERO,
        "First component on empty canvas must be at world origin (0.0, 0.0)"
    );

    // Canvas pan must now align world origin (0.0, 0.0) with the click screen position
    let rendered_screen = app.canvas.world_to_screen(placed.pos);
    assert!(
        (rendered_screen.x - click_screen.x).abs() <= 20.0,
        "Rendered screen x ({}) must be near click x ({})",
        rendered_screen.x,
        click_screen.x
    );
    assert!(
        (rendered_screen.y - click_screen.y).abs() <= 20.0,
        "Rendered screen y ({}) must be near click y ({})",
        rendered_screen.y,
        click_screen.y
    );

    // Canvas is no longer empty
    assert!(!app.is_canvas_empty());
}

#[test]
fn test_second_component_placement_preserves_origin() {
    let ctx = Context::default();
    let viewport = Rect::from_min_size(pos2(0.0, 0.0), vec2(1280.0, 800.0));

    let mut app = PhononApp::default();
    app.clear_canvas_state();

    // Select Ground tool
    app.selected_tool = ToolMode::PlaceComponent(ComponentKind::Ground);

    let first_click = pos2(400.0, 300.0);
    simulate_click(&ctx, &mut app, viewport, first_click);

    assert_eq!(app.components.len(), 1);
    assert_eq!(app.components[0].pos, Pos2::ZERO);

    // Pan recorded after first placement
    let pan_after_first = app.canvas.pan;

    // 2. Place second component at an offset screen position (+80px right)
    let second_click = first_click + vec2(80.0, 0.0);
    simulate_click(&ctx, &mut app, viewport, second_click);

    assert_eq!(app.components.len(), 2);
    // Second component should NOT be at (0, 0)
    let second_comp = &app.components[1];
    assert_ne!(
        second_comp.pos,
        Pos2::ZERO,
        "Second component must not be forced to origin"
    );

    // Canvas pan must not have been reset on the second placement
    assert_eq!(
        app.canvas.pan, pan_after_first,
        "Canvas pan must remain stable after first placement"
    );

    // Relative offset should be ~80 units in world space
    assert!(
        (second_comp.pos.x - 80.0).abs() <= 5.0,
        "Second component should be at x ~ 80.0 relative to origin, got {}",
        second_comp.pos.x
    );
}

#[test]
fn test_clear_canvas_re_enables_origin_reset() {
    let ctx = Context::default();
    let viewport = Rect::from_min_size(pos2(0.0, 0.0), vec2(1280.0, 800.0));

    let mut app = PhononApp::default();

    // Place a dummy component
    app.components.push(SchematicComponent::new(1, ComponentKind::Resistor, pos2(400.0, 500.0), 1));
    assert!(!app.is_canvas_empty());

    // Clear canvas
    app.clear_canvas_state();
    assert!(app.is_canvas_empty());

    // Now place new component at a fresh position
    app.selected_tool = ToolMode::PlaceComponent(ComponentKind::VoltageSource);
    let click_pos = pos2(300.0, 200.0);
    simulate_click(&ctx, &mut app, viewport, click_pos);

    assert_eq!(app.components.len(), 1);
    assert_eq!(app.components[0].pos, Pos2::ZERO);
}

#[test]
fn test_empty_canvas_starts_wire_at_origin() {
    let ctx = Context::default();
    let viewport = Rect::from_min_size(pos2(0.0, 0.0), vec2(1280.0, 800.0));

    let mut app = PhononApp::default();
    app.clear_canvas_state();
    assert!(app.is_canvas_empty());

    // Pan canvas far away
    app.canvas.pan = vec2(-1500.0, 1200.0);
    app.selected_tool = ToolMode::Wire;

    let click_pos = pos2(600.0, 450.0);
    simulate_click(&ctx, &mut app, viewport, click_pos);

    // Active wire start must now be at world origin Pos2::ZERO
    assert_eq!(
        app.active_wire_start,
        Some(Pos2::ZERO),
        "Starting wire on empty canvas must set active_wire_start to Pos2::ZERO"
    );

    // The origin should now map to the click screen position
    let rendered_screen = app.canvas.world_to_screen(Pos2::ZERO);
    assert!(
        (rendered_screen.x - click_pos.x).abs() <= 20.0,
        "Origin x ({}) must be near click x ({})",
        rendered_screen.x,
        click_pos.x
    );
    assert!(
        (rendered_screen.y - click_pos.y).abs() <= 20.0,
        "Origin y ({}) must be near click y ({})",
        rendered_screen.y,
        click_pos.y
    );
}

#[test]
fn test_empty_canvas_starts_bus_at_origin() {
    let ctx = Context::default();
    let viewport = Rect::from_min_size(pos2(0.0, 0.0), vec2(1280.0, 800.0));

    let mut app = PhononApp::default();
    app.clear_canvas_state();
    assert!(app.is_canvas_empty());

    // Pan canvas far away
    app.canvas.pan = vec2(-3000.0, 2500.0);
    app.selected_tool = ToolMode::Bus;

    let click_pos = pos2(700.0, 350.0);
    simulate_click(&ctx, &mut app, viewport, click_pos);

    // Active wire/bus start must be at Pos2::ZERO
    assert_eq!(
        app.active_wire_start,
        Some(Pos2::ZERO),
        "Starting bus on empty canvas must set active_wire_start to Pos2::ZERO"
    );

    // Origin maps to click screen position
    let rendered_screen = app.canvas.world_to_screen(Pos2::ZERO);
    assert!(
        (rendered_screen.x - click_pos.x).abs() <= 20.0,
        "Origin x ({}) must be near click x ({})",
        rendered_screen.x,
        click_pos.x
    );
    assert!(
        (rendered_screen.y - click_pos.y).abs() <= 20.0,
        "Origin y ({}) must be near click y ({})",
        rendered_screen.y,
        click_pos.y
    );
}
