#![deny(unsafe_code)]

//! GUI test suite for Phase 405: Topological Higher-Order Acoustic Quadrupole
//! Corner-Pumped Polariton Laser and PT-Symmetric Metamaterial Dialog.

use egui::Context;
use phonon_gui::widgets::corner_laser_dialog::{CornerLaserDialog, CornerLaserTab};
use std::time::Instant;

#[test]
fn test_corner_laser_dialog_initialization() {
    let dialog = CornerLaserDialog::default();
    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, CornerLaserTab::PtQuadrupoleLattice);
    assert_eq!(dialog.grid_size, 6);
    assert_eq!(dialog.intracell_coupling_gamma_mhz, 2.0);
    assert_eq!(dialog.intercell_coupling_lambda_mhz, 10.0);
    assert_eq!(dialog.gain_loss_strength_mhz, 1.5);
    assert_eq!(dialog.corner_pump_boost, 3.0);
    assert_eq!(dialog.pump_rate_mw, 20.0);
    assert_eq!(dialog.cavity_decay_rate_mhz, 2.5);
    assert_eq!(dialog.gain_coefficient_g0_mhz, 8.0);
    assert_eq!(dialog.spontaneous_beta, 0.05);

    // Verify 10/10 audit pass on defaults
    assert_eq!(dialog.cached_audit_report.pass_count, 10);
    assert_eq!(dialog.cached_audit_report.total_tests, 10);
    assert!(dialog.cached_audit_report.all_passed);
    assert!(!dialog.cached_li_curve.is_empty());
    assert!(!dialog.cached_coherence_curve.is_empty());
    assert!(!dialog.cached_eigenvalues.is_empty());
    assert_eq!(dialog.cached_corner_modes.len(), 4);
}

#[test]
fn test_corner_laser_dialog_tab_switching() {
    let mut dialog = CornerLaserDialog::default();
    let tabs = [
        CornerLaserTab::PtQuadrupoleLattice,
        CornerLaserTab::LaserLiCurve,
        CornerLaserTab::ModeSpectrumSmsr,
        CornerLaserTab::TemporalCoherence,
        CornerLaserTab::PhysicsAuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
    }
}

#[test]
fn test_corner_laser_dialog_parameter_adjustment() {
    let mut dialog = CornerLaserDialog::default();

    // Adjust parameters
    dialog.pump_rate_mw = 30.0;
    dialog.gain_loss_strength_mhz = 2.0;
    dialog.cavity_decay_rate_mhz = 3.0;
    dialog.recompute();

    assert_eq!(dialog.processor.lasing_solver.params.pump_rate_mw, 30.0);
    assert_eq!(dialog.processor.lattice.params.gain_loss_strength_mhz, 2.0);
    assert_eq!(dialog.processor.lasing_solver.params.cavity_decay_rate_mhz, 3.0);
    assert!(!dialog.cached_li_curve.is_empty());
    assert!(!dialog.cached_coherence_curve.is_empty());
}

#[test]
fn test_corner_laser_dialog_cold_boot_latency() {
    let start = Instant::now();
    let dialog = CornerLaserDialog::new_fast();
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
fn test_corner_laser_dialog_headless_render() {
    let ctx = Context::default();
    let mut dialog = CornerLaserDialog::default();
    dialog.is_open = true;

    let tabs = [
        CornerLaserTab::PtQuadrupoleLattice,
        CornerLaserTab::LaserLiCurve,
        CornerLaserTab::ModeSpectrumSmsr,
        CornerLaserTab::TemporalCoherence,
        CornerLaserTab::PhysicsAuditTelemetry,
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
