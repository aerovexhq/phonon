#![deny(unsafe_code)]

//! Comprehensive test suite validating the elimination of ghost selection halos
//! and the component mirroring feature (producing 8 distinct orientation combinations
//! via 4 rotations + 1 mirror on/off switch).

use egui::{Pos2, Vec2};
use phonon_gui::actions::{ActionCategory, ActionId, ActionRegistry};
use phonon_gui::app::ToolMode;
use phonon_gui::schematic::{
    deserialize_project, serialize_project, ComponentKind, SchematicComponent,
};
use phonon_gui::PhononApp;

#[test]
fn test_ghost_selection_halo_eliminated_on_move() {
    let mut app = PhononApp::default();
    app.load_voltage_divider_demo();
    assert!(!app.components.is_empty(), "App has demo components");

    let comp_id = app.components[0].id;
    let initial_pos = app.components[0].pos;

    // 1. Select the component
    app.select_component(comp_id, false);
    assert!(app.is_component_selected(comp_id));
    assert!(app.canvas.is_component_selected(comp_id));

    // Initially, both app.components and canvas.components are synchronized
    assert_eq!(app.components[0].pos, initial_pos);
    assert_eq!(app.canvas.components[0].pos, initial_pos);

    // 2. Drag / translate the selection by (+120.0, +80.0)
    let move_delta = Vec2::new(120.0, 80.0);
    app.translate_selection(move_delta);

    let expected_pos = initial_pos + move_delta;
    assert_eq!(app.components[0].pos, expected_pos);

    // CRITICAL: canvas.components must match app.components (no stale coordinates remaining)
    assert_eq!(app.canvas.components[0].pos, expected_pos);

    // Verify selection halos check:
    // When render_selection_halos is called with app.components,
    // the halo bounding box center is at expected_pos, not initial_pos!
    let active_comp = app.components.iter().find(|c| c.id == comp_id).unwrap();
    assert_eq!(active_comp.bounding_box().center(), expected_pos);
    assert!((active_comp.bounding_box().center() - initial_pos).length() > 50.0);
}

#[test]
fn test_mirror_horizontal_toggle() {
    let mut comp = SchematicComponent::new(1, ComponentKind::OpAmp, Pos2::new(100.0, 100.0), 1);
    assert!(!comp.mirrored, "Components default to unmirrored");

    comp.mirror_horizontal();
    assert!(comp.mirrored, "mirror_horizontal toggles mirror state to true");

    comp.mirror_horizontal();
    assert!(!comp.mirrored, "mirror_horizontal toggles mirror state back to false");
}

#[test]
fn test_eight_orientation_combinations_dihedral_group() {
    let mut comp = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::ZERO, 1);
    let probe = Vec2::new(30.0, 15.0);

    let mut transformed_points = Vec::with_capacity(8);

    // 4 rotations x 2 mirror states = 8 unique combinations
    for mirrored in [false, true] {
        comp.mirrored = mirrored;
        for rot in 0..4 {
            comp.rotation = rot;
            let pt = comp.transform_vec(probe);
            transformed_points.push((rot, mirrored, pt));
        }
    }

    assert_eq!(transformed_points.len(), 8);

    // Verify exact analytical coordinates for each of the 8 states:
    // Unmirrored (rotations of (30, 15)):
    assert_eq!(transformed_points[0].2, Vec2::new(30.0, 15.0));   // 0 deg
    assert_eq!(transformed_points[1].2, Vec2::new(-15.0, 30.0));  // 90 deg CW
    assert_eq!(transformed_points[2].2, Vec2::new(-30.0, -15.0)); // 180 deg
    assert_eq!(transformed_points[3].2, Vec2::new(15.0, -30.0));  // 270 deg

    // Mirrored (horizontal flip (-30, 15) then rotations):
    assert_eq!(transformed_points[4].2, Vec2::new(-30.0, 15.0));  // 0 deg mirrored
    assert_eq!(transformed_points[5].2, Vec2::new(-15.0, -30.0)); // 90 deg CW mirrored
    assert_eq!(transformed_points[6].2, Vec2::new(30.0, -15.0));  // 180 deg mirrored
    assert_eq!(transformed_points[7].2, Vec2::new(15.0, 30.0));   // 270 deg CW mirrored

    // Verify all 8 vectors are distinct (cardinality == 8)
    for i in 0..8 {
        for j in (i + 1)..8 {
            let diff = (transformed_points[i].2 - transformed_points[j].2).length();
            assert!(
                diff > 1.0,
                "State {:?} and State {:?} must produce distinct coordinates (diff = {})",
                (transformed_points[i].0, transformed_points[i].1),
                (transformed_points[j].0, transformed_points[j].1),
                diff
            );
        }
    }
}

