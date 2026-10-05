#![deny(unsafe_code)]

//! GUI test suite for Phase 358: QuantumBraidingLatticeDialog in Phonon CAD Studio.
//!
//! Verifies:
//! - QuantumBraidingLatticeDialog initialization and defaults.
//! - Braid sequence compilation and trajectory state stepping in GUI.
//! - Surface code error injection, syndrome detection, and MWPM correction.
//! - Headless egui Context render pass and view tabs execution.

use phonon_gui::widgets::quantum_braiding_lattice_dialog::{
    BraidingDialogTab, QuantumBraidingLatticeDialog,
};
use phonon_solver::protected_braiding_lattice::{BraidStep, TargetGate};

#[test]
fn test_quantum_braiding_lattice_dialog_initialization_and_defaults() {
    let dialog = QuantumBraidingLatticeDialog::new();

    // Dialog must be closed initially
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Default tab
    assert_eq!(dialog.active_tab, BraidingDialogTab::StudioOverview);
    assert_eq!(dialog.active_tab.label(), "Studio Overview");

    // Physical parameter defaults
    assert_eq!(dialog.params.qubit_count, 1);
    assert!((dialog.params.topological_gap_mhz - 2.5).abs() < 1e-6);
    assert!((dialog.params.braid_duration_ns - 120.0).abs() < 1e-6);
    assert!((dialog.params.waveguide_length_um - 10.0).abs() < 1e-6);
    assert!((dialog.params.dispersive_shift_mhz - 4.0).abs() < 1e-6);
    assert!((dialog.params.cavity_linewidth_mhz - 0.5).abs() < 1e-6);

    // Initial compiled target gate is Hadamard
    assert_eq!(dialog.target_gate, TargetGate::Hadamard);
    assert_eq!(dialog.compiled_result.target_gate, TargetGate::Hadamard);
    assert_eq!(
        dialog.compiled_result.braid_word,
        vec![BraidStep::B1, BraidStep::B2, BraidStep::B1]
    );
    assert!(
        dialog.compiled_result.net_gate_fidelity >= 0.999,
        "Compiled gate fidelity must be >= 0.999"
    );

    // Parity readout initial checks
    assert!(
        dialog.parity_readout.snr_db() >= 15.0,
        "Parity readout SNR must be >= 15.0 dB"
    );
    assert!(
        dialog.parity_readout.readout_fidelity() >= 0.995,
        "Parity readout fidelity must be >= 0.995"
    );

    // Surface code initial check
    assert_eq!(dialog.surface_grid.data_positions.len(), 9);
    assert_eq!(dialog.surface_grid.star_stabilizers.len(), 4);
    assert_eq!(dialog.surface_grid.plaquette_stabilizers.len(), 4);
}

