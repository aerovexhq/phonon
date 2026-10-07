#![deny(unsafe_code)]

use egui::Context;
use phonon_gui::widgets::chern_simons_interferometer_dialog::{
    ChernSimonsInterferometerDialog, ChernSimonsTab,
};
use phonon_solver::chiral_chern_simons_interferometer::FractionalAnyonKind;

#[test]
fn test_dialog_initialization_and_cold_boot() {
    let start = std::time::Instant::now();
    let dialog = ChernSimonsInterferometerDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "Cold boot initialization must be < 5ms, took {} ms",
        elapsed.as_millis()
    );
    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        ChernSimonsTab::AnyonicInterferometer
    );
    assert!(!dialog.cached_spectrum.is_empty());
    assert!(
        dialog.cached_metrics.visibility >= 0.85,
        "Visibility must be >= 85%"
    );
    assert!(
        dialog.cached_memory.coherence_time_topo_us >= 250.0,
        "Topological coherence must be >= 250 us"
    );
    assert!(
        dialog.cached_memory.storage_retrieval_fidelity >= 0.995,
        "Storage retrieval fidelity must be >= 0.995"
    );
}

#[test]
fn test_dialog_tab_switching() {
    let mut dialog = ChernSimonsInterferometerDialog::new_fast();

    let tabs = [
        ChernSimonsTab::AnyonicInterferometer,
        ChernSimonsTab::FractionalStatistics,
        ChernSimonsTab::NonAbelianQuantumMemory,
        ChernSimonsTab::RealSpaceInterferometer,
        ChernSimonsTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_dialog_recompute_and_parameter_updates() {
    let mut dialog = ChernSimonsInterferometerDialog::new_fast();

    // Adjust parameters and trigger recompute
    dialog.anyon_kind = FractionalAnyonKind::LaughlinOneFifth;
    dialog.qpc1_tunneling_prob = 0.30;
    dialog.topological_gap_mhz = 22.0;
    dialog.recompute();

    assert_eq!(dialog.anyon_kind, FractionalAnyonKind::LaughlinOneFifth);
    assert_eq!(dialog.cached_metrics.effective_anyon_charge, 0.20);
    assert!(
        dialog.cached_metrics.visibility >= 0.85,
        "Visibility must remain >= 85%"
    );
    assert!(
        dialog.cached_memory.coherence_time_topo_us >= 250.0,
        "Topological coherence must remain >= 250 us"
    );
    assert!(
        dialog.cached_audit.all_passed,
        "All 10 audit criteria must pass after recompute"
    );
}

#[test]
fn test_dialog_headless_render() {
    let ctx = Context::default();
    let mut dialog = ChernSimonsInterferometerDialog::new_fast();
    dialog.is_open = true;

    for tab in &[
        ChernSimonsTab::AnyonicInterferometer,
        ChernSimonsTab::FractionalStatistics,
        ChernSimonsTab::NonAbelianQuantumMemory,
        ChernSimonsTab::RealSpaceInterferometer,
        ChernSimonsTab::AuditTelemetry,
    ] {
        dialog.active_tab = *tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        output.textures_delta.clear();
    }
}
