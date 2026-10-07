#![deny(unsafe_code)]

//! Automated GUI test suite for Phase 414: Phonon Studio Non-Hermitian Higher-Order
//! Topological Quadrupole Skin-Effect Laser & Chiral Edge Emitter Dialog.

use egui::Context;
use phonon_gui::widgets::{NonHermitianSkinLaserDialog, SkinLaserTab};

#[test]
fn test_dialog_initialization_and_fast_boot() {
    let dialog = NonHermitianSkinLaserDialog::new_fast();
    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, SkinLaserTab::SkinLattice);
    assert_eq!(dialog.nx, 6);
    assert_eq!(dialog.ny, 6);
    assert!((dialog.skin_asymmetry_g - 0.85).abs() < 1e-6);

    // Verify precomputed caches
    assert!(!dialog.cached_spatial_profile.is_empty());
    assert!(!dialog.cached_complex_spectrum.is_empty());
    assert!(!dialog.cached_laser_curve.is_empty());
    assert!(!dialog.cached_radiation_pattern.is_empty());
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_tab_switching() {
    let mut dialog = NonHermitianSkinLaserDialog::new();
    let tabs = [
        SkinLaserTab::SkinLattice,
        SkinLaserTab::ComplexSpectrum,
        SkinLaserTab::TopologicalLaser,
        SkinLaserTab::ChiralEmitter,
        SkinLaserTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_parameter_adjustment_and_recomputation() {
    let mut dialog = NonHermitianSkinLaserDialog::new();

    // Adjust skin asymmetry
    dialog.skin_asymmetry_g = 1.2;
    dialog.pump_rate_mhz = 4.5;
    dialog.directivity_db = 32.0;
    dialog.defect_present = true;
    dialog.recompute();

    assert!((dialog.cached_skin_metrics.asymmetry_ratio - 1.2f64.exp()).abs() < 1e-6);
    assert!(dialog.cached_laser_metrics.is_lasing);
    assert!(dialog.cached_emitter_metrics.defect_transmission_ratio < 1.0);
    assert!(dialog.last_solve_time_us > 0.0);
}

#[test]
fn test_preset_switching() {
    let mut dialog = NonHermitianSkinLaserDialog::new();

    // Preset 2: Symmetric BBH (no skin effect)
    dialog.skin_asymmetry_g = 0.0;
    dialog.recompute();
    assert_eq!(dialog.cached_skin_metrics.point_gap_winding, 0);
    assert!((dialog.cached_skin_metrics.gbz_radius - 1.0).abs() < 1e-6);

    // Preset 1: Topological skin laser
    dialog.skin_asymmetry_g = 0.85;
    dialog.recompute();
    assert_eq!(dialog.cached_skin_metrics.point_gap_winding, 1);
    assert!(dialog.cached_skin_metrics.gbz_radius < 1.0);
}

#[test]
fn test_headless_egui_render_pass() {
    let ctx = Context::default();
    let mut dialog = NonHermitianSkinLaserDialog::new();
    dialog.is_open = true;

    // Render across all 5 tabs in headless mode
    let tabs = [
        SkinLaserTab::SkinLattice,
        SkinLaserTab::ComplexSpectrum,
        SkinLaserTab::TopologicalLaser,
        SkinLaserTab::ChiralEmitter,
        SkinLaserTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            ui.set_min_size(egui::vec2(1000.0, 700.0));
            dialog.render_content(ui);
        });
        output.textures_delta.clear();
        assert_eq!(dialog.active_tab, tab);
    }

    assert!(dialog.is_open);
}
