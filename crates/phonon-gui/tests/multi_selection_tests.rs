#![deny(unsafe_code)]

//! Comprehensive test suite for Phase 351: Multi-Item Selection, Marquee Region Drag,
//! Group Operations (Translate, Delete, Duplicate, Rotate), Floating CAD Island, and Action Registry.

use egui::{Pos2, Rect, Vec2};
use phonon_gui::actions::{fuzzy_match_score, ActionCategory, ActionId, ActionRegistry};
use phonon_gui::schematic::{
    CanvasCommand, ComponentKind, SchematicCanvas, SchematicComponent, SchematicWire, WireSegment,
};
use phonon_gui::widgets::command_palette::CommandPalette;
use phonon_gui::widgets::floating_toolbar::FloatingToolbarState;
use phonon_gui::{PhononApp, ToolMode};

#[test]
fn test_shift_click_multi_selection_toggle_components_and_wires() {
    let mut canvas = SchematicCanvas::new();

    let c1 = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(100.0, 100.0), 1);
    let c2 = SchematicComponent::new(2, ComponentKind::Capacitor, Pos2::new(200.0, 100.0), 1);
    let c3 = SchematicComponent::new(3, ComponentKind::Inductor, Pos2::new(300.0, 100.0), 1);
    canvas.components = vec![c1, c2, c3];

    let w1 = SchematicWire::manhattan_route(10, Pos2::new(100.0, 80.0), Pos2::new(200.0, 80.0));
    let w2 = SchematicWire::manhattan_route(20, Pos2::new(200.0, 80.0), Pos2::new(300.0, 80.0));
    canvas.wires = vec![w1, w2];

    assert!(canvas.selected_component_ids.is_empty());
    assert!(canvas.selected_wire_ids.is_empty());
    assert_eq!(canvas.selected_component_id, None);
    assert_eq!(canvas.selected_wire_id, None);

    // 1. Shift+Click toggle component 1
    canvas.toggle_component_selection(1);
    assert!(canvas.is_component_selected(1));
    assert_eq!(canvas.selected_component_ids.len(), 1);
    assert_eq!(canvas.selected_component_id, Some(1));

    // 2. Shift+Click toggle component 2 (both 1 and 2 selected)
    canvas.toggle_component_selection(2);
    assert!(canvas.is_component_selected(1));
    assert!(canvas.is_component_selected(2));
    assert_eq!(canvas.selected_component_ids.len(), 2);

    // 3. Shift+Click toggle component 1 again (deselects 1, keeps 2)
    canvas.toggle_component_selection(1);
    assert!(!canvas.is_component_selected(1));
    assert!(canvas.is_component_selected(2));
    assert_eq!(canvas.selected_component_ids.len(), 1);
    assert_eq!(canvas.selected_component_id, Some(2));

    // 4. Shift+Click toggle wire 10
    canvas.toggle_wire_selection(10);
    assert!(canvas.is_wire_selected(10));
    assert_eq!(canvas.selected_wire_ids.len(), 1);
    assert_eq!(canvas.selected_wire_id, Some(10));

    // 5. Shift+Click toggle wire 20
    canvas.toggle_wire_selection(20);
    assert!(canvas.is_wire_selected(10));
    assert!(canvas.is_wire_selected(20));
    assert_eq!(canvas.selected_wire_ids.len(), 2);

    // 6. Clear selection
    canvas.clear_selection();
    assert!(canvas.selected_component_ids.is_empty());
    assert!(canvas.selected_wire_ids.is_empty());
    assert_eq!(canvas.selected_component_id, None);
    assert_eq!(canvas.selected_wire_id, None);
}

