#![deny(unsafe_code)]

//! Comprehensive test suite verifying that every component in Phonon Studio
//! has all pins strictly aligned to the 20px grid, and that post-placement
//! sizing (scale up / down) only permits scaling factors where all pins remain on grid.

use egui::Pos2;
use phonon_gui::schematic::{
    deserialize_history, read_command, serialize_history, write_command, ActionOpcode,
    CanvasCommand, ComponentKind, HistoryStack, SchematicComponent,
};

const GRID_SIZE: f32 = 20.0;

#[test]
fn test_all_71_components_have_pins_strictly_on_grid() {
    assert_eq!(
        ComponentKind::ALL.len(),
        71,
        "Phonon Studio must have exactly 71 categorized component kinds"
    );

    let base_pos = Pos2::new(100.0, 100.0);

    for &kind in ComponentKind::ALL {
        let mut comp = SchematicComponent::new(1, kind, base_pos, 1);

        // Test across all 4 rotations and both mirror states
        for rot in 0..4 {
            comp.rotation = rot;
            for mirrored in [false, true] {
                comp.mirrored = mirrored;

                let pins = comp.all_pins();
                for (name, pin_pos) in pins {
                    let dx = (pin_pos.x - base_pos.x).abs();
                    let dy = (pin_pos.y - base_pos.y).abs();

                    let rem_x = dx % GRID_SIZE;
                    let rem_y = dy % GRID_SIZE;

                    let on_grid_x = rem_x < 1e-3 || (GRID_SIZE - rem_x) < 1e-3;
                    let on_grid_y = rem_y < 1e-3 || (GRID_SIZE - rem_y) < 1e-3;

                    assert!(
                        on_grid_x && on_grid_y,
                        "Component {:?} pin '{}' at rot={}, mirrored={} pos=({:.1}, {:.1}) is off the 20px grid! dx={}, dy={}, rem_x={}, rem_y={}",
                        kind,
                        name,
                        rot,
                        mirrored,
                        pin_pos.x,
                        pin_pos.y,
                        dx,
                        dy,
                        rem_x,
                        rem_y
                    );
                }
            }
        }
    }
}

#[test]
fn test_xvt2_vocal_tract_pins_strictly_on_grid() {
    let base_pos = Pos2::new(200.0, 200.0);
    let comp = SchematicComponent::new(1, ComponentKind::PhVocalTract, base_pos, 1);
    let pins = comp.all_pins();

    assert_eq!(pins.len(), 4, "PhVocalTract must have exactly 4 pins");

    for (name, pin_pos) in pins {
        let dx = (pin_pos.x - base_pos.x).abs();
        let dy = (pin_pos.y - base_pos.y).abs();

        let rem_x = dx % GRID_SIZE;
        let rem_y = dy % GRID_SIZE;

        let on_grid_x = rem_x < 1e-3 || (GRID_SIZE - rem_x) < 1e-3;
        let on_grid_y = rem_y < 1e-3 || (GRID_SIZE - rem_y) < 1e-3;

        assert!(
            on_grid_x && on_grid_y,
            "PhVocalTract pin '{}' at ({:.1}, {:.1}) must be strictly on 20px grid",
            name,
            pin_pos.x,
            pin_pos.y
        );
    }

    // Direct check of pin offsets
    let defs = ComponentKind::PhVocalTract.pin_definitions();
    for (name, offset) in defs {
        assert_ne!(
            offset.y.abs(),
            32.0,
            "PhVocalTract pin '{}' local offset must not be +/-32.0 px",
            name
        );
        let rem_x = offset.x.abs() % GRID_SIZE;
        let rem_y = offset.y.abs() % GRID_SIZE;
        assert!(
            rem_x < 1e-3 || (GRID_SIZE - rem_x) < 1e-3,
            "PhVocalTract pin '{}' offset.x = {} is not multiple of 20",
            name,
            offset.x
        );
        assert!(
            rem_y < 1e-3 || (GRID_SIZE - rem_y) < 1e-3,
            "PhVocalTract pin '{}' offset.y = {} is not multiple of 20",
            name,
            offset.y
        );
    }
}

#[test]
fn test_scale_validation_rejects_off_grid_scales() {
    let mut comp = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(100.0, 100.0), 1);

    // Resistor pin offsets: (0, -40), (0, 40)
    // Scale 1.0 -> (0, -40), (0, 40) -> on grid
    assert!(comp.is_valid_scale(1.0, GRID_SIZE));

    // Scale 1.5 -> (0, -60), (0, 60) -> on grid
    assert!(comp.is_valid_scale(1.5, GRID_SIZE));

    // Scale 2.0 -> (0, -80), (0, 80) -> on grid
    assert!(comp.is_valid_scale(2.0, GRID_SIZE));

    // Scale 0.5 -> (0, -20), (0, 20) -> on grid
    assert!(comp.is_valid_scale(0.5, GRID_SIZE));

    // Scale 1.25 -> (0, -50), (0, 50) -> 50 % 20 = 10 -> OFF GRID!
    assert!(!comp.is_valid_scale(1.25, GRID_SIZE));

    // Scale 0.75 -> (0, -30), (0, 30) -> 30 % 20 = 10 -> OFF GRID!
    assert!(!comp.is_valid_scale(0.75, GRID_SIZE));

    // Scale 1.1 -> OFF GRID
    assert!(!comp.is_valid_scale(1.1, GRID_SIZE));

    // Attempting to set an off-grid scale must fail
    assert!(!comp.set_scale(1.25, GRID_SIZE));
    assert_eq!(comp.scale, 1.0, "Scale must remain 1.0 after rejected scale");

    // Setting a valid scale must succeed
    assert!(comp.set_scale(1.5, GRID_SIZE));
    assert_eq!(comp.scale, 1.5, "Scale must update to 1.5");
}

