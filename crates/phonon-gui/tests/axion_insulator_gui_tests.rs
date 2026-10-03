#![deny(unsafe_code)]

//! GUI test suite for Phase 349: AxionInsulatorDialog in Phonon CAD Studio.
//!
//! Verifies:
//! - AxionInsulatorDialog initialization, default parameters, and cache population.
//! - Axion preset switching (Topological theta = PI vs Trivial theta = 0) and recomputation.
//! - Parameter adjustment (mass M_0, hopping t, surface mass Delta_surf) in GUI state.
//! - Headless egui Context execution pass and plot generation.

use std::f64::consts::PI;
use phonon_gui::widgets::axion_insulator_dialog::{
    AxionInsulatorDialog, AxionPlotTab, AxionSpatialViewMode,
};

#[test]
fn test_axion_insulator_dialog_initialization_and_defaults() {
    let dialog = AxionInsulatorDialog::new();

    // Dialog must be closed initially
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Physical parameter defaults
    assert_eq!(dialog.t_hop_mhz, 5.0);
    assert_eq!(dialog.m_0_mhz, 10.0);
    assert_eq!(dialog.v_dirac_mhz_mm, 15.0);
    assert!((dialog.theta_rad - PI).abs() < 1e-12);
    assert_eq!(dialog.delta_surf_mhz, 2.0);
    assert_eq!(dialog.a_mm, 5.0);
    assert_eq!(dialog.omega_0_ghz, 1.0);
    assert_eq!(dialog.nx, 5);
    assert_eq!(dialog.ny, 5);

    // Initial state must be topological
    assert!(dialog.rod_result.is_topological);
    assert_eq!(dialog.rod_result.hinge_modes.len(), 4);
    assert!(dialog.rod_result.mean_hinge_confinement >= 0.80);
    assert!(dialog.rod_result.directivity_db >= 25.0);

    // Cached curves must be populated
    assert!(!dialog.dispersion_forward_branch.is_empty());
    assert!(!dialog.dispersion_backward_branch.is_empty());
    assert!(!dialog.s21_curve.is_empty());
    assert!(!dialog.s12_curve.is_empty());
    assert!(!dialog.bulk_band_curves[0].is_empty());
}

#[test]
fn test_axion_preset_switching_and_recomputation() {
    let mut dialog = AxionInsulatorDialog::new();
    assert!(dialog.rod_result.is_topological);

    // 1. Switch to Trivial Insulator preset (theta = 0)
    dialog.set_trivial_preset();
    assert_eq!(dialog.theta_rad, 0.0);
    assert_eq!(dialog.m_0_mhz, 35.0);
    assert_eq!(dialog.delta_surf_mhz, 0.0);
    assert!(!dialog.rod_result.is_topological);
    assert_eq!(dialog.rod_result.surface_bandgap_mhz, 0.0);
    assert!(dialog.status_msg.contains("Trivial Insulator"));

    // 2. Switch back to Topological Axion Insulator preset (theta = PI)
    dialog.set_topological_preset();
    assert!((dialog.theta_rad - PI).abs() < 1e-12);
    assert_eq!(dialog.m_0_mhz, 10.0);
    assert_eq!(dialog.delta_surf_mhz, 2.0);
    assert!(dialog.rod_result.is_topological);
    assert!(dialog.rod_result.mean_hinge_confinement >= 0.80);
    assert!(dialog.status_msg.contains("Topological Axion Insulator"));
}

#[test]
fn test_parameter_adjustment_in_gui_state() {
    let mut dialog = AxionInsulatorDialog::new();

    // Adjust hopping amplitude and mass
    dialog.t_hop_mhz = 6.0;
    dialog.m_0_mhz = 12.0;
    dialog.delta_surf_mhz = 2.5;
    dialog.recompute();

    assert_eq!(dialog.params.t_hop, 6.0);
    assert_eq!(dialog.params.m_0, 12.0);
    assert_eq!(dialog.params.delta_surf, 2.5);
    assert!(dialog.rod_result.is_topological);

    // Mode and tab switches
    dialog.spatial_view_mode = AxionSpatialViewMode::Hinge1BottomLeft;
    assert_eq!(dialog.spatial_view_mode.label(), "Hinge 1 (BL: +z forward)");

    dialog.active_plot_tab = AxionPlotTab::NonReciprocalSParameters;
    assert_eq!(dialog.active_plot_tab.label(), "Non-Reciprocal S-Parameters");

    dialog.active_plot_tab = AxionPlotTab::SurfaceHallDomainWalls;
    assert_eq!(dialog.active_plot_tab.label(), "Surface Hall Domain Walls");

    dialog.active_plot_tab = AxionPlotTab::BulkBandStructure;
    assert_eq!(dialog.active_plot_tab.label(), "3D Bulk Band Structure");
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = AxionInsulatorDialog::new();
    dialog.is_open = true;

    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    output.textures_delta.clear();

    assert!(dialog.is_open, "Dialog should remain open after render pass");
    assert!(!dialog.dispersion_forward_branch.is_empty());
    assert!(!dialog.s21_curve.is_empty());
}