#[test]
fn test_marquee_rectangle_intersection_geometry() {
    let mut canvas = SchematicCanvas::new();

    // Component centers: c1 at (100, 100), c2 at (250, 100), c3 at (500, 500)
    // Component bounding box is 70x70 centered on pos:
    // c1 bounds: [65..135, 65..135]
    // c2 bounds: [215..285, 65..135]
    // c3 bounds: [465..535, 465..535]
    let c1 = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(100.0, 100.0), 1);
    let c2 = SchematicComponent::new(2, ComponentKind::Capacitor, Pos2::new(250.0, 100.0), 1);
    let c3 = SchematicComponent::new(3, ComponentKind::Inductor, Pos2::new(500.0, 500.0), 1);
    canvas.components = vec![c1, c2, c3];

    // Wire 10 connects (80, 100) to (270, 100)
    // Wire 20 connects (600, 600) to (700, 600)
    let w1 = SchematicWire::manhattan_route(10, Pos2::new(80.0, 100.0), Pos2::new(270.0, 100.0));
    let w2 = SchematicWire::manhattan_route(20, Pos2::new(600.0, 600.0), Pos2::new(700.0, 600.0));
    canvas.wires = vec![w1, w2];

    // Rect enclosing c1 and crossing w1, but excluding c2, c3, w2
    let marquee_rect1 = Rect::from_min_max(Pos2::new(50.0, 50.0), Pos2::new(150.0, 150.0));

    // Test component intersection directly
    assert!(canvas.components[0].intersects_rect(&marquee_rect1));
    assert!(!canvas.components[1].intersects_rect(&marquee_rect1));
    assert!(!canvas.components[2].intersects_rect(&marquee_rect1));

    // Test wire intersection directly
    assert!(canvas.wires[0].intersects_rect(&marquee_rect1));
    assert!(!canvas.wires[1].intersects_rect(&marquee_rect1));

    // Select with add = false
    canvas.select_in_rect(marquee_rect1, false);
    assert!(canvas.is_component_selected(1));
    assert!(!canvas.is_component_selected(2));
    assert!(!canvas.is_component_selected(3));
    assert!(canvas.is_wire_selected(10));
    assert!(!canvas.is_wire_selected(20));

    // Marquee rect enclosing c2 only: [200..300, 50..150]
    let marquee_rect2 = Rect::from_min_max(Pos2::new(200.0, 50.0), Pos2::new(300.0, 150.0));

    // Select with add = true (union)
    canvas.select_in_rect(marquee_rect2, true);
    assert!(canvas.is_component_selected(1));
    assert!(canvas.is_component_selected(2));
    assert!(!canvas.is_component_selected(3));

    // Select all
    canvas.select_all();
    assert_eq!(canvas.selected_component_ids.len(), 3);
    assert_eq!(canvas.selected_wire_ids.len(), 2);
}

#[test]
fn test_wire_segment_intersection_crossing() {
    // Segment from (50, 50) to (250, 50) crossing a rect [100..200, 0..100]
    let seg_crossing = WireSegment::new(Pos2::new(50.0, 50.0), Pos2::new(250.0, 50.0));
    let rect = Rect::from_min_max(Pos2::new(100.0, 0.0), Pos2::new(200.0, 100.0));
    assert!(seg_crossing.intersects_rect(&rect));

    // Segment completely outside: from (300, 50) to (400, 50)
    let seg_outside = WireSegment::new(Pos2::new(300.0, 50.0), Pos2::new(400.0, 50.0));
    assert!(!seg_outside.intersects_rect(&rect));

    // Segment with endpoint inside: from (150, 50) to (300, 50)
    let seg_endpoint = WireSegment::new(Pos2::new(150.0, 50.0), Pos2::new(300.0, 50.0));
    assert!(seg_endpoint.intersects_rect(&rect));
}

#[test]
fn test_group_translation_and_attached_wires() {
    let mut canvas = SchematicCanvas::new();

    // c1 at (100, 100), c2 at (200, 100), c3 at (400, 100) with rotation 0
    let c1 = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(100.0, 100.0), 0);
    let c2 = SchematicComponent::new(2, ComponentKind::Resistor, Pos2::new(200.0, 100.0), 0);
    let c3 = SchematicComponent::new(3, ComponentKind::Resistor, Pos2::new(400.0, 100.0), 0);

    let c1_pin1 = c1.all_pins()[0].1;
    let c2_pin1 = c2.all_pins()[0].1;
    let c2_pin2 = c2.all_pins()[1].1;
    let c3_pin1 = c3.all_pins()[0].1;

    canvas.components = vec![c1, c2, c3];

    // Wire 1 connects c1 top pin to c2 top pin
    let w1 = SchematicWire::manhattan_route(1, c1_pin1, c2_pin1);
    // Wire 2 connects c2 bottom pin to c3 top pin
    let w2 = SchematicWire::manhattan_route(2, c2_pin2, c3_pin1);
    canvas.wires = vec![w1, w2];

    // Select c1 and c2, but NOT c3 and NOT any wires
    canvas.select_component(1, false);
    canvas.toggle_component_selection(2);

    let delta = Vec2::new(30.0, 40.0);
    canvas.translate_selection(delta);

    // c1 and c2 must be moved by delta
    assert_eq!(canvas.components[0].pos, Pos2::new(130.0, 140.0));
    assert_eq!(canvas.components[1].pos, Pos2::new(230.0, 140.0));
    assert_eq!(canvas.components[2].pos, Pos2::new(400.0, 100.0)); // unselected stays

    // w1 has both ends connected to moving components (c1 and c2), so it must be moved by delta
    assert_eq!(canvas.wires[0].start_point(), c1_pin1 + delta);
    assert_eq!(canvas.wires[0].end_point(), c2_pin1 + delta);

    // w2 has its start attached to moving c2, while its end remains at stationary c3
    assert_eq!(canvas.wires[1].start_point(), c2_pin2 + delta);
    assert_eq!(canvas.wires[1].end_point(), c3_pin1);
}