#[test]
fn test_pin_positions_with_mirroring() {
    let mut inv = SchematicComponent::new(10, ComponentKind::Inverter, Pos2::new(200.0, 200.0), 1);

    // Inverter on-grid pins: IN at (-40, 0), OUT at (40, 0)
    let in_unmirrored = inv.pin_world_pos(0).unwrap();
    let out_unmirrored = inv.pin_world_pos(1).unwrap();
    assert_eq!(in_unmirrored, Pos2::new(160.0, 200.0));
    assert_eq!(out_unmirrored, Pos2::new(240.0, 200.0));

    // Mirror horizontally
    inv.mirror_horizontal();
    let in_mirrored = inv.pin_world_pos(0).unwrap();
    let out_mirrored = inv.pin_world_pos(1).unwrap();

    // IN should flip to +40 relative (+X direction) -> (240, 200)
    // OUT should flip to -40 relative (-X direction) -> (160, 200)
    assert_eq!(in_mirrored, Pos2::new(240.0, 200.0));
    assert_eq!(out_mirrored, Pos2::new(160.0, 200.0));

    // all_pins() must also reflect mirrored coordinates
    let pins = inv.all_pins();
    assert_eq!(pins[0], ("IN", Pos2::new(240.0, 200.0)));
    assert_eq!(pins[1], ("OUT", Pos2::new(160.0, 200.0)));
}

#[test]
fn test_mirror_history_undo_redo() {
    let mut app = PhononApp::default();
    app.load_voltage_divider_demo();
    let cid = app.components[0].id;
    app.select_component(cid, false);

    assert!(!app.components[0].mirrored);

    // 1. Mirror active component
    app.mirror_active();
    assert!(app.components[0].mirrored, "Component must be mirrored after mirror_active");

    // 2. Undo
    app.undo();
    assert!(!app.components[0].mirrored, "Component must be restored to unmirrored on undo");

    // 3. Redo
    app.redo();
    assert!(app.components[0].mirrored, "Component must be re-mirrored on redo");
}

#[test]
fn test_action_registry_mirror_registration() {
    let registry = ActionRegistry::new();
    let def = registry.get(ActionId::MirrorComponent).expect("MirrorComponent must be registered");

    assert_eq!(def.title, "Mirror Component");
    assert_eq!(def.category, ActionCategory::Edit);
    assert_eq!(def.shortcut, Some("M"));
}

#[test]
fn test_placement_mirror_controls() {
    let mut app = PhononApp::default();
    app.selected_tool = ToolMode::PlaceComponent(ComponentKind::Diode);

    assert!(!app.placement_mirrored);

    // Toggle mirror during placement mode
    app.mirror_active();
    assert!(app.placement_mirrored, "mirror_active in place mode toggles placement_mirrored");

    // Rotate placement as well
    app.rotate_active();
    assert_eq!(app.placement_rotation, 1);

    // Place the component
    let place_pos = Pos2::new(320.0, 440.0);
    let mut placed = SchematicComponent::new(999, ComponentKind::Diode, place_pos, 1);
    placed.rotation = app.placement_rotation;
    placed.mirrored = app.placement_mirrored;

    assert_eq!(placed.rotation, 1);
    assert!(placed.mirrored);
}

#[test]
fn test_binary_persistence_with_mirrored_component() {
    let title = "Mirrored Component Persistence Test";
    let mut c1 = SchematicComponent::new(1, ComponentKind::OpAmp, Pos2::new(100.0, 100.0), 1);
    c1.rotation = 1;
    c1.mirrored = true;

    let mut c2 = SchematicComponent::new(2, ComponentKind::Nmos, Pos2::new(250.0, 200.0), 1);
    c2.rotation = 3;
    c2.mirrored = false;

    let bytes = serialize_project(title, &[c1.clone(), c2.clone()], &[]);
    assert!(!bytes.is_empty());

    let loaded = deserialize_project(&bytes).expect("Deserialization must succeed");
    assert_eq!(loaded.components.len(), 2);

    assert_eq!(loaded.components[0].id, 1);
    assert_eq!(loaded.components[0].rotation, 1);
    assert!(loaded.components[0].mirrored, "Loaded component 1 must preserve mirrored = true");

    assert_eq!(loaded.components[1].id, 2);
    assert_eq!(loaded.components[1].rotation, 3);
    assert!(!loaded.components[1].mirrored, "Loaded component 2 must preserve mirrored = false");
}
