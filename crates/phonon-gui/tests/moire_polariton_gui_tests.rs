#![deny(unsafe_code)]

//! GUI Integration test suite for Phase 448:
//! Moiré Exciton-Polariton Valley Hall Chiral Lasing Metasurface & Opto-Acoustic Synthesizer Dialog.

use egui::Context;
use phonon_gui::widgets::moire_polariton_dialog::{
    MoirePolaritonDialog, MoirePolaritonDialogTab,
};
use phonon_solver::moire_polariton_laser::CircularPolarization;

#[test]
fn test_moire_polariton_dialog_cold_boot_and_audit() {
    let start = std::time::Instant::now();
    let dialog = MoirePolaritonDialog::new_fast();
    let boot_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, MoirePolaritonDialogTab::MoirePolaritonBand);
    assert!(
        boot_time_ms < 5.0,
        "Cold boot latency {:.2} ms exceeds 5.0 ms threshold",
        boot_time_ms
    );

    assert_eq!(dialog.cached_audit.total_score, 10);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_moire_polariton_dialog_tab_switching() {
    let mut dialog = MoirePolaritonDialog::new_fast();

    let tabs = [
        MoirePolaritonDialogTab::MoirePolaritonBand,
        MoirePolaritonDialogTab::ValleyHallEdgeWaveguide,
        MoirePolaritonDialogTab::ChiralPolaritonLaser,
        MoirePolaritonDialogTab::OptoAcousticSynthesizer,
        MoirePolaritonDialogTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
    }
}

#[test]
fn test_moire_polariton_dialog_interactive_recompute() {
    let mut dialog = MoirePolaritonDialog::new_fast();

    // Modify parameters
    dialog.twist_angle_deg = 2.4;
    dialog.rabi_splitting_mev = 30.0;
    dialog.pump_power_uw_um2 = 8.0;
    dialog.microwave_acoustic_freq_ghz = 5.0;
    dialog.pump_polarization = CircularPolarization::SigmaMinus;

    dialog.recompute();

    assert_eq!(dialog.twist_angle_deg, 2.4);
    assert!(dialog.cached_lattice_metrics.rabi_splitting_mev >= 29.0);
    assert!(dialog.cached_lasing_metrics.is_above_threshold);
    assert!(dialog.last_solve_time_us < 5000.0);
}

#[test]
fn test_moire_polariton_dialog_egui_headless_render() {
    let ctx = Context::default();
    let mut dialog = MoirePolaritonDialog::new_fast();
    dialog.is_open = true;

    for tab in [
        MoirePolaritonDialogTab::MoirePolaritonBand,
        MoirePolaritonDialogTab::ValleyHallEdgeWaveguide,
        MoirePolaritonDialogTab::ChiralPolaritonLaser,
        MoirePolaritonDialogTab::OptoAcousticSynthesizer,
        MoirePolaritonDialogTab::AuditTelemetry,
    ] {
        dialog.active_tab = tab;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        output.textures_delta.clear();

        assert_eq!(dialog.active_tab, tab);
    }

    assert!(dialog.is_open);
}
