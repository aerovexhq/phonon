#![deny(unsafe_code)]

//! Verification test suite for Phase 333: Neuromorphic SNN Studio Dialog.
//!
//! Tests dialog initialization, default physical parameters, crossbar layout,
//! SNN simulation execution, trajectory caching, and headless egui rendering.

use phonon_gui::widgets::neuromorphic_snn_dialog::NeuromorphicSnnDialog;

#[test]
fn test_neuromorphic_snn_dialog_initialization_defaults() {
    let dialog = NeuromorphicSnnDialog::new();

    // Dialog should be closed by default
    assert!(!dialog.is_open, "Dialog should be closed by default");

    // Network size defaults
    assert_eq!(dialog.num_inputs, 8);
    assert_eq!(dialog.num_outputs, 4);

    // STDP parameter defaults
    assert_eq!(dialog.stdp_params.a_plus, 0.05);
    assert_eq!(dialog.stdp_params.a_minus, 0.025);
    assert_eq!(dialog.stdp_params.tau_plus, 0.020);
    assert_eq!(dialog.stdp_params.tau_minus, 0.020);

    // Initial pre-simulated trajectory must be present
    assert!(dialog.trajectory.is_some(), "Initial trajectory should be pre-computed");
    let traj = dialog.trajectory.as_ref().unwrap();
    assert!(!traj.timestamps.is_empty());
    assert_eq!(traj.membrane_potentials.len(), 4);
    assert_eq!(traj.initial_conductances.len(), 8);
    assert_eq!(traj.initial_conductances[0].len(), 4);
    assert_eq!(traj.final_conductances.len(), 8);
    assert_eq!(traj.final_conductances[0].len(), 4);

    // Synaptic operation telemetry must be non-zero
    assert!(traj.total_synaptic_ops > 0);
    assert!(traj.energy_per_sop_fj > 0.0);
    assert!(traj.mean_conductance_s > 0.0);
}

#[test]
fn test_simulation_execution_and_trajectory() {
    let mut dialog = NeuromorphicSnnDialog::new();

    // 1. Run standard simulation
    dialog.run_simulation();
    let traj = dialog.trajectory.as_ref().expect("Trajectory must exist");
    assert!(!traj.timestamps.is_empty());
    assert_eq!(traj.output_spike_rates_hz.len(), 4);

    // 2. Apply pattern stimulus
    dialog.apply_pattern_stimulus();
    assert!(dialog.status_msg.contains("Pattern stimulus applied"));
    let traj_pat = dialog.trajectory.as_ref().expect("Trajectory must exist");
    assert!(!traj_pat.input_spikes.is_empty());

    // 3. Reset weights
    dialog.reset_weights();
    assert!(dialog.status_msg.contains("reset to baseline"));
    let traj_reset = dialog.trajectory.as_ref().expect("Trajectory must exist");
    assert_eq!(traj_reset.final_conductances.len(), 8);
}

#[test]
fn test_crossbar_layout_and_presets() {
    let mut dialog = NeuromorphicSnnDialog::new();

    // Resize crossbar array to 16x8
    dialog.num_inputs = 16;
    dialog.num_outputs = 8;
    dialog.rebuild_network_and_run(true);

    assert_eq!(dialog.network.num_inputs, 16);
    assert_eq!(dialog.network.num_outputs, 8);
    assert_eq!(dialog.network.crossbar.len(), 16);
    assert_eq!(dialog.network.crossbar[0].len(), 8);

    let traj = dialog.trajectory.as_ref().expect("Trajectory must exist");
    assert_eq!(traj.membrane_potentials.len(), 8);
    assert_eq!(traj.output_spike_rates_hz.len(), 8);
    assert_eq!(traj.final_conductances.len(), 16);
    assert_eq!(traj.final_conductances[0].len(), 8);
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = NeuromorphicSnnDialog::new();
    dialog.is_open = true;

    // Headless egui Context execution pass
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    output.textures_delta.clear();

    assert!(dialog.is_open);
    assert!(dialog.trajectory.is_some());
}
