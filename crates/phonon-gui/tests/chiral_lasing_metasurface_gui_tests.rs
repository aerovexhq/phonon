#![deny(unsafe_code)]

//! GUI Integration test suite for Phase 441:
//! Topological Non-Hermitian Floquet Acoustic Chiral Lasing Metasurface & Vortex Waveguide Dialog.

use egui::Context;
use phonon_gui::widgets::chiral_lasing_metasurface_dialog::{
    ChiralLasingMetasurfaceDialog, ChiralLasingTab,
};

#[test]
fn test_chiral_lasing_dialog_cold_boot_and_audit() {
    let start = std::time::Instant::now();
    let dialog = ChiralLasingMetasurfaceDialog::new_fast();
    let boot_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, ChiralLasingTab::FloquetChiralGain);
    assert!(
        boot_time_ms < 5.0,
        "Cold boot latency {:.2} ms exceeds 5.0 ms threshold",
        boot_time_ms
    );

    assert_eq!(dialog.cached_audit.total_score, 10);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_chiral_lasing_dialog_tab_switching() {
    let mut dialog = ChiralLasingMetasurfaceDialog::new_fast();

    let tabs = [
        ChiralLasingTab::FloquetChiralGain,
        ChiralLasingTab::OamVortexWaveguide,
        ChiralLasingTab::ModeCompetition,
        ChiralLasingTab::ChiralRadiationPattern,
        ChiralLasingTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_chiral_lasing_dialog_interactive_recompute() {
    let mut dialog = ChiralLasingMetasurfaceDialog::new_fast();

    // 1. Initial cached metrics check
    assert!(dialog.cached_lattice_metrics.chiral_isolation_db >= 25.0);
    assert!(dialog.cached_vortex_metrics.oam_modal_purity >= 0.90);
    assert!(dialog.cached_rate_metrics.smsr_db >= 30.0);

    // 2. Adjust Floquet parameters and recompute
    dialog.floquet_modulation_mhz = 12.0;
    dialog.gain_rate_gamma_mhz = 4.0;
    dialog.recompute();
    assert!(dialog.cached_lattice_metrics.chiral_isolation_db >= 25.0);

    // 3. Switch topological charge ell = 2 and recompute
    dialog.topological_charge_ell = 2;
    dialog.beam_waist_um = 60.0;
    dialog.recompute();
    assert_eq!(dialog.cached_vortex_metrics.measured_topological_charge, 2);
    assert!(dialog.cached_vortex_metrics.oam_modal_purity >= 0.90);

    // 4. Adjust pump current and recompute
    dialog.pump_current_ma = 45.0;
    dialog.recompute();
    assert!(dialog.cached_rate_metrics.steady_state_power_mw > 0.0);
    assert!(dialog.cached_rate_metrics.emission_linewidth_khz <= 5.0);
}

#[test]
fn test_chiral_lasing_dialog_headless_render() {
    let mut dialog = ChiralLasingMetasurfaceDialog::new_fast();
    dialog.is_open = true;

    let ctx = Context::default();

    // Test render pass for all 5 tabs
    let tabs = [
        ChiralLasingTab::FloquetChiralGain,
        ChiralLasingTab::OamVortexWaveguide,
        ChiralLasingTab::ModeCompetition,
        ChiralLasingTab::ChiralRadiationPattern,
        ChiralLasingTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_dialog_contents(ui);
        });
        output.textures_delta.clear();
        assert_eq!(dialog.active_tab, tab);
    }

    // Also verify standard window ui() call
    let mut window_output = ctx.run_ui(Default::default(), |ui| {
        dialog.ui(ui.ctx());
    });
    window_output.textures_delta.clear();

    assert!(dialog.is_open);
}
