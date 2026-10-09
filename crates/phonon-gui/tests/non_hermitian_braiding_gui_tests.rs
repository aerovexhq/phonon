#![deny(unsafe_code)]

use egui::Context;
use phonon_gui::widgets::non_hermitian_braiding_dialog::{
    NonHermitianBraidingDialog, NonHermitianBraidingTab,
};
use phonon_solver::non_hermitian_braiding::NonHermitianGateKind;
use std::time::Instant;

#[test]
fn test_non_hermitian_braiding_dialog_initialization_and_fast_boot() {
    let start = Instant::now();
    let dialog = NonHermitianBraidingDialog::new_fast();
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 5,
        "Fast boot latency must be < 5 ms, took {} ms",
        duration.as_millis()
    );

    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        NonHermitianBraidingTab::SkinBraiding
    );

    assert_eq!(dialog.cached_audit.passed_count, 10);
    assert_eq!(dialog.cached_audit.total_count, 10);
    assert!(dialog.cached_audit.is_all_pass());
    assert!(dialog.cached_skin_metrics.skin_confinement_ratio >= 0.88);
    assert!(dialog.cached_skin_metrics.bulk_point_gap_mhz >= 15.0);
    assert!(dialog.cached_skin_metrics.braid_process_fidelity >= 0.996);
    assert!(dialog.cached_sensor_metrics.responsivity_enhancement >= 120.0);
    assert!(dialog.cached_sensor_metrics.min_detectable_perturbation <= 1.0e-10);
    assert!(dialog.cached_compiler_metrics.gate_fidelity >= 0.998);
}

#[test]
fn test_non_hermitian_braiding_dialog_parameter_mutation_and_recompute() {
    let mut dialog = NonHermitianBraidingDialog::new_fast();

    dialog.non_hermitian_drift_g = 0.50;
    dialog.test_perturbation_epsilon = 1.0e-4;
    dialog.metric_parameter_s = 0.40;
    dialog.selected_gate = NonHermitianGateKind::PhaseS;
    dialog.recompute();

    assert!(!dialog.cached_spatial_points.is_empty());
    assert!(!dialog.cached_braid_trajectory.is_empty());
    assert!(!dialog.cached_splitting_spectrum.is_empty());

    assert!(dialog.cached_skin_metrics.gbz_radius < 0.95);
    assert!(dialog.cached_sensor_metrics.dynamic_range_db >= 65.0);
    assert!(dialog.cached_gate_result.process_fidelity >= 0.997);
    assert!(dialog.cached_audit.passed_count >= 9);
}

#[test]
fn test_non_hermitian_braiding_dialog_tab_switching() {
    let mut dialog = NonHermitianBraidingDialog::new_fast();

    let tabs = [
        NonHermitianBraidingTab::SkinBraiding,
        NonHermitianBraidingTab::ExceptionalSurfaceSensor,
        NonHermitianBraidingTab::HolonomicCompiler,
        NonHermitianBraidingTab::SkinLatticeCanvas,
        NonHermitianBraidingTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!dialog.active_tab.label().is_empty());
    }
}

#[test]
fn test_non_hermitian_braiding_dialog_headless_egui_render() {
    let mut dialog = NonHermitianBraidingDialog::new_fast();
    dialog.is_open = true;
    dialog.recompute();

    let ctx = Context::default();

    for tab in [
        NonHermitianBraidingTab::SkinBraiding,
        NonHermitianBraidingTab::ExceptionalSurfaceSensor,
        NonHermitianBraidingTab::HolonomicCompiler,
        NonHermitianBraidingTab::SkinLatticeCanvas,
        NonHermitianBraidingTab::AuditTelemetry,
    ] {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_content(ui);
        });
        output.textures_delta.clear();
    }
}
