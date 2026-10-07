#![deny(unsafe_code)]

//! Comprehensive test suite for Phonon UI DevTools: Synthetic scripting, drag interactions,
//! wire rerouting, simulation triggers, and visual automation.

use egui::{Context, Pos2, Vec2};
use phonon_gui::devtools::{DevtoolsState, ScriptRunner, UiScript};
use phonon_gui::schematic::ErcEngine;
use phonon_gui::PhononApp;

#[test]
fn test_ui_script_component_drag_state() {
    let mut app = PhononApp::default();
    assert_eq!(app.components.len(), 4, "Initial circuit must have 4 components");
    assert_eq!(app.wires.len(), 4, "Initial circuit must have 4 wires");

    // Resistor R1 is component ID 1
    let r1_initial_pos = app
        .components
        .iter()
        .find(|c| c.id == 1)
        .map(|c| c.pos)
        .expect("Component R1 must exist");

    // Step 1: Start dragging R1
    app.devtools_start_drag(1);
    assert!(app.dragging_selection, "Drag state must be active");
    assert!(app.is_component_selected(1), "R1 must be selected");

    // Step 2: Drag R1 by (+80.0, +60.0)
    let delta = Vec2::new(80.0, 60.0);
    app.devtools_drag_delta(delta);
    let r1_drag_pos = app
        .components
        .iter()
        .find(|c| c.id == 1)
        .map(|c| c.pos)
        .expect("Component R1 must exist");
    assert_eq!(
        r1_drag_pos,
        r1_initial_pos + delta,
        "R1 must be located at the dragged position"
    );

    // Step 3: Finish drag and commit Manhattan routes
    app.devtools_finish_drag();
    assert!(!app.dragging_selection, "Drag state must be finalized");

    let r1_final_pos = app
        .components
        .iter()
        .find(|c| c.id == 1)
        .map(|c| c.pos)
        .expect("Component R1 must exist");
    assert_eq!(r1_final_pos, r1_initial_pos + delta);

    // Verify ERC produces zero critical errors
    let erc_diagnostics = ErcEngine::evaluate_canvas(&app.canvas);
    let error_count = erc_diagnostics
        .iter()
        .filter(|d| d.severity == phonon_gui::schematic::ErcSeverity::Error)
        .count();
    assert_eq!(error_count, 0, "Dragged circuit must produce zero ERC errors");
}

#[test]
fn test_ui_script_transient_simulation() {
    let mut app = PhononApp::default();
    assert!(app.oscilloscope.traces.is_empty(), "Oscilloscope should start empty");

    // Execute transient simulation
    app.run_transient_demo();

    // Verify traces generated
    assert!(
        !app.oscilloscope.traces.is_empty(),
        "Oscilloscope must contain traces after transient solve"
    );
    assert!(
        app.sim_status.contains("Transient Solved"),
        "Status must report successful transient solution: {}",
        app.sim_status
    );
}

#[test]
fn test_ui_script_marquee_selection() {
    let mut app = PhononApp::default();
    app.clear_selection();
    assert_eq!(app.canvas.selected_component_ids.len(), 0);

    // Start marquee bounding box
    app.devtools_start_marquee(Pos2::new(50.0, 50.0));
    assert!(app.marquee_start.is_some());

    // Expand marquee over multiple components
    app.devtools_update_marquee(Pos2::new(450.0, 450.0));
    assert_eq!(app.marquee_current, Some(Pos2::new(450.0, 450.0)));

    // Finalize marquee
    app.devtools_finish_marquee();
    assert!(app.marquee_start.is_none());
    assert!(
        app.canvas.selected_component_ids.len() >= 2,
        "Marquee must select multiple components in rectangle"
    );
}

#[test]
fn test_ui_script_preferences_dialog() {
    let mut app = PhononApp::default();
    assert!(!app.preferences_dialog.is_open);

    app.preferences_dialog.open(None);
    assert!(app.preferences_dialog.is_open);

    app.preferences_dialog.close();
    assert!(!app.preferences_dialog.is_open);
}

#[test]
fn test_script_runner_execution() {
    let mut app = PhononApp::default();
    let script = UiScript::component_drag_scenario();
    let mut runner = ScriptRunner::new(script);

    assert!(!runner.is_finished);
    assert_eq!(runner.step_index, 0);

    // Step through the entire script
    let mut requested_screenshots = Vec::new();
    while !runner.is_finished {
        if let Some((label, filename)) = runner.step(&mut app) {
            requested_screenshots.push((label, filename));
        }
    }

    assert!(runner.is_finished, "ScriptRunner must finish");
    assert_eq!(
        requested_screenshots.len(),
        4,
        "Component drag script must request 4 keyframe screenshots"
    );
    assert!(!runner.execution_log.is_empty(), "Runner must record execution logs");
}

#[test]
fn test_devtools_panel_headless_render() {
    let mut app = PhononApp::default();
    let mut state = DevtoolsState::default();
    let ctx = Context::default();

    let mut output = ctx.run_ui(Default::default(), |ui| {
        state.render(ui.ctx(), &mut app);
    });
    output.textures_delta.clear();

    assert_eq!(state.frame_counter, 1);
    assert!(state.visible);
}
