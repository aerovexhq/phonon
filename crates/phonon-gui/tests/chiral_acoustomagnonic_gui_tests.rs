#![deny(unsafe_code)]

use egui::Context;
use phonon_gui::widgets::chiral_acoustomagnonic_dialog::{
    AcoustomagnonicTab, ChiralAcoustomagnonicDialog,
};

#[test]
fn test_dialog_initialization_and_cold_boot() {
    let start = std::time::Instant::now();
    let dialog = ChiralAcoustomagnonicDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "Cold boot initialization must be < 5ms, took {} ms",
        elapsed.as_millis()
    );
    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        AcoustomagnonicTab::AcoustomagnonicDispersion
    );
    assert!(
        !dialog.cached_dispersion.is_empty(),
        "Cached dispersion must not be empty"
    );
    assert!(
        dialog.cached_saw_metrics.isolation_depth_db >= 35.0,
        "Cached SAW isolation depth must be >= 35 dB"
    );
    assert!(
        dialog.cached_circ_metrics.added_noise_quanta <= 0.55,
        "Cached added noise must be <= 0.55 quanta"
    );
}

#[test]
fn test_dialog_tab_switching() {
    let mut dialog = ChiralAcoustomagnonicDialog::new_fast();

    let tabs = [
        AcoustomagnonicTab::AcoustomagnonicDispersion,
        AcoustomagnonicTab::NonReciprocalSawIsolator,
        AcoustomagnonicTab::CryogenicQubitCirculator,
        AcoustomagnonicTab::RealSpaceHeterostructure,
        AcoustomagnonicTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_dialog_recompute_and_parameter_updates() {
    let mut dialog = ChiralAcoustomagnonicDialog::new_fast();

    // Adjust parameters and trigger recompute
    dialog.center_freq_ghz = 4.5;
    dialog.coupling_strength_mhz = 45.0;
    dialog.waveguide_length_mm = 3.5;
    dialog.operating_temp_k = 0.015;
    dialog.recompute();

    assert_eq!(dialog.center_freq_ghz, 4.5);
    assert!(
        dialog.cached_saw_metrics.insertion_loss_db <= 0.60,
        "Insertion loss must remain <= 0.60 dB"
    );
    assert!(
        dialog.cached_saw_metrics.isolation_depth_db >= 35.0,
        "Isolation depth must remain >= 35 dB"
    );
    assert!(
        dialog.cached_circ_metrics.thermal_leakage_photons < 1e-3,
        "Thermal leakage must be < 1e-3 photons"
    );
    assert!(
        dialog.cached_audit.all_passed,
        "All 10 audit criteria must pass after recompute"
    );
}

#[test]
fn test_dialog_headless_render() {
    let ctx = Context::default();
    let mut dialog = ChiralAcoustomagnonicDialog::new_fast();
    dialog.is_open = true;

    for tab in &[
        AcoustomagnonicTab::AcoustomagnonicDispersion,
        AcoustomagnonicTab::NonReciprocalSawIsolator,
        AcoustomagnonicTab::CryogenicQubitCirculator,
        AcoustomagnonicTab::RealSpaceHeterostructure,
        AcoustomagnonicTab::AuditTelemetry,
    ] {
        dialog.active_tab = *tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        output.textures_delta.clear();
    }
}
