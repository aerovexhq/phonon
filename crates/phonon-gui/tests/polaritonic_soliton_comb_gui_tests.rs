#![deny(unsafe_code)]

//! GUI test suite for Phase 434: Polaritonic Soliton Frequency Comb & Dissipative Kerr Squeezer Dialog.

use egui::Context;
use phonon_gui::widgets::polaritonic_soliton_comb_dialog::{
    PolaritonicCombTab, PolaritonicSolitonCombDialog,
};

#[test]
fn test_polaritonic_soliton_comb_dialog_initialization_and_cold_boot() {
    let start = std::time::Instant::now();
    let dialog = PolaritonicSolitonCombDialog::new_fast();
    let duration = start.elapsed();

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, PolaritonicCombTab::KagomeMetamaterialBands);
    assert!(
        duration.as_millis() < 5,
        "Cold-boot latency must be under 5 ms (got {:?})",
        duration
    );
    assert!(dialog.cached_bands.relative_flatness < 1e-4);
    assert!(dialog.cached_soliton.contrast_ratio_db >= 20.0);
    assert!(dialog.cached_soliton.active_comb_lines >= 30);
    assert!(dialog.cached_squeezing.squeezing_db >= 6.0);
    assert_eq!(dialog.cached_audit.total_score, 10);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_polaritonic_soliton_comb_dialog_tab_switching() {
    let mut dialog = PolaritonicSolitonCombDialog::default();

    dialog.active_tab = PolaritonicCombTab::KagomeMetamaterialBands;
    assert_eq!(dialog.active_tab.label(), "Kagome Metamaterial Bands");

    dialog.active_tab = PolaritonicCombTab::DissipativeKerrSoliton;
    assert_eq!(dialog.active_tab.label(), "Dissipative Kerr Soliton");

    dialog.active_tab = PolaritonicCombTab::FrequencyCombSpectrum;
    assert_eq!(dialog.active_tab.label(), "Microcomb Spectrum");

    dialog.active_tab = PolaritonicCombTab::QuantumNoiseSqueezing;
    assert_eq!(dialog.active_tab.label(), "Quantum Noise Squeezing");

    dialog.active_tab = PolaritonicCombTab::AuditTelemetry;
    assert_eq!(dialog.active_tab.label(), "Physics Audit & Telemetry");
}

#[test]
fn test_polaritonic_soliton_comb_parameter_adjustment_and_recompute() {
    let mut dialog = PolaritonicSolitonCombDialog::default();

    dialog.hopping_t_mhz = 15.0;
    dialog.pump_power_mw = 20.0;
    dialog.detuning_alpha = 3.0;
    dialog.squeezing_angle_rad = 0.85;
    dialog.recompute_all();

    assert_eq!(dialog.processor.kagome_solver.params.hopping_t_mhz, 15.0);
    assert_eq!(dialog.processor.soliton_solver.params.pump_power_mw, 20.0);
    assert_eq!(dialog.processor.soliton_solver.params.detuning_alpha, 3.0);
    assert_eq!(
        dialog.processor.noise_solver.params.squeezing_angle_rad,
        0.85
    );

    assert!(dialog.cached_bands.points.len() >= 140);
    assert!(dialog.cached_soliton.active_comb_lines >= 30);
    assert!(dialog.cached_squeezing.squeezing_db >= 6.0);
}

#[test]
fn test_polaritonic_soliton_comb_headless_render_pass() {
    let ctx = Context::default();
    let mut dialog = PolaritonicSolitonCombDialog::default();
    dialog.is_open = true;

    // Render across all 5 tabs in headless egui passes
    let tabs = [
        PolaritonicCombTab::KagomeMetamaterialBands,
        PolaritonicCombTab::DissipativeKerrSoliton,
        PolaritonicCombTab::FrequencyCombSpectrum,
        PolaritonicCombTab::QuantumNoiseSqueezing,
        PolaritonicCombTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        out.textures_delta.clear();
    }
}
