#![deny(unsafe_code)]

//! Integration, Cold-Boot Latency, and Headless egui Render Tests for CornerDoublerDialog.

use std::time::Instant;
use egui::Context;
use phonon_gui::{
    CornerDoublerDialog, CornerDoublerTab, DialogColormap, DisplayModeType,
};
use phonon_solver::corner_harmonic_doubler::RouterTargetPort;

#[test]
fn test_corner_doubler_dialog_cold_boot_latency() {
    let start = Instant::now();
    let dialog = CornerDoublerDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "Cold-boot constructor must complete in < 5ms for instantaneous startup, took {:?}",
        elapsed
    );
    assert!(!dialog.is_open, "Dialog must be closed by default on cold boot");
    assert_eq!(
        dialog.active_tab,
        CornerDoublerTab::HierarchicalTopology,
        "Dialog must open to Hierarchical Topology tab by default"
    );
}

#[test]
fn test_corner_doubler_dialog_parameter_adjustments() {
    let mut dialog = CornerDoublerDialog::new_fast();

    // 1. Initial topological verification
    assert!(dialog.coupling_params.is_topological_soti());
    assert!(dialog.steady_state.efficiency >= 0.30);
    assert!(dialog.scattering_matrix.s21_transmission_linear >= 0.95);

    // 2. Adjust parameters: transition to trivial phase (gamma > lambda)
    dialog.gamma_mhz = 12.0;
    dialog.lambda_mhz = 3.0;
    dialog.recompute();

    assert!(!dialog.coupling_params.is_topological_soti(), "Must become trivial phase when gamma > lambda");
    assert!(dialog.coupling_params.hopping_ratio() > 1.0);

    // 3. Restore topological parameters and increase pump power
    dialog.gamma_mhz = 2.0;
    dialog.lambda_mhz = 8.0;
    dialog.p_in_mw = 75.0;
    dialog.recompute();

    assert!(dialog.coupling_params.is_topological_soti());
    assert!(
        dialog.steady_state.efficiency >= 0.30,
        "Efficiency with 75 mW pump must be >= 30%, got {:.1}%",
        dialog.steady_state.efficiency_pct
    );

    // 4. Change visualization colormap and display mode
    dialog.selected_colormap = DialogColormap::Turbo;
    dialog.selected_display_mode = DisplayModeType::Edge1D;
    assert_eq!(dialog.selected_colormap, DialogColormap::Turbo);
    assert_eq!(dialog.selected_display_mode, DisplayModeType::Edge1D);

    // 5. Change router target port
    dialog.router_params.target_port = RouterTargetPort::Port1Forward;
    dialog.recompute();
    assert_eq!(dialog.router_params.target_port, RouterTargetPort::Port1Forward);
    assert!(dialog.scattering_matrix.cross_port_isolation_db >= 25.0);
}

#[test]
fn test_corner_doubler_dialog_tab_switching() {
    let mut dialog = CornerDoublerDialog::new_fast();

    let all_tabs = [
        CornerDoublerTab::HierarchicalTopology,
        CornerDoublerTab::FrequencyDoubler,
        CornerDoublerTab::BackscatteringImmuneRouting,
        CornerDoublerTab::MultiOctaveBeamSteering,
        CornerDoublerTab::AuditTelemetry,
    ];

    for tab in all_tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab, "Active tab must switch correctly");
    }
}

#[test]
fn test_corner_doubler_dialog_audit_full_pass() {
    let dialog = CornerDoublerDialog::new_fast();

    assert_eq!(
        dialog.audit_score,
        (10, 10),
        "Baseline configuration must pass all 10/10 audit criteria, got {:?}",
        dialog.audit_score
    );

    for crit in &dialog.audit_criteria {
        assert!(
            crit.is_passed,
            "Audit criterion '{}' failed: spec='{}', obs='{}'",
            crit.criterion, crit.specification, crit.observed_state
        );
    }
}

#[test]
fn test_corner_doubler_dialog_headless_render_pass() {
    let ctx = Context::default();
    let mut dialog = CornerDoublerDialog::new_fast();
    dialog.is_open = true;

    let all_tabs = [
        CornerDoublerTab::HierarchicalTopology,
        CornerDoublerTab::FrequencyDoubler,
        CornerDoublerTab::BackscatteringImmuneRouting,
        CornerDoublerTab::MultiOctaveBeamSteering,
        CornerDoublerTab::AuditTelemetry,
    ];

    for tab in all_tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.ui(ui.ctx());
        });
        out.textures_delta.clear();
        assert!(dialog.is_open, "Dialog must remain open across headless render passes");
    }
}
