#![deny(unsafe_code)]

//! GUI Integration and Headless Render Test Suite for Phase 419:
//! Phonon Studio Quantum Metamaterial Topological Acoustic Higher-Order Octupole
//! Vortex Metamaterial & 3D Chiral Dislocation Router.

use egui::Context;
use phonon_gui::widgets::{OctupoleDislocationDialog, OctupoleDislocationTab};

#[test]
fn test_octupole_dislocation_dialog_initialization_and_cold_boot() {
    let dialog = OctupoleDislocationDialog::new_fast();

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, OctupoleDislocationTab::OctupoleMetamaterial);
    assert!(!dialog.cached_dispersion.is_empty());
    assert_eq!(dialog.cached_corner_modes.len(), 8);
    assert_eq!(dialog.cached_dislocation_dispersion.len(), 21);
    assert_eq!(dialog.cached_router_spectrum.len(), 21);
    assert!(!dialog.cached_realspace_slice.is_empty());
    assert_eq!(dialog.cached_audit.total_pass_score, 10);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_octupole_dislocation_dialog_tab_switching() {
    let mut dialog = OctupoleDislocationDialog::new_fast();

    let tabs = [
        (OctupoleDislocationTab::OctupoleMetamaterial, "3D Octupole Metamaterial"),
        (OctupoleDislocationTab::ChiralDislocation, "Chiral Screw Dislocation"),
        (OctupoleDislocationTab::MultiPortRouter, "4-Port Vortex Router"),
        (OctupoleDislocationTab::RealSpaceMetamaterial, "Real-Space Metamaterial"),
        (OctupoleDislocationTab::AuditTelemetry, "Physics Audit & Telemetry"),
    ];

    for (tab, label) in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert_eq!(dialog.active_tab.label(), label);
    }
}

#[test]
fn test_octupole_dislocation_dialog_recompute_and_parameter_updates() {
    let mut dialog = OctupoleDislocationDialog::new_fast();

    // Modify parameters
    dialog.intracell_coupling_gamma_mhz = 1.5;
    dialog.intercell_coupling_lambda_mhz = 9.0;
    dialog.bare_frequency_ghz = 1.2;
    dialog.burgers_vector_bz_mm = 5.0;
    dialog.vortex_charge_l = -1;
    dialog.obstacle_defect_enabled = true;

    dialog.recompute();

    assert_eq!(dialog.cached_dispersion.len(), 61); // 15*4 + 1
    assert_eq!(dialog.cached_corner_modes.len(), 8);
    assert_eq!(dialog.cached_dislocation_dispersion.len(), 25);
    assert_eq!(dialog.cached_router_spectrum.len(), 25);
    assert_eq!(dialog.cached_realspace_slice.len(), 24);
    assert_eq!(dialog.cached_realspace_slice[0].len(), 24);

    assert!(dialog.last_solve_time_us > 0.0);
    assert_eq!(dialog.cached_audit.total_pass_score, 10);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_octupole_dislocation_dialog_headless_render() {
    let ctx = Context::default();
    let mut dialog = OctupoleDislocationDialog::new_fast();
    dialog.is_open = true;

    let tabs = [
        OctupoleDislocationTab::OctupoleMetamaterial,
        OctupoleDislocationTab::ChiralDislocation,
        OctupoleDislocationTab::MultiPortRouter,
        OctupoleDislocationTab::RealSpaceMetamaterial,
        OctupoleDislocationTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        output.textures_delta.clear();
    }
}
