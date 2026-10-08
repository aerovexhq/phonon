#![deny(unsafe_code)]

//! GUI Integration test suite for Phase 446:
//! Topological Phononic Non-Hermitian Skin-Effect Microwave Amplification & Directional Axion Transducer Dialog.

use egui::Context;
use phonon_gui::widgets::topological_skin_axion_dialog::{
    SkinAxionDialogTab, TopologicalSkinAxionDialog,
};

#[test]
fn test_topological_skin_axion_dialog_cold_boot_and_audit() {
    let start = std::time::Instant::now();
    let dialog = TopologicalSkinAxionDialog::new_fast();
    let boot_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        SkinAxionDialogTab::SkinAmplifierGbz
    );
    assert!(
        boot_time_ms < 5.0,
        "Cold boot latency {:.2} ms exceeds 5.0 ms threshold",
        boot_time_ms
    );

    assert_eq!(dialog.cached_audit.total_score, 10);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_topological_skin_axion_dialog_tab_switching() {
    let mut dialog = TopologicalSkinAxionDialog::new_fast();

    let tabs = [
        SkinAxionDialogTab::SkinAmplifierGbz,
        SkinAxionDialogTab::SpatialSkinModes,
        SkinAxionDialogTab::AxionPhononConversion,
        SkinAxionDialogTab::CryogenicReadoutCrossbar,
        SkinAxionDialogTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_topological_skin_axion_dialog_interactive_recompute() {
    let mut dialog = TopologicalSkinAxionDialog::new_fast();

    // 1. Initial cached metrics check
    assert!(dialog.cached_amp_metrics.forward_power_gain_db >= 24.0);
    assert!(dialog.cached_amp_metrics.backward_isolation_db >= 25.0);
    assert!(dialog.cached_amp_metrics.added_noise_quanta <= 0.55);
    assert!(dialog.cached_amp_metrics.skin_localization_ratio >= 0.85);

    assert!(dialog.cached_axion_metrics.conversion_efficiency >= 1.0e-4);
    assert!(dialog.cached_axion_metrics.acoustic_quality_factor >= 1.0e5);
    assert!(dialog.cached_axion_metrics.yoctowatt_sensitivity_w_sqrt_hz <= 1.0e-21);

    assert!(dialog.cached_crossbar_metrics.directivity_db >= 30.0);
    assert!(dialog.cached_crossbar_metrics.dynamic_range_db >= 60.0);
    assert!(dialog.cached_crossbar_metrics.dispersive_readout_snr_db >= 18.0);

    // 2. Adjust amplifier parameters and recompute
    dialog.forward_hopping_tr_mhz = 16.0;
    dialog.reverse_hopping_tl_mhz = 2.0;
    dialog.lattice_site_count = 40;
    dialog.recompute();

    assert!(dialog.cached_amp_metrics.forward_power_gain_db >= 24.0);
    assert!(dialog.cached_amp_metrics.backward_isolation_db >= 25.0);

    // 3. Adjust axion transducer parameters and recompute
    dialog.magnetic_field_t = 12.0;
    dialog.axion_mass_micro_ev = 14.5;
    dialog.acoustic_quality_factor_qm = 300_000.0;
    dialog.recompute();

    assert!(dialog.cached_axion_metrics.conversion_efficiency >= 1.0e-4);
    assert!(dialog.cached_axion_metrics.acoustic_quality_factor >= 1.0e5);

    // 4. Adjust crossbar parameters and recompute
    dialog.operating_temp_k = 0.015;
    dialog.dispersive_shift_chi_mhz = 5.5;
    dialog.recompute();

    assert!(dialog.cached_crossbar_metrics.directivity_db >= 30.0);
    assert!(dialog.cached_crossbar_metrics.dispersive_readout_snr_db >= 18.0);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_topological_skin_axion_dialog_headless_render() {
    let mut dialog = TopologicalSkinAxionDialog::new_fast();
    dialog.is_open = true;

    let ctx = Context::default();

    // Test render pass for all 5 tabs
    let tabs = [
        SkinAxionDialogTab::SkinAmplifierGbz,
        SkinAxionDialogTab::SpatialSkinModes,
        SkinAxionDialogTab::AxionPhononConversion,
        SkinAxionDialogTab::CryogenicReadoutCrossbar,
        SkinAxionDialogTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_dialog_contents(ui);
        });
        output.textures_delta.clear();
        assert_eq!(dialog.active_tab, tab);
    }
}
