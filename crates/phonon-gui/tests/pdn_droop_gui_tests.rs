#![deny(unsafe_code)]

//! GUI test suite for Power Delivery Network (PDN) & Ultra-High di/dt Dynamic Droop CAD Dialog.

use egui::Context;
use phonon_gui::widgets::pdn_droop_dialog::{PdnDroopDialog, PdnTab};
use std::time::Instant;

#[test]
fn test_pdn_droop_dialog_initialization_and_cold_boot() {
    let start = Instant::now();
    let dialog = PdnDroopDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "PdnDroopDialog::new_fast() must initialize in sub-5ms, took {:?}",
        elapsed
    );

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, PdnTab::ImpedanceSpectrum);
    assert!(!dialog.cached_freq_hz.is_empty());
    assert!(!dialog.cached_z_mohm.is_empty());
    assert!(dialog.cached_z_target_mohm > 0.0);
    assert!(!dialog.cached_transient_time_ns.is_empty());
    assert!(!dialog.cached_unmitigated_v_die_v.is_empty());
    assert!(!dialog.cached_mitigated_v_die_v.is_empty());

    let tel = &dialog.cached_telemetry;
    assert!(tel.target_impedance_mohm > 0.0);
    assert!(tel.unmitigated_peak_droop_mv > 0.0);
    assert!(tel.mitigated_peak_droop_mv > 0.0);
    assert!(tel.droop_reduction_pct >= 35.0);
}

#[test]
fn test_pdn_droop_dialog_recompute_and_parameter_updates() {
    let mut dialog = PdnDroopDialog::new_fast();

    // Adjust nominal voltage, load step, and DLDO parameters
    dialog.sim.params.vrm.v_dd_v = 0.90;
    dialog.sim.params.load_step_current_a = 900.0;
    dialog.sim.load.delta_i_a = 900.0;
    dialog.sim.dldo.max_injected_current_a = 450.0;
    dialog.sim.stretch.stretch_ratio = 0.50;

    dialog.recompute_all();

    assert!(!dialog.cached_freq_hz.is_empty());
    assert!(!dialog.cached_transient_time_ns.is_empty());

    let tel = &dialog.cached_telemetry;
    assert_eq!(tel.v_dd_nominal_v, 0.90);
    assert!(tel.droop_reduction_pct >= 35.0);
}

#[test]
fn test_pdn_droop_dialog_headless_egui_render_all_tabs() {
    let ctx = Context::default();
    let mut dialog = PdnDroopDialog::new_fast();
    dialog.is_open = true;

    let tabs = [
        PdnTab::ImpedanceSpectrum,
        PdnTab::DynamicDroopScope,
        PdnTab::DecouplingHierarchy,
        PdnTab::ActiveMitigation,
        PdnTab::PowerIntegrityMatrix,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.ui(ui.ctx());
        });
        out.textures_delta.clear();
    }

    // Test show method
    let mut out_window = ctx.run_ui(Default::default(), |ui| {
        dialog.show(ui.ctx());
    });
    out_window.textures_delta.clear();
}
