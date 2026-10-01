#![deny(unsafe_code)]

//! Verification suite for Phonon Studio Full Undo/Redo History Stack & Non-Destructive Action Command Engine (Phase 309).
//!
//! Validates reversible command pattern semantics, dirty state tracking, drag coalescing,
//! wire and component manipulation, depth bounding, and high-throughput execution.

use egui::Pos2;
use phonon_gui::schematic::{
    CanvasCommand, ComponentKind, HistoryStack, SchematicComponent, SchematicWire,
};
use phonon_gui::PhononApp;
use std::time::Instant;

#[test]
fn test_history_empty_state() {
    let mut stack = HistoryStack::new();
    let mut components = Vec::new();
    let mut wires = Vec::new();

    assert!(!stack.can_undo(), "Empty history stack cannot undo");
    assert!(!stack.can_redo(), "Empty history stack cannot redo");
    assert_eq!(stack.undo_count(), 0);
    assert_eq!(stack.redo_count(), 0);
    assert!(!stack.is_dirty(), "Empty history stack is not dirty");

    assert!(!stack.undo(&mut components, &mut wires));
    assert!(!stack.redo(&mut components, &mut wires));

    // Verify PhononApp starts with a clean history state
    let app = PhononApp::default();
    assert!(!app.history.can_undo(), "Fresh app start must have empty undo stack");
    assert!(!app.history.can_redo(), "Fresh app start must have empty redo stack");
    assert!(!app.history.is_dirty(), "Fresh app start must not be dirty");
}

#[test]
fn test_add_and_undo_component() {
    let mut stack = HistoryStack::new();
    let mut components = Vec::new();
    let mut wires = Vec::new();

    let comp = SchematicComponent::new(42, ComponentKind::Resistor, Pos2::new(120.0, 240.0), 1);
    components.push(comp.clone());
    stack.record(CanvasCommand::AddComponent(comp.clone()));

    assert!(stack.can_undo());
    assert!(!stack.can_redo());
    assert_eq!(stack.undo_count(), 1);
    assert!(stack.is_dirty());

    // Undo: should remove the component
    assert!(stack.undo(&mut components, &mut wires));
    assert!(components.is_empty(), "Component must be removed on undo");
    assert!(!stack.can_undo());
    assert!(stack.can_redo());

    // Redo: should restore the exact component with original ID and coordinates
    assert!(stack.redo(&mut components, &mut wires));
    assert_eq!(components.len(), 1, "Component must be restored on redo");
    assert_eq!(components[0].id, 42);
    assert_eq!(components[0].pos, Pos2::new(120.0, 240.0));
    assert_eq!(components[0].kind, ComponentKind::Resistor);
    assert_eq!(components[0].name, comp.name);
    assert_eq!(components[0].value_str, comp.value_str);
}

#[test]
fn test_delete_and_undo_component() {
    let mut stack = HistoryStack::new();
    let mut components = Vec::new();
    let mut wires = Vec::new();

    let comp = SchematicComponent::new(10, ComponentKind::Capacitor, Pos2::new(300.0, 150.0), 1);
    components.push(comp.clone());

    // Delete component
    let removed = components.remove(0);
    stack.record(CanvasCommand::DeleteComponent(removed.clone()));

    assert!(components.is_empty());
    assert!(stack.can_undo());

    // Undo deletion: component is restored
    assert!(stack.undo(&mut components, &mut wires));
    assert_eq!(components.len(), 1);
    assert_eq!(components[0].id, 10);
    assert_eq!(components[0].kind, ComponentKind::Capacitor);
    assert_eq!(components[0].pos, Pos2::new(300.0, 150.0));

    // Redo deletion: component is removed again
    assert!(stack.redo(&mut components, &mut wires));
    assert!(components.is_empty());
}