#[test]
fn test_group_deletion_and_atomic_batch_undo_redo() {
    let mut app = PhononApp::default();
    app.clear_canvas_state();
    app.history.clear();

    let c1 = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(100.0, 100.0), 1);
    let c2 = SchematicComponent::new(2, ComponentKind::Capacitor, Pos2::new(200.0, 100.0), 1);
    let c3 = SchematicComponent::new(3, ComponentKind::Inductor, Pos2::new(300.0, 100.0), 1);
    app.components = vec![c1, c2, c3];

    let w1 = SchematicWire::manhattan_route(1, Pos2::new(100.0, 80.0), Pos2::new(200.0, 80.0));
    let w2 = SchematicWire::manhattan_route(2, Pos2::new(200.0, 80.0), Pos2::new(300.0, 80.0));
    app.wires = vec![w1, w2];

    // Select c1, c2, and w1
    app.selected_component_ids.insert(1);
    app.selected_component_ids.insert(2);
    app.selected_wire_ids.insert(1);

    // Atomic group deletion
    app.delete_selected();

    // Only c3 and w2 should remain
    assert_eq!(app.components.len(), 1);
    assert_eq!(app.components[0].id, 3);
    assert_eq!(app.wires.len(), 1);
    assert_eq!(app.wires[0].id, 2);
    assert!(app.selected_component_ids.is_empty());
    assert!(app.selected_wire_ids.is_empty());

    // Single atomic undo
    let undo_success = app.undo();
    assert!(undo_success);
    assert_eq!(app.components.len(), 3);
    assert_eq!(app.wires.len(), 2);
    assert!(app.components.iter().any(|c| c.id == 1));
    assert!(app.components.iter().any(|c| c.id == 2));
    assert!(app.components.iter().any(|c| c.id == 3));
    assert!(app.wires.iter().any(|w| w.id == 1));
    assert!(app.wires.iter().any(|w| w.id == 2));

    // Single atomic redo
    let redo_success = app.redo();
    assert!(redo_success);
    assert_eq!(app.components.len(), 1);
    assert_eq!(app.wires.len(), 1);
    assert_eq!(app.components[0].id, 3);
    assert_eq!(app.wires[0].id, 2);
}

#[test]
fn test_group_duplication_offset_and_atomic_undo_redo() {
    let mut app = PhononApp::default();
    app.clear_canvas_state();
    app.history.clear();

    let c1 = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(100.0, 100.0), 0);
    let c2 = SchematicComponent::new(2, ComponentKind::Capacitor, Pos2::new(200.0, 100.0), 0);
    let c1_pin = c1.all_pins()[0].1;
    let c2_pin = c2.all_pins()[0].1;
    app.components = vec![c1, c2];
    app.next_comp_id = 3;

    let w1 = SchematicWire::manhattan_route(1, c1_pin, c2_pin);
    app.wires = vec![w1];
    app.next_wire_id = 2;

    // Select both c1 and c2
    app.selected_component_ids.insert(1);
    app.selected_component_ids.insert(2);

    // Duplicate selected items
    app.duplicate_selected();

    // 2 original + 2 duplicated = 4 components
    assert_eq!(app.components.len(), 4);
    // 1 original + 1 duplicated intra-selection wire = 2 wires
    assert_eq!(app.wires.len(), 2);

    let dup_c1 = app.components.iter().find(|c| c.id == 3).expect("Duplicated c1");
    let dup_c2 = app.components.iter().find(|c| c.id == 4).expect("Duplicated c2");
    let dup_w1 = app.wires.iter().find(|w| w.id == 2).expect("Duplicated w1");

    // Offsets must be (+40.0, +40.0)
    let offset = Vec2::new(40.0, 40.0);
    assert_eq!(dup_c1.pos, Pos2::new(140.0, 140.0));
    assert_eq!(dup_c2.pos, Pos2::new(240.0, 140.0));
    assert_eq!(dup_w1.start_point(), c1_pin + offset);
    assert_eq!(dup_w1.end_point(), c2_pin + offset);

    // Selection now tracks the duplicated items
    assert!(app.selected_component_ids.contains(&3));
    assert!(app.selected_component_ids.contains(&4));
    assert!(app.selected_wire_ids.contains(&2));

    // Atomic undo
    let undo_success = app.undo();
    assert!(undo_success);
    assert_eq!(app.components.len(), 2);
    assert_eq!(app.wires.len(), 1);

    // Atomic redo
    let redo_success = app.redo();
    assert!(redo_success);
    assert_eq!(app.components.len(), 4);
    assert_eq!(app.wires.len(), 2);
}

