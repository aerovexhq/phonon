#![deny(unsafe_code)]

//! GUI test suite for the Non-Hermitian Higher-Order Topological Corner Laser Dialog.

use egui::Context;
use phonon_gui::widgets::corner_laser_dialog::{CornerLaserDialog, CornerLaserTab};
use phonon_solver::non_hermitian_corner_laser::CornerLaserLatticeKind;

#[test]
fn test_corner_laser_dialog_initialization() {
    let dialog = CornerLaserDialog::default();
    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, CornerLaserTab::SpatialIntensityCanvas);
    assert_eq!(dialog.intracell_coupling_mhz, 2.5);
    assert_eq!(dialog.intercell_coupling_mhz, 10.0);
    assert_eq!(dialog.corner_gain_mhz, 1.8);
    assert_eq!(dialog.bulk_loss_mhz, 1.2);
    assert_eq!(dialog.pump_current_ma, 15.0);
    assert_eq!(dialog.lattice_size_x, 4);
    assert_eq!(dialog.lattice_size_y, 4);
    assert!(!dialog.defect_active);
    assert_eq!(dialog.lattice_kind, CornerLaserLatticeKind::QuadrupoleCornerLaser);

    // Verify fast-path pre-seeded metrics
    assert_eq!(dialog.engine.metrics.quantized_quadrupole_moment, 0.50);
    assert!(dialog.engine.metrics.side_mode_suppression_ratio_db >= 32.0);
    assert!(dialog.engine.metrics.corner_confinement_ratio_percent >= 85.0);
}

#[test]
fn test_corner_laser_dialog_presets_and_recompute() {
    let mut dialog = CornerLaserDialog::default();

    // Preset: Topological Corner Laser
    dialog.intracell_coupling_mhz = 2.5;
    dialog.intercell_coupling_mhz = 10.0;
    dialog.corner_gain_mhz = 1.8;
    dialog.bulk_loss_mhz = 1.2;
    dialog.defect_active = false;
    dialog.recompute();

    assert_eq!(dialog.engine.metrics.quantized_quadrupole_moment, 0.50);
    assert!(dialog.engine.metrics.active_lasing_mode_count >= 1);
    assert!(dialog.engine.metrics.side_mode_suppression_ratio_db >= 30.0);
    assert!(!dialog.cached_corner_modes.is_empty());
    assert!(!dialog.cached_complex_spectrum.is_empty());
    assert!(!dialog.cached_li_curve.is_empty());

    // Preset: Trivial Bulk Insulator
    dialog.intracell_coupling_mhz = 10.0;
    dialog.intercell_coupling_mhz = 2.5;
    dialog.recompute();
    assert_eq!(dialog.engine.metrics.quantized_quadrupole_moment, 0.0);

    // Preset: Exceptional Point
    dialog.intracell_coupling_mhz = 5.0;
    dialog.intercell_coupling_mhz = 5.0;
    dialog.corner_gain_mhz = 2.5;
    dialog.bulk_loss_mhz = 2.5;
    dialog.recompute();
    assert!(!dialog.cached_ep_sweep.is_empty());
}

#[test]
fn test_corner_laser_dialog_headless_render() {
    let ctx = Context::default();
    let mut dialog = CornerLaserDialog::default();
    dialog.is_open = true;

    let tabs = [
        CornerLaserTab::SpatialIntensityCanvas,
        CornerLaserTab::ComplexEigenvalueSpectrum,
        CornerLaserTab::LightCurrentCurve,
        CornerLaserTab::SmsrSpectrum,
        CornerLaserTab::ExceptionalPointSweep,
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
