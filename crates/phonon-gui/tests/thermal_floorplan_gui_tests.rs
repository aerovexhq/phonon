#![deny(unsafe_code)]

//! Verification test suite for Phonon Visual Studio Interactive CAD Studio
//! Thermal Floorplan & Heatmap Visualizer dialog.

use phonon_core::CircuitGraph;
use phonon_gui::thermal::Colormap;
use phonon_gui::widgets::thermal_floorplan_dialog::ThermalFloorplanDialog;
use phonon_models::diode::DiodeModel;
use phonon_solver::mna::ModelContext;

#[test]
fn test_dialog_initialization_default_floorplan_and_component_bindings() {
    let dialog = ThermalFloorplanDialog::new();

    // Dialog must be closed by default
    assert!(!dialog.is_open, "Dialog should be closed by default");

    // Verify semiconductor die default geometry
    assert_eq!(dialog.mesh.die.width, 0.005);
    assert_eq!(dialog.mesh.die.height, 0.005);
    assert_eq!(dialog.mesh.die.thickness, 0.0003);
    assert_eq!(dialog.mesh.die.ambient_temperature, 300.0);
    assert!(dialog.mesh.die.use_silicon_k);

    // Verify default components placed on the die
    let comp_names: Vec<&str> = dialog
        .mesh
        .components
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert!(comp_names.contains(&"D1"));
    assert!(comp_names.contains(&"R1"));
    assert!(comp_names.contains(&"M1"));

    // Verify visualizer display toggles and colormap defaults
    assert_eq!(dialog.colormap, Colormap::Turbo);
    assert!(dialog.show_heatmap);
    assert!(dialog.show_contours);
    assert!(dialog.show_components);
    assert!(dialog.show_hotspot_reticle);
    assert_eq!(dialog.contour_count, 5);
    assert_eq!(dialog.runaway_threshold_k, 500.0);

    // Baseline simulation data must be present
    assert!(
        dialog.trajectory.is_some(),
        "Baseline trajectory should be populated"
    );
    let traj = dialog.trajectory.as_ref().unwrap();
    assert!(!traj.is_empty());
}

#[test]
fn test_simulation_execution_and_timeline_scrubber() {
    let mut dialog = ThermalFloorplanDialog::new();

    // Configure test circuit
    let mut graph = CircuitGraph::new();
    let _ = graph.add_voltage_source("V1", "n_in", "0", 8.0);
    let _ = graph.add_resistor("R1", "n_in", "n_d", 30.0);
    let _ = graph.add_diode("D1", "n_d", "0");

    let mut ctx = ModelContext::new();
    ctx.temperature_kelvin = 300.0;
    ctx.set_diode_model("D1", DiodeModel::default());

    // Auto-place components from schematic onto die
    dialog.auto_place_components(&graph);
    let names: Vec<String> = dialog.mesh.components.iter().map(|c| c.name.clone()).collect();
    assert!(names.contains(&"R1".to_string()));
    assert!(names.contains(&"D1".to_string()));

    // Configure simulation duration
    dialog.t_stop_s = 0.004;
    dialog.dt_s = 0.0005;

    // Run simulation
    dialog.run_simulation(&graph, &ctx);

    assert!(dialog.trajectory.is_some());
    let traj = dialog.trajectory.as_ref().unwrap();
    assert!(!traj.is_empty());
    assert!(!traj.runaway_detected);

    let total_steps = traj.len();
    assert!(total_steps >= 8);

    // Test timeline scrubber across different time indices
    dialog.selected_step_idx = 0;
    let rec_t0 = &traj.records[dialog.selected_step_idx];
    assert_eq!(rec_t0.time_s, 0.0);

    let mid_idx = total_steps / 2;
    dialog.selected_step_idx = mid_idx;
    let rec_mid = &traj.records[dialog.selected_step_idx];
    assert!(rec_mid.time_s > 0.0);

    let last_idx = total_steps - 1;
    dialog.selected_step_idx = last_idx;
    let rec_last = &traj.records[dialog.selected_step_idx];
    assert!(rec_last.peak_temp_k >= rec_t0.peak_temp_k);
}

#[test]
fn test_contour_and_colormap_selection() {
    let mut dialog = ThermalFloorplanDialog::new();

    // Verify colormap switching
    dialog.colormap = Colormap::Magma;
    assert_eq!(dialog.colormap, Colormap::Magma);
    dialog.colormap = Colormap::Inferno;
    assert_eq!(dialog.colormap, Colormap::Inferno);
    dialog.colormap = Colormap::Turbo;
    assert_eq!(dialog.colormap, Colormap::Turbo);

    // Verify contour interval count adjustment
    dialog.contour_count = 8;
    assert_eq!(dialog.contour_count, 8);

    // Verify visual display toggles
    dialog.show_heatmap = false;
    assert!(!dialog.show_heatmap);
    dialog.show_contours = false;
    assert!(!dialog.show_contours);
    dialog.show_components = false;
    assert!(!dialog.show_components);
    dialog.show_hotspot_reticle = false;
    assert!(!dialog.show_hotspot_reticle);

    dialog.show_heatmap = true;
    dialog.show_contours = true;
    dialog.show_components = true;
    dialog.show_hotspot_reticle = true;
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = ThermalFloorplanDialog::new();
    dialog.is_open = true;

    // Headless egui Context execution pass
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    output.textures_delta.clear();

    assert!(dialog.is_open);
    assert!(dialog.trajectory.is_some());
    assert!(!dialog.mesh.temperatures.is_empty());
}