#[test]
fn test_move_coalescing_and_undo() {
    let mut stack = HistoryStack::new();
    let mut components = vec![SchematicComponent::new(
        1,
        ComponentKind::Inductor,
        Pos2::new(100.0, 100.0),
        1,
    )];
    let mut wires = Vec::new();

    let start_pos = Pos2::new(100.0, 100.0);
    let final_pos = Pos2::new(250.0, 300.0);

    // Simulate drag coalescing: thousands of intermediate delta frames produce 1 atomic command
    let intermediate_positions = [
        Pos2::new(102.0, 105.0),
        Pos2::new(120.0, 130.0),
        Pos2::new(180.0, 210.0),
        final_pos,
    ];
    for &pos in &intermediate_positions {
        components[0].pos = pos;
    }

    // On pointer release, only one command is recorded
    stack.record(CanvasCommand::MoveComponent {
        id: 1,
        from: start_pos,
        to: final_pos,
    });

    assert_eq!(stack.undo_count(), 1, "Move coalescing must push exactly 1 history command");
    assert_eq!(components[0].pos, final_pos);

    // Undo: position returns to start_pos
    assert!(stack.undo(&mut components, &mut wires));
    assert_eq!(components[0].pos, start_pos, "Undo must restore start position");

    // Redo: position returns to final_pos
    assert!(stack.redo(&mut components, &mut wires));
    assert_eq!(components[0].pos, final_pos, "Redo must restore final position");
}

#[test]
fn test_rotate_component_undo_redo() {
    let mut stack = HistoryStack::new();
    let mut components = vec![SchematicComponent::new(
        1,
        ComponentKind::Diode,
        Pos2::new(200.0, 200.0),
        1,
    )];
    let mut wires = Vec::new();

    let pins_initial = components[0].all_pins();
    assert_eq!(components[0].rotation, 0);

    // Rotate clockwise 90 degrees
    let from_rot = components[0].rotation;
    components[0].rotate_clockwise();
    let to_rot = components[0].rotation;
    assert_eq!(to_rot, 1);

    stack.record(CanvasCommand::RotateComponent {
        id: 1,
        from_rot,
        to_rot,
    });

    let pins_rotated = components[0].all_pins();
    assert_ne!(pins_initial, pins_rotated, "Pin positions must change upon rotation");

    // Undo rotation: restores previous rotation index and pin geometry
    assert!(stack.undo(&mut components, &mut wires));
    assert_eq!(components[0].rotation, 0);
    assert_eq!(
        components[0].all_pins(),
        pins_initial,
        "Pin geometry must be completely restored on undo"
    );

    // Redo rotation
    assert!(stack.redo(&mut components, &mut wires));
    assert_eq!(components[0].rotation, 1);
    assert_eq!(components[0].all_pins(), pins_rotated);
}

#[test]
fn test_wire_add_delete_undo() {
    let mut stack = HistoryStack::new();
    let mut components = Vec::new();
    let mut wires = Vec::new();

    let wire = SchematicWire::manhattan_route(100, Pos2::new(50.0, 50.0), Pos2::new(200.0, 150.0));
    wires.push(wire.clone());
    stack.record(CanvasCommand::AddWire(wire.clone()));

    assert_eq!(wires.len(), 1);

    // Undo add wire
    assert!(stack.undo(&mut components, &mut wires));
    assert!(wires.is_empty(), "Wire must be removed on undo");

    // Redo add wire
    assert!(stack.redo(&mut components, &mut wires));
    assert_eq!(wires.len(), 1);
    assert_eq!(wires[0].id, 100);

    // Delete wire
    let removed_wire = wires.remove(0);
    stack.record(CanvasCommand::DeleteWire(removed_wire));
    assert!(wires.is_empty());

    // Undo delete wire
    assert!(stack.undo(&mut components, &mut wires));
    assert_eq!(wires.len(), 1);
    assert_eq!(wires[0].id, 100);

    // Redo delete wire
    assert!(stack.redo(&mut components, &mut wires));
    assert!(wires.is_empty());
}