#[test]
fn test_braid_sequence_compilation_and_state_stepping() {
    let mut dialog = QuantumBraidingLatticeDialog::new();

    // 1. Compile Phase S gate
    dialog.target_gate = TargetGate::PhaseS;
    dialog.compile_current_gate();
    assert_eq!(dialog.compiled_result.target_gate, TargetGate::PhaseS);
    assert_eq!(dialog.compiled_result.braid_word, vec![BraidStep::B3]);
    assert!(dialog.compiled_result.net_gate_fidelity >= 0.999);

    // 2. Compile Pauli X gate
    dialog.target_gate = TargetGate::PauliX;
    dialog.compile_current_gate();
    assert_eq!(dialog.compiled_result.target_gate, TargetGate::PauliX);
    assert_eq!(
        dialog.compiled_result.braid_word,
        vec![BraidStep::B2, BraidStep::B2]
    );
    assert!(dialog.compiled_result.net_gate_fidelity >= 0.999);

    // 3. Compile Pauli Z gate
    dialog.target_gate = TargetGate::PauliZ;
    dialog.compile_current_gate();
    assert_eq!(dialog.compiled_result.target_gate, TargetGate::PauliZ);
    assert_eq!(
        dialog.compiled_result.braid_word,
        vec![BraidStep::B1, BraidStep::B1]
    );
    assert!(dialog.compiled_result.net_gate_fidelity >= 0.999);

    // 4. Compile CNOT gate on multi-qubit register
    dialog.params.qubit_count = 2;
    dialog.target_gate = TargetGate::Cnot;
    dialog.compile_current_gate();
    assert_eq!(dialog.compiled_result.target_gate, TargetGate::Cnot);
    assert!(dialog.compiled_result.braid_word.len() >= 3);
    assert!(dialog.compiled_result.net_gate_fidelity >= 0.999);

    // 5. Test Trajectory State Stepping
    assert_eq!(dialog.anim_step, 0);
    dialog.step_trajectory();
    assert_eq!(dialog.anim_step, 1);
    dialog.step_trajectory();
    assert_eq!(dialog.anim_step, 2);
    dialog.step_trajectory();
    assert_eq!(dialog.anim_step, 3);
    dialog.step_trajectory();
    assert_eq!(dialog.anim_step, 4);
    dialog.step_trajectory();
    assert_eq!(dialog.anim_step, 0); // Cycles back
}

#[test]
fn test_surface_code_error_injection_and_syndrome_correction() {
    let mut dialog = QuantumBraidingLatticeDialog::new();

    // 1. Clean cycle test
    let clean_syn = dialog
        .surface_grid
        .extract_syndromes_from_errors([false; 9], [false; 9], 0.0);
    assert!(clean_syn.is_clean());
    let clean_corr = dialog.surface_grid.decode_and_correct(&clean_syn);
    assert!(clean_corr.is_clean);
    assert!(!clean_corr.has_logical_error);

    // 2. Inject Pauli Z error on data qubit 0
    let mut z_err = [false; 9];
    z_err[0] = true;
    let syn_z0 = dialog
        .surface_grid
        .extract_syndromes_from_errors([false; 9], z_err, 0.01);
    // Qubit 0 touches Star check A_0
    assert_eq!(syn_z0.star_defects, vec![0]);
    let corr_z0 = dialog.surface_grid.decode_and_correct(&syn_z0);
    assert!(corr_z0.is_clean);
    assert!(!corr_z0.has_logical_error);
    assert!(corr_z0.correction_z[0]);

    // 3. Inject Pauli X error on data qubit 4 (center)
    let mut x_err = [false; 9];
    x_err[4] = true;
    let syn_x4 = dialog
        .surface_grid
        .extract_syndromes_from_errors(x_err, [false; 9], 0.01);
    // Qubit 4 touches Plaquette checks B_0 and B_3
    assert_eq!(syn_x4.plaquette_defects, vec![0, 3]);
    let corr_x4 = dialog.surface_grid.decode_and_correct(&syn_x4);
    assert!(corr_x4.is_clean);
    assert!(!corr_x4.has_logical_error);
    assert!(corr_x4.correction_x[4]);

    // 4. Run syndrome cycle in GUI
    dialog.physical_error_rate = 0.005;
    dialog.run_syndrome_cycle();
    assert!(dialog.logical_error_rate_cache < 0.05);
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = QuantumBraidingLatticeDialog::new();
    dialog.is_open = true;

    let ctx = egui::Context::default();

    // Test render pass on default StudioOverview tab
    let mut output = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    output.textures_delta.clear();
    assert!(dialog.is_open);

    // Test render pass across all individual tabs
    for tab in [
        BraidingDialogTab::PlanarLattice,
        BraidingDialogTab::SurfaceCode,
        BraidingDialogTab::ParitySpectrum,
        BraidingDialogTab::BraidCompiler,
    ] {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.render_content(ui);
        });
        out.textures_delta.clear();
        assert_eq!(dialog.active_tab, tab);
    }
}