#[test]
fn test_group_rotation_and_atomic_undo_redo() {
    let mut app = PhononApp::default();
    app.clear_canvas_state();
    app.history.clear();

    let c1 = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(100.0, 100.0), 1);
    let c2 = SchematicComponent::new(2, ComponentKind::Capacitor, Pos2::new(200.0, 100.0), 1);
    assert_eq!(c1.rotation, 0);
    assert_eq!(c2.rotation, 0);
    app.components = vec![c1, c2];

    app.selected_component_ids.insert(1);
    app.selected_component_ids.insert(2);

    // Rotate active
    app.rotate_active();

    assert_eq!(app.components[0].rotation, 1);
    assert_eq!(app.components[1].rotation, 1);

    // Atomic undo
    let undo_success = app.undo();
    assert!(undo_success);
    assert_eq!(app.components[0].rotation, 0);
    assert_eq!(app.components[1].rotation, 0);

    // Atomic redo
    let redo_success = app.redo();
    assert!(redo_success);
    assert_eq!(app.components[0].rotation, 1);
    assert_eq!(app.components[1].rotation, 1);
}

#[test]
fn test_action_registry_and_fuzzy_search() {
    let registry = ActionRegistry::new();

    // Verify presence of standard actions
    assert!(registry.get(ActionId::ToolSelect).is_some());
    assert!(registry.get(ActionId::ToolWire).is_some());
    assert!(registry.get(ActionId::ToolBus).is_some());
    assert!(registry.get(ActionId::ToolProbe).is_some());
    assert!(registry.get(ActionId::Duplicate).is_some());
    assert!(registry.get(ActionId::Delete).is_some());
    assert!(registry.get(ActionId::SelectAll).is_some());
    assert!(registry.get(ActionId::OpenCommandPalette).is_some());
    assert!(registry.get(ActionId::ToggleFloatingToolbar).is_some());

    // Test fuzzy matching logic
    assert_eq!(ActionCategory::Edit.display_name(), "Edit");
    assert_eq!(ActionCategory::Tools.display_name(), "Tools");
    assert!(fuzzy_match_score("wire", "wire tool").is_some());
    assert!(fuzzy_match_score("xyz", "wire tool").is_none());
    assert!(fuzzy_match_score("", "anything").is_some());

    // Exact prefix match scores higher than non-consecutive
    let score_prefix = fuzzy_match_score("wire", "wire tool").unwrap();
    let score_scatter = fuzzy_match_score("wt", "wire tool").unwrap();
    assert!(score_prefix > score_scatter);

    // Test registry search
    let wire_results = registry.search("wire");
    assert!(!wire_results.is_empty());
    assert!(wire_results.iter().any(|a| a.id == ActionId::ToolWire));

    let dup_results = registry.search("dup");
    assert!(!dup_results.is_empty());
    assert_eq!(dup_results[0].id, ActionId::Duplicate);

    let sel_results = registry.search("select");
    assert!(!sel_results.is_empty());

    // Search with empty query returns all actions
    let all_results = registry.search("");
    assert_eq!(all_results.len(), registry.actions().len());
}

#[test]
fn test_command_palette_state() {
    let mut palette = CommandPalette::new();
    assert!(!palette.is_open);
    assert!(palette.search_query.is_empty());
    assert_eq!(palette.selected_index, 0);

    palette.open();
    assert!(palette.is_open);
    assert!(palette.request_focus);

    palette.search_query = "wire".to_string();
    palette.selected_index = 2;

    palette.toggle();
    assert!(!palette.is_open);

    palette.toggle();
    assert!(palette.is_open);
    assert!(palette.search_query.is_empty());
    assert_eq!(palette.selected_index, 0);

    palette.close();
    assert!(!palette.is_open);
}

