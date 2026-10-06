#![deny(unsafe_code)]

//! GUI test suite for Phase 406: Multi-Octave Acoustic Metasurface Wavefront Hologram
//! & Ultrasonic Tractor Beam CAD Studio Dialog.

use egui::Context;
use phonon_gui::widgets::metasurface_hologram_dialog::{
    MetasurfaceHologramDialog, MetasurfaceHologramTab,
};
use std::time::Instant;

#[test]
fn test_metasurface_hologram_dialog_initialization() {
    let dialog = MetasurfaceHologramDialog::default();
    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        MetasurfaceHologramTab::MetasurfacePhaseProfile
    );
    assert_eq!(dialog.grid_size, 16);
    assert_eq!(dialog.operating_frequency_hz, 40_000.0);
    assert_eq!(dialog.focal_plane_z_mm, 50.0);
    assert_eq!(dialog.topological_charge, 1);

    // Verify 10/10 audit pass on defaults
    assert_eq!(dialog.cached_audit_report.pass_count, 10);
    assert_eq!(dialog.cached_audit_report.total_tests, 10);
    assert!(dialog.cached_audit_report.all_passed);

    // Verify cached fields are populated
    assert!(dialog.cached_hologram_result.final_psnr_db >= 25.0);
    assert!(dialog.cached_trap_metrics.axial_pulling_force_n < 0.0);
    assert!(dialog.cached_trap_metrics.is_3d_stable);
    assert!(!dialog.cached_axial_force_profile.is_empty());
    assert!(!dialog.cached_bessel_result.radial_profile.is_empty());
    assert!(!dialog.cached_airy_trajectory.is_empty());
}

#[test]
fn test_metasurface_hologram_dialog_tab_switching() {
    let mut dialog = MetasurfaceHologramDialog::default();
    let tabs = [
        MetasurfaceHologramTab::MetasurfacePhaseProfile,
        MetasurfaceHologramTab::HolographicFocalField,
        MetasurfaceHologramTab::UltrasonicTractorBeam,
        MetasurfaceHologramTab::BesselAiryVortexBeams,
        MetasurfaceHologramTab::PhysicsAuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
    }
}

#[test]
fn test_metasurface_hologram_dialog_parameter_adjustment() {
    let mut dialog = MetasurfaceHologramDialog::default();

    dialog.operating_frequency_hz = 50_000.0;
    dialog.focal_plane_z_mm = 60.0;
    dialog.topological_charge = 2;
    dialog.particle_radius_um = 150.0;
    dialog.recompute();

    assert_eq!(dialog.processor.unit_cell.params.base_frequency_hz, 50_000.0);
    assert_eq!(dialog.processor.gs_params.focal_plane_z_m, 0.060);
    assert_eq!(dialog.processor.bessel_params.topological_charge, 2);
    assert!(dialog.cached_hologram_result.final_psnr_db >= 25.0);
    assert!(dialog.cached_trap_metrics.axial_pulling_force_n < 0.0);
}

#[test]
fn test_metasurface_hologram_dialog_cold_boot_latency() {
    let start = Instant::now();
    let dialog = MetasurfaceHologramDialog::new_fast();
    let elapsed = start.elapsed();

    // Cold boot latency must be < 2ms (2000 microseconds)
    assert!(
        elapsed.as_millis() < 2,
        "Cold boot took {:?}, expected < 2ms",
        elapsed
    );
    assert!(dialog.last_solve_time_us < 2000.0);
}

#[test]
fn test_metasurface_hologram_dialog_headless_render() {
    let ctx = Context::default();
    let mut dialog = MetasurfaceHologramDialog::default();
    dialog.is_open = true;

    let tabs = [
        MetasurfaceHologramTab::MetasurfacePhaseProfile,
        MetasurfaceHologramTab::HolographicFocalField,
        MetasurfaceHologramTab::UltrasonicTractorBeam,
        MetasurfaceHologramTab::BesselAiryVortexBeams,
        MetasurfaceHologramTab::PhysicsAuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.render_content(ui);
        });
        out.textures_delta.clear();
    }

    let mut out_window = ctx.run_ui(Default::default(), |ui| {
        dialog.ui(ui.ctx());
    });
    out_window.textures_delta.clear();
}
