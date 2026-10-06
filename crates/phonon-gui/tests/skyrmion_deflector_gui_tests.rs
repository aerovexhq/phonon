#![deny(unsafe_code)]

//! GUI test suite for the Acoustic Higher-Order Skyrmion-Lattice Beam Deflector & Chiral Router Dialog.

use egui::Context;
use phonon_gui::widgets::skyrmion_deflector_dialog::{SkyrmionDeflectorDialog, SkyrmionDeflectorTab};
use phonon_solver::skyrmion_deflector::{AcousticPseudoSpin, SkyrmionProfileKind};

#[test]
fn test_skyrmion_deflector_dialog_initialization() {
    let dialog = SkyrmionDeflectorDialog::default();
    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, SkyrmionDeflectorTab::RealSpaceTexture);
    assert_eq!(dialog.pseudo_spin, AcousticPseudoSpin::SpinUp);
    assert_eq!(dialog.vorticity, 1);
    assert_eq!(dialog.frequency_khz, 5.0);
    assert!(dialog.cached_beam.deflection_angle_deg > 5.0);
    assert!(dialog.engine.metrics.s21_spin_up_db >= -1.0);
    assert!(dialog.engine.metrics.s31_spin_up_isolation_db <= -28.0);
}

#[test]
fn test_skyrmion_deflector_dialog_presets() {
    let mut dialog = SkyrmionDeflectorDialog::default();

    // Preset: Higher-Order (N_sk = 2)
    dialog.vorticity = 2;
    dialog.profile_kind = SkyrmionProfileKind::HigherOrder;
    dialog.recompute();
    assert!(dialog.cached_beam.deflection_angle_deg > 30.0);
    assert_eq!(dialog.vorticity, 2);

    // Preset: Antiskyrmion (N_sk = -1)
    dialog.vorticity = -1;
    dialog.profile_kind = SkyrmionProfileKind::Antiskyrmion;
    dialog.recompute();
    assert!(dialog.cached_beam.deflection_angle_deg < -5.0);

    // Preset: Trivial Ferromagnet (N_sk = 0)
    dialog.vorticity = 0;
    dialog.profile_kind = SkyrmionProfileKind::TrivialFerromagnet;
    dialog.recompute();
    assert_eq!(dialog.cached_beam.deflection_angle_deg, 0.0);
}

#[test]
fn test_skyrmion_deflector_dialog_headless_render() {
    let ctx = Context::default();
    let mut dialog = SkyrmionDeflectorDialog::default();
    dialog.is_open = true;

    let tabs = [
        SkyrmionDeflectorTab::RealSpaceTexture,
        SkyrmionDeflectorTab::AnomalousHallDeflection,
        SkyrmionDeflectorTab::TopologicalDensitySweep,
        SkyrmionDeflectorTab::MultiPortSParameters,
        SkyrmionDeflectorTab::DefectImmunity,
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

    assert!(dialog.is_open);
}