#[test]
fn test_floating_toolbar_state_defaults() {
    let state = FloatingToolbarState::default();
    assert!(state.is_visible);
    assert!(!state.is_collapsed);
    assert_eq!(state.custom_pos, None);
}

#[test]
fn test_phonon_app_action_dispatch() {
    let mut app = PhononApp::default();
    app.clear_canvas_state();

    // Test ActionId::ToolWire
    app.execute_action(ActionId::ToolWire);
    assert_eq!(app.selected_tool, ToolMode::Wire);

    // Test ActionId::ToolBus
    app.execute_action(ActionId::ToolBus);
    assert_eq!(app.selected_tool, ToolMode::Bus);

    // Test ActionId::ToolProbe
    app.execute_action(ActionId::ToolProbe);
    assert_eq!(app.selected_tool, ToolMode::Probe);

    // Test ActionId::ToolSelect
    app.execute_action(ActionId::ToolSelect);
    assert_eq!(app.selected_tool, ToolMode::Select);

    // Test ActionId::ToggleFloatingToolbar
    assert!(app.floating_toolbar_state.is_visible);
    app.execute_action(ActionId::ToggleFloatingToolbar);
    assert!(!app.floating_toolbar_state.is_visible);
    app.execute_action(ActionId::ToggleFloatingToolbar);
    assert!(app.floating_toolbar_state.is_visible);

    // Test ActionId::OpenCommandPalette
    assert!(!app.command_palette.is_open);
    app.execute_action(ActionId::OpenCommandPalette);
    assert!(app.command_palette.is_open);
}

#[test]
fn test_drag_component_with_attached_wire_undo_redo() {
    let mut app = PhononApp::default();
    app.load_voltage_divider_demo();

    // In voltage divider demo:
    // V1 (comp id 1) is at (200, 300)
    // w1 (wire id 1) connects (200, 260) to (360, 200)
    let initial_v1_pos = app.components[0].pos;
    let initial_w1 = app.wires[0].clone();

    // Select V1
    app.select_component(1, false);
    assert!(app.is_component_selected(1));

    // Simulate drag start
    app.dragging_selection = true;
    app.drag_start_positions = vec![(1, initial_v1_pos)];
    app.drag_start_wires = app.wires.clone();

    // Drag by (+40, +40)
    let delta = Vec2::new(40.0, 40.0);
    app.translate_selection(delta);

    assert_eq!(app.components[0].pos, initial_v1_pos + delta);
    // Wire start point must be updated to new pin position
    assert_ne!(app.wires[0], initial_w1);
    assert_eq!(app.wires[0].start_point(), initial_w1.start_point() + delta);

    // Simulate drag stopped: compile batch and record
    let mut batch = Vec::new();
    for (id, start_pos) in app.drag_start_positions.drain(..) {
        if let Some(comp) = app.components.iter_mut().find(|c| c.id == id) {
            comp.pos = app.canvas.snap_to_grid(comp.pos);
            if comp.pos != start_pos {
                batch.push(CanvasCommand::MoveComponent {
                    id,
                    from: start_pos,
                    to: comp.pos,
                });
            }
        }
    }
    for start_wire in app.drag_start_wires.drain(..) {
        if let Some(curr_wire) = app.wires.iter().find(|w| w.id == start_wire.id) {
            if curr_wire.segments != start_wire.segments {
                batch.push(CanvasCommand::DeleteWire(start_wire));
                batch.push(CanvasCommand::AddWire(curr_wire.clone()));
            }
        }
    }
    assert!(!batch.is_empty());
    app.history.record(CanvasCommand::Batch(batch));
    app.dragging_selection = false;
    app.sync_canvas_state();

    // Now call Undo
    let undo_ok = app.undo();
    assert!(undo_ok);

    // V1 must be restored to its exact original position
    assert_eq!(app.components[0].pos, initial_v1_pos);
    // w1 must be restored to its exact original coordinates and segments!
    assert_eq!(app.wires[0], initial_w1);

    // Canvas state must also be perfectly in sync
    assert_eq!(app.canvas.components[0].pos, initial_v1_pos);
    assert_eq!(app.canvas.wires[0], initial_w1);

    // Now call Redo
    let redo_ok = app.redo();
    assert!(redo_ok);
    assert_eq!(app.components[0].pos, initial_v1_pos + delta);
    assert_eq!(app.wires[0].start_point(), initial_w1.start_point() + delta);

    // Undo again
    assert!(app.undo());
    assert_eq!(app.components[0].pos, initial_v1_pos);
    assert_eq!(app.wires[0], initial_w1);
}
