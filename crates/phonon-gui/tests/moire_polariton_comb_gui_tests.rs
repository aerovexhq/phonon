#![deny(unsafe_code)]

//! Automated GUI test suite for Phase 415: Phonon Studio Topological Acoustic
//! Moiré Flat-Band Polariton Soliton & Higher-Order Corner Comb Dialog.

use egui::Context;
use phonon_gui::widgets::{MoireCombTab, MoirePolaritonCombDialog};

#[test]
fn test_dialog_initialization_and_fast_boot() {
    let dialog = MoirePolaritonCombDialog::new_fast();
    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, MoireCombTab::MoireFlatBand);
    assert!((dialog.twist_angle_deg - 1.08).abs() < 1e-4);
    assert!(dialog.cached_band_metrics.is_magic_angle);
    assert!(dialog.cached_soliton_metrics.is_soliton_stable);
    assert!(dialog.cached_comb_metrics.is_comb_active);

    // Verify precomputed caches
    assert!(!dialog.cached_dispersion_path.is_empty());
    assert!(!dialog.cached_spatial_pattern.is_empty());
    assert!(!dialog.cached_soliton_profile.is_empty());
    assert!(!dialog.cached_comb_spectrum.is_empty());
    assert!(!dialog.cached_corner_profile.is_empty());
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_tab_switching() {
    let mut dialog = MoirePolaritonCombDialog::new();
    let tabs = [
        MoireCombTab::MoireFlatBand,
        MoireCombTab::PolaritonSoliton,
        MoireCombTab::CornerModes,
        MoireCombTab::MicrocombSpectrum,
        MoireCombTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_parameter_adjustment_and_recomputation() {
    let mut dialog = MoirePolaritonCombDialog::new();

    // Adjust parameters away from default
    dialog.twist_angle_deg = 1.05;
    dialog.peak_power_mw = 25.0;
    dialog.target_comb_lines = 46;
    dialog.corner_q_factor = 45000.0;
    dialog.recompute();

    assert!(dialog.cached_band_metrics.flat_bandwidth_mhz <= 0.50);
    assert!(dialog.cached_soliton_metrics.self_trapping_ratio >= 0.85);
    assert!(dialog.cached_comb_metrics.comb_line_count >= 40);
    assert!(dialog.last_solve_time_us > 0.0);
}

#[test]
fn test_preset_switching() {
    let mut dialog = MoirePolaritonCombDialog::new();

    // Preset 2: Detuned broad dispersion
    dialog.twist_angle_deg = 1.35;
    dialog.recompute();
    assert!(!dialog.cached_band_metrics.is_magic_angle);
    assert!(dialog.cached_band_metrics.flat_bandwidth_mhz > 0.50);

    // Preset 1: Magic angle flat band
    dialog.twist_angle_deg = 1.08;
    dialog.recompute();
    assert!(dialog.cached_band_metrics.is_magic_angle);
    assert!(dialog.cached_band_metrics.flat_bandwidth_mhz <= 0.50);
}

#[test]
fn test_headless_egui_render_pass() {
    let ctx = Context::default();
    let mut dialog = MoirePolaritonCombDialog::new();
    dialog.is_open = true;

    // Render across all 5 tabs in headless mode
    let tabs = [
        MoireCombTab::MoireFlatBand,
        MoireCombTab::PolaritonSoliton,
        MoireCombTab::CornerModes,
        MoireCombTab::MicrocombSpectrum,
        MoireCombTab::AuditTelemetry,
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
}