#[test]
fn test_size_up_and_size_down_stepper() {
    let mut comp = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(100.0, 100.0), 1);
    assert_eq!(comp.scale, 1.0);

    // Size up from 1.0 -> 1.5
    assert!(comp.size_up(GRID_SIZE));
    assert_eq!(comp.scale, 1.5);

    // Size up from 1.5 -> 2.0
    assert!(comp.size_up(GRID_SIZE));
    assert_eq!(comp.scale, 2.0);

    // Size down from 2.0 -> 1.5
    assert!(comp.size_down(GRID_SIZE));
    assert_eq!(comp.scale, 1.5);

    // Size down from 1.5 -> 1.0
    assert!(comp.size_down(GRID_SIZE));
    assert_eq!(comp.scale, 1.0);

    // Size down from 1.0 -> 0.5
    assert!(comp.size_down(GRID_SIZE));
    assert_eq!(comp.scale, 0.5);

    // Pins must still be on grid at scale 0.5
    for (_, pin_pos) in comp.all_pins() {
        assert_eq!((pin_pos.x - 100.0).abs() % GRID_SIZE, 0.0);
        assert_eq!((pin_pos.y - 100.0).abs() % GRID_SIZE, 0.0);
    }
}

#[test]
fn test_scale_undo_redo_history() {
    let mut components = vec![SchematicComponent::new(
        1,
        ComponentKind::Resistor,
        Pos2::new(100.0, 100.0),
        1,
    )];
    let mut wires = Vec::new();
    let mut history = HistoryStack::with_max_depth(500);

    // Execute scale change 1.0 -> 1.5
    let cmd = CanvasCommand::ScaleComponent {
        id: 1,
        from_scale: 1.0,
        to_scale: 1.5,
    };
    cmd.execute(&mut components, &mut wires);
    assert_eq!(components[0].scale, 1.5);

    history.record(cmd);
    assert!(history.can_undo());

    // Undo -> 1.0
    history.undo(&mut components, &mut wires);
    assert_eq!(components[0].scale, 1.0);

    // Redo -> 1.5
    assert!(history.can_redo());
    history.redo(&mut components, &mut wires);
    assert_eq!(components[0].scale, 1.5);
}

#[test]
fn test_binary_history_scale_serialization_roundtrip() {
    let cmd = CanvasCommand::ScaleComponent {
        id: 42,
        from_scale: 1.0,
        to_scale: 2.0,
    };

    let mut bytes = Vec::new();
    write_command(&cmd, &mut bytes);
    assert_eq!(bytes[0], ActionOpcode::ScaleComponent as u8);

    let mut cursor = 0;
    let read_cmd = read_command(&bytes, &mut cursor).expect("Failed to read ScaleComponent");
    assert_eq!(cursor, bytes.len());
    assert_eq!(cmd, read_cmd);

    let mut history = HistoryStack::with_max_depth(500);
    history.record(cmd);

    let serialized = serialize_history(&history, 100);
    let deserialized = deserialize_history(&serialized, 100).expect("Deserialization must succeed");

    assert_eq!(deserialized.undo_count(), 1);
    assert_eq!(deserialized.clean_index, 0);

    let mut components = vec![SchematicComponent::new(
        42,
        ComponentKind::Capacitor,
        Pos2::new(200.0, 200.0),
        1,
    )];
    let mut wires = Vec::new();

    // Execute restored command
    deserialized.undo_stack[0].execute(&mut components, &mut wires);
    assert_eq!(components[0].scale, 2.0);

    // Undo restored command
    deserialized.undo_stack[0].undo(&mut components, &mut wires);
    assert_eq!(components[0].scale, 1.0);
}

#[test]
fn test_scaled_bounding_box_expands_and_contracts() {
    let mut comp = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(100.0, 100.0), 1);
    let bb_1 = comp.bounding_box();

    comp.scale = 2.0;
    let bb_2 = comp.bounding_box();

    assert_eq!(bb_2.width(), bb_1.width() * 2.0);
    assert_eq!(bb_2.height(), bb_1.height() * 2.0);

    comp.scale = 0.5;
    let bb_half = comp.bounding_box();

    assert_eq!(bb_half.width(), bb_1.width() * 0.5);
    assert_eq!(bb_half.height(), bb_1.height() * 0.5);
}
