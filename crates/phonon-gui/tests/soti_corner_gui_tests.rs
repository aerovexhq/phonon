#![deny(unsafe_code)]

//! Test suite for Phase 347: SotiCornerDialog in Phonon CAD Studio.
//!
//! Verifies:
//! - SotiCornerDialog initialization and default parameters.
//! - Preset switching (Topological SOTI vs Trivial Insulator) and recomputation.
//! - Disorder/defect toggle in GUI state.
//! - Headless egui Context execution pass and plot generation.

use phonon_gui::widgets::soti_corner_dialog::{SotiCornerDialog, SpatialModeSelection};

#[test]
fn test_soti_corner_dialog_initialization_and_defaults() {
    let dialog = SotiCornerDialog::new();

    // Dialog must be closed by default
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Default physical parameters
    assert_eq!(dialog.gamma_mhz, 2.0);
    assert_eq!(dialog.lambda_mhz, 10.0);
    assert_eq!(dialog.omega_0_ghz, 1.0);
    assert_eq!(dialog.a_mm, 5.0);
    assert_eq!(dialog.nx, 5);
    assert_eq!(dialog.ny, 5);

    // Initial topological SOTI state
    assert!(dialog.lattice_result.is_topological);
    assert_eq!(dialog.lattice_result.corner_states.len(), 4);
    assert!(dialog.lattice_result.energy_confinement_ratio >= 0.80);
    assert!(dialog.lattice_result.quality_factor >= 1.0e4);

    // Cached curves verification
    assert!(!dialog.band_curve_1.is_empty(), "Band curve 1 must be populated");
    assert!(!dialog.band_curve_2.is_empty(), "Band curve 2 must be populated");
    assert!(!dialog.band_curve_3.is_empty(), "Band curve 3 must be populated");
    assert!(!dialog.band_curve_4.is_empty(), "Band curve 4 must be populated");
    assert!(!dialog.discrete_spectrum_bulk.is_empty(), "Discrete spectrum bulk points must be populated");
    assert_eq!(dialog.discrete_spectrum_corner.len(), 4, "4 discrete corner state markers expected");
}

#[test]
fn test_preset_switching_and_recomputation() {
    let mut dialog = SotiCornerDialog::new();
    assert!(dialog.lattice_result.is_topological);

    // Switch to Trivial Insulator preset
    dialog.set_trivial_preset();
    assert_eq!(dialog.gamma_mhz, 10.0);
    assert_eq!(dialog.lambda_mhz, 2.0);
    assert!(!dialog.lattice_result.is_topological);
    assert_eq!(dialog.lattice_result.corner_states.len(), 0);
    assert!(dialog.lattice_result.localization_length_mm.is_infinite());

    // Switch back to Topological SOTI preset
    dialog.set_topological_preset();
    assert_eq!(dialog.gamma_mhz, 2.0);
    assert_eq!(dialog.lambda_mhz, 10.0);
    assert!(dialog.lattice_result.is_topological);
    assert_eq!(dialog.lattice_result.corner_states.len(), 4);
    assert!(dialog.lattice_result.energy_confinement_ratio >= 0.80);
}

#[test]
fn test_disorder_toggle_in_gui_state() {
    let mut dialog = SotiCornerDialog::new();

    // Enable coupling disorder
    dialog.disorder_enabled = true;
    dialog.disorder_amplitude = 0.5;
    dialog.disorder_seed = 12345;
    dialog.recompute();

    assert!(dialog.disorder_enabled);
    assert_eq!(dialog.lattice_result.corner_states.len(), 4);
    // Pinning near midgap
    let threshold = 0.05 * dialog.lattice_result.bulk_bandgap;
    for cs in &dialog.lattice_result.corner_states {
        assert!(cs.energy.abs() < threshold);
    }
    assert!(dialog.status_msg.contains("Disorder"));

    // Disable disorder
    dialog.disorder_enabled = false;
    dialog.recompute();
    assert!(!dialog.disorder_enabled);
    assert!(dialog.status_msg.contains("Topological SOTI"));
}

#[test]
fn test_mode_selection_and_options() {
    let mut dialog = SotiCornerDialog::new();
    assert_eq!(dialog.mode_view, SpatialModeSelection::CombinedCorners);

    dialog.mode_view = SpatialModeSelection::BottomLeft;
    assert_eq!(dialog.mode_view.label(), "Mode 1: Bottom-Left Corner");

    dialog.mode_view = SpatialModeSelection::BottomRight;
    assert_eq!(dialog.mode_view.label(), "Mode 2: Bottom-Right Corner");

    dialog.mode_view = SpatialModeSelection::TopLeft;
    assert_eq!(dialog.mode_view.label(), "Mode 3: Top-Left Corner");

    dialog.mode_view = SpatialModeSelection::TopRight;
    assert_eq!(dialog.mode_view.label(), "Mode 4: Top-Right Corner");
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = SotiCornerDialog::new();
    dialog.is_open = true;

    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    output.textures_delta.clear();

    assert!(dialog.is_open, "Dialog should remain open after render pass");
    assert!(!dialog.band_curve_1.is_empty());
    assert!(!dialog.discrete_spectrum_bulk.is_empty());
}
