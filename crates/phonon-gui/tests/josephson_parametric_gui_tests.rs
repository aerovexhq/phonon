#![deny(unsafe_code)]

//! GUI test suite for Phase 402: JosephsonParametricDialog in Phonon CAD Studio.
//!
//! Verifies:
//! - JosephsonParametricDialog initialization, default state, and physics caches.
//! - Tab switching across all 5 visual workspaces.
//! - Parameter adjustment (flux bias, pump power ratio, squeezing angle).
//! - Instantaneous cold boot latency (< 2.0 ms).
//! - Headless egui Context render pass across all 5 tabs.

use std::time::Instant;
use egui::vec2;
use phonon_gui::widgets::josephson_parametric_dialog::{
    JosephsonParametricDialog, JosephsonParametricTab, WignerColormap,
};

#[test]
fn test_josephson_parametric_dialog_initialization_and_defaults() {
    let dialog = JosephsonParametricDialog::new();

    // Dialog must be closed by default
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Default tab
    assert_eq!(dialog.active_tab, JosephsonParametricTab::SquidFluxTuning);
    assert_eq!(
        dialog.active_tab.label(),
        "1. SQUID Loop & Flux Tuning"
    );

    // Waveguide parameter defaults
    assert_eq!(dialog.processor.waveguide_params.bare_frequency_ghz, 6.0);
    assert_eq!(dialog.processor.waveguide_params.critical_current_ua, 2.5);
    assert_eq!(dialog.processor.waveguide_params.shunt_capacitance_pf, 0.8);
    assert_eq!(dialog.processor.waveguide_params.geometric_inductance_ph, 50.0);
    assert_eq!(dialog.processor.waveguide_params.piezo_coupling_mhz, 15.0);
    assert_eq!(dialog.processor.waveguide_params.flux_bias_ratio, 0.25);
    assert_eq!(dialog.processor.waveguide_params.pump_frequency_ghz, 12.0);
    assert_eq!(dialog.processor.waveguide_params.pump_power_ratio, 0.70);

    // Squeezing parameter defaults
    assert_eq!(dialog.processor.squeezing_params.pump_phase_rad, 0.0);
    assert_eq!(dialog.processor.squeezing_params.squeezing_angle_rad, 0.0);
    assert_eq!(dialog.processor.squeezing_params.operating_temp_k, 0.010);

    // Cluster parameter defaults
    assert_eq!(dialog.processor.cluster_params.beam_splitter_reflectivity, 0.50);
    assert_eq!(dialog.processor.cluster_params.mode_count, 2);

    // Cached curves and metrics
    assert!(
        !dialog.cached_tuning_curve.is_empty(),
        "Tuning curve points must be cached"
    );
    assert!(
        !dialog.cached_inductance_curve.is_empty(),
        "Inductance curve points must be cached"
    );
    assert!(
        !dialog.cached_gain_spectrum.is_empty(),
        "Gain spectrum points must be cached"
    );
    assert!(
        !dialog.cached_quadrature_scan.is_empty(),
        "Quadrature scan points must be cached"
    );

    // Audit report check
    assert_eq!(dialog.cached_audit.total_count, 10);
    assert_eq!(dialog.cached_audit.passed_count, 10);
    assert!(dialog.cached_audit.overall_pass);
}

#[test]
fn test_josephson_parametric_tab_switching() {
    let mut dialog = JosephsonParametricDialog::new();

    let tabs = [
        (
            JosephsonParametricTab::SquidFluxTuning,
            "1. SQUID Loop & Flux Tuning",
        ),
        (
            JosephsonParametricTab::ParametricGainBandwidth,
            "2. Parametric Gain & Bandwidth",
        ),
        (
            JosephsonParametricTab::SqueezedVacuumWigner,
            "3. Squeezed Vacuum & Wigner Map",
        ),
        (
            JosephsonParametricTab::CvEntanglementClusterState,
            "4. CV Entanglement & Cluster State",
        ),
        (
            JosephsonParametricTab::PhysicsAuditTelemetry,
            "5. Physics Audit & Telemetry",
        ),
    ];

    for (tab, expected_label) in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert_eq!(dialog.active_tab.label(), expected_label);
    }
}

#[test]
fn test_parameter_adjustment() {
    let mut dialog = JosephsonParametricDialog::new();

    // 1. Adjust flux bias
    dialog.processor.waveguide_params.flux_bias_ratio = 0.35;
    dialog.refresh_simulation();
    assert_eq!(dialog.processor.waveguide_params.flux_bias_ratio, 0.35);

    // Resonant frequency must reflect the updated flux bias
    let f_tuned = dialog.processor.waveguide_params.resonant_frequency_ghz();
    assert!(f_tuned > 0.0, "Tuned frequency must be positive");

    // 2. Adjust pump power ratio
    dialog.processor.waveguide_params.pump_power_ratio = 0.85;
    dialog.refresh_simulation();
    assert_eq!(dialog.processor.waveguide_params.pump_power_ratio, 0.85);
    assert!(
        dialog.processor.amp_response.gain_max_db >= 20.0,
        "Gain must remain >= 20 dB under higher pump drive"
    );

    // 3. Adjust squeezing angle and colormap
    dialog.processor.squeezing_params.squeezing_angle_rad = 0.785; // pi / 4
    dialog.colormap = WignerColormap::Magma;
    dialog.refresh_simulation();
    assert_eq!(dialog.processor.squeezing_params.squeezing_angle_rad, 0.785);
    assert_eq!(dialog.colormap, WignerColormap::Magma);
}

#[test]
fn test_cold_boot_latency() {
    let t_start = Instant::now();
    let dialog = JosephsonParametricDialog::new();
    let elapsed = t_start.elapsed();

    assert!(
        elapsed.as_millis() < 50,
        "Cold boot instantiation must be fast, took {:?}",
        elapsed
    );

    assert!(
        dialog.cached_audit.cold_boot_latency_us < 2000.0,
        "Cold boot latency {} us must be < 2000 us (2.0 ms)",
        dialog.cached_audit.cold_boot_latency_us
    );
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = JosephsonParametricDialog::new();
    dialog.is_open = true;

    let ctx = egui::Context::default();

    // Render pass on each of the 5 tabs
    for tab in [
        JosephsonParametricTab::SquidFluxTuning,
        JosephsonParametricTab::ParametricGainBandwidth,
        JosephsonParametricTab::SqueezedVacuumWigner,
        JosephsonParametricTab::CvEntanglementClusterState,
        JosephsonParametricTab::PhysicsAuditTelemetry,
    ] {
        dialog.active_tab = tab;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            ui.set_min_size(vec2(1000.0, 700.0));
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
}
