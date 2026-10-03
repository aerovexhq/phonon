#![deny(unsafe_code)]

//! GUI test suite for Phase 348: LiebLatticeDialog in Phonon CAD Studio.
//!
//! Verifies:
//! - LiebLatticeDialog initialization, defaults, and cache setup.
//! - Preset switching (Caging Phi=pi, Standard Phi=0, CLS) and recomputation.
//! - Gauge flux slider adjustment and caging curve generation.
//! - Mode selection and plot tab switching.
//! - Headless egui Context execution pass and plot generation.

use std::f64::consts::PI;
use phonon_gui::widgets::lieb_lattice_dialog::{
    LiebLatticeDialog, LiebPlotTab, LiebSpatialModeSelection,
};

#[test]
fn test_lieb_lattice_dialog_initialization_and_defaults() {
    let dialog = LiebLatticeDialog::new();

    // Dialog must be closed by default
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Physical parameter defaults
    assert_eq!(dialog.j_mhz, 5.0);
    assert_eq!(dialog.omega_0_ghz, 1.0);
    assert_eq!(dialog.a_mm, 10.0);
    assert_eq!(dialog.phi_flux, 0.0);
    assert_eq!(dialog.delta_site, 0.0);
    assert_eq!(dialog.nx, 4);
    assert_eq!(dialog.ny, 4);

    // Lattice result check
    assert_eq!(dialog.lattice.total_sites(), 48); // 3 * 4 * 4
    assert!(dialog.current_cls.is_some(), "Center CLS must be constructed");
    assert_eq!(dialog.current_cls.as_ref().unwrap().energy, 0.0);
    assert_eq!(dialog.current_cls.as_ref().unwrap().confinement_ratio, 1.0);

    // Dispersion curves populated
    assert!(!dialog.band_curve_upper.is_empty());
    assert!(!dialog.band_curve_flat.is_empty());
    assert!(!dialog.band_curve_lower.is_empty());

    // Caging curves populated
    assert!(!dialog.caging_curve.is_empty());
    assert!(!dialog.time_dynamics_current.is_empty());
    assert!(!dialog.time_dynamics_caged.is_empty());
}

#[test]
fn test_preset_switching_and_recomputation() {
    let mut dialog = LiebLatticeDialog::new();
    assert_eq!(dialog.phi_flux, 0.0);

    // 1. Switch to Aharonov-Bohm Caging (Phi = PI) preset
    dialog.apply_preset_caging();
    assert!((dialog.phi_flux - PI).abs() < 1e-12);
    assert_eq!(
        dialog.mode_view,
        LiebSpatialModeSelection::TimeEvolvedWavepacket
    );
    assert_eq!(dialog.active_plot_tab, LiebPlotTab::CagingCurve);
    assert!(dialog.status_msg.contains("Phi = 1.00 pi"));

    // 2. Switch to Compact Localized State (CLS) preset
    dialog.apply_preset_cls();
    assert_eq!(dialog.phi_flux, 0.0);
    assert_eq!(
        dialog.mode_view,
        LiebSpatialModeSelection::SinglePlaquetteCls
    );
    assert_eq!(dialog.active_plot_tab, LiebPlotTab::BandDispersion);

    // 3. Switch to Standard Lieb Lattice preset
    dialog.apply_preset_standard();
    assert_eq!(dialog.phi_flux, 0.0);
    assert_eq!(dialog.active_plot_tab, LiebPlotTab::BandDispersion);
}

#[test]
fn test_gauge_flux_slider_adjustment_and_caging_curve_generation() {
    let mut dialog = LiebLatticeDialog::new();

    // Change flux to 0.5 * PI
    dialog.phi_flux = 0.5 * PI;
    dialog.recompute();

    assert!((dialog.phi_flux - 0.5 * PI).abs() < 1e-12);
    assert!(!dialog.caging_curve.is_empty());

    // Verify peak around phi = pi in caging curve
    let pi_sample = dialog
        .caging_curve
        .iter()
        .find(|[p, _]| (*p - 1.0).abs() < 0.1);
    assert!(pi_sample.is_some(), "Caging curve must contain sample near Phi = pi");
}

#[test]
fn test_mode_and_plot_tab_selection() {
    let mut dialog = LiebLatticeDialog::new();

    // Mode view labels
    dialog.mode_view = LiebSpatialModeSelection::SinglePlaquetteCls;
    assert_eq!(dialog.mode_view.label(), "Single-Plaquette CLS Mode");

    dialog.mode_view = LiebSpatialModeSelection::TimeEvolvedWavepacket;
    assert_eq!(
        dialog.mode_view.label(),
        "Time-Evolved Wavepacket |psi(t)|^2"
    );

    dialog.mode_view = LiebSpatialModeSelection::AllPlaquettesCls;
    assert_eq!(dialog.mode_view.label(), "Combined All Plaquettes CLS");

    // Plot tabs
    dialog.active_plot_tab = LiebPlotTab::BandDispersion;
    assert_eq!(dialog.active_plot_tab.label(), "3-Band Dispersion");

    dialog.active_plot_tab = LiebPlotTab::CagingCurve;
    assert_eq!(dialog.active_plot_tab.label(), "AB Caging vs Flux");

    dialog.active_plot_tab = LiebPlotTab::TimeDynamics;
    assert_eq!(dialog.active_plot_tab.label(), "Wavepacket Dynamics");
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = LiebLatticeDialog::new();
    dialog.is_open = true;

    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    output.textures_delta.clear();

    assert!(dialog.is_open, "Dialog should remain open after render pass");
    assert!(!dialog.band_curve_upper.is_empty());
    assert!(!dialog.caging_curve.is_empty());
}