#[test]
fn test_clear_all_undo() {
    let mut stack = HistoryStack::new();
    let mut components = vec![
        SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(100.0, 100.0), 1),
        SchematicComponent::new(2, ComponentKind::Capacitor, Pos2::new(200.0, 100.0), 1),
        SchematicComponent::new(3, ComponentKind::Ground, Pos2::new(150.0, 200.0), 1),
    ];
    let mut wires = vec![
        SchematicWire::manhattan_route(1, Pos2::new(100.0, 100.0), Pos2::new(200.0, 100.0)),
        SchematicWire::manhattan_route(2, Pos2::new(150.0, 100.0), Pos2::new(150.0, 200.0)),
    ];

    let saved_comps = components.clone();
    let saved_wires = wires.clone();

    // Clear canvas
    components.clear();
    wires.clear();
    stack.record(CanvasCommand::ClearAll {
        components: saved_comps,
        wires: saved_wires,
    });

    assert!(components.is_empty());
    assert!(wires.is_empty());

    // Undo clear: restores all components and wires
    assert!(stack.undo(&mut components, &mut wires));
    assert_eq!(components.len(), 3, "All components must be restored on undo");
    assert_eq!(wires.len(), 2, "All wires must be restored on undo");
    assert_eq!(components[0].id, 1);
    assert_eq!(components[1].id, 2);
    assert_eq!(components[2].id, 3);
    assert_eq!(wires[0].id, 1);
    assert_eq!(wires[1].id, 2);

    // Redo clear: canvas is cleared again
    assert!(stack.redo(&mut components, &mut wires));
    assert!(components.is_empty());
    assert!(wires.is_empty());
}

#[test]
fn test_history_depth_limit() {
    let max_depth = 5;
    let mut stack = HistoryStack::with_max_depth(max_depth);
    let mut components = Vec::new();
    let mut wires = Vec::new();

    // Push 12 commands
    for i in 1..=12 {
        let comp = SchematicComponent::new(i, ComponentKind::Resistor, Pos2::new(i as f32, 0.0), i);
        stack.record(CanvasCommand::AddComponent(comp));
    }

    assert_eq!(
        stack.undo_count(),
        max_depth,
        "History stack must be truncated to max_depth"
    );

    // Undo all remaining commands
    let mut undos = 0;
    while stack.can_undo() {
        assert!(stack.undo(&mut components, &mut wires));
        undos += 1;
    }
    assert_eq!(undos, max_depth);
    assert_eq!(stack.undo_count(), 0);
    assert_eq!(stack.redo_count(), max_depth);
}

#[test]
fn test_history_throughput_benchmark() {
    let mut components = vec![SchematicComponent::new(
        1,
        ComponentKind::Resistor,
        Pos2::new(10.0, 10.0),
        1,
    )];
    let mut wires = Vec::new();

    let cmd = CanvasCommand::MoveComponent {
        id: 1,
        from: Pos2::new(10.0, 10.0),
        to: Pos2::new(20.0, 20.0),
    };

    let start = Instant::now();
    let iterations = 10_000;
    for _ in 0..iterations {
        cmd.execute(&mut components, &mut wires);
        cmd.undo(&mut components, &mut wires);
        cmd.execute(&mut components, &mut wires);
    }
    let elapsed = start.elapsed();
    let total_ops = iterations * 3; // execute, undo, redo/execute
    let ops_per_sec = (total_ops as f64) / elapsed.as_secs_f64();

    eprintln!(
        "History command engine throughput: {} ops in {:?} ({:.2} ops/sec)",
        total_ops, elapsed, ops_per_sec
    );

    assert!(
        elapsed.as_millis() < 15,
        "10,000 execute/undo/redo cycles took {:?}, exceeding 15 ms limit",
        elapsed
    );
    assert!(
        ops_per_sec > 650_000.0,
        "Throughput {:.2} ops/sec below 650,000 ops/sec threshold",
        ops_per_sec
    );

    // Also benchmark HistoryStack undo / redo cycles
    let mut stack = HistoryStack::new();
    stack.record(cmd);
    let start_stack = Instant::now();
    for _ in 0..iterations {
        stack.undo(&mut components, &mut wires);
        stack.redo(&mut components, &mut wires);
    }
    let elapsed_stack = start_stack.elapsed();
    let stack_total_ops = iterations * 2;
    let stack_ops_per_sec = (stack_total_ops as f64) / elapsed_stack.as_secs_f64();

    eprintln!(
        "HistoryStack undo/redo throughput: {} ops in {:?} ({:.2} ops/sec)",
        stack_total_ops, elapsed_stack, stack_ops_per_sec
    );

    assert!(
        elapsed_stack.as_millis() < 15,
        "10,000 HistoryStack undo/redo cycles took {:?}, exceeding 15 ms limit",
        elapsed_stack
    );
    assert!(
        stack_ops_per_sec > 650_000.0,
        "Throughput {:.2} ops/sec below 650,000 ops/sec threshold",
        stack_ops_per_sec
    );
}
