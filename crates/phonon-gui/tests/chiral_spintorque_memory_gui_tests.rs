#![deny(unsafe_code)]

//! GUI Integration test suite for Phase 443:
//! Chiral Phonon-Magnon Spin-Torque Acoustic Memory & Cryogenic Superconducting Spintronic Crossbar Dialog.

use egui::Context;
use phonon_gui::widgets::chiral_spintorque_memory_dialog::{
    ChiralSpinTorqueMemoryDialog, MemoryDialogTab,
};
use phonon_solver::chiral_spintorque_memory::MemoryCellState;

#[test]
fn test_chiral_spintorque_dialog_cold_boot_and_audit() {
    let start = std::time::Instant::now();
    let dialog = ChiralSpinTorqueMemoryDialog::new_fast();
    let boot_time_ms = start.elapsed().as_secs_f64() * 1000.0;

    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        MemoryDialogTab::AcousticSpinTorqueDynamics
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
fn test_chiral_spintorque_dialog_tab_switching() {
    let mut dialog = ChiralSpinTorqueMemoryDialog::new_fast();

    let tabs = [
        MemoryDialogTab::AcousticSpinTorqueDynamics,
        MemoryDialogTab::PolaritonWritingHead,
        MemoryDialogTab::SpintronicCrossbarGrid,
        MemoryDialogTab::CryogenicReadout,
        MemoryDialogTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_chiral_spintorque_dialog_interactive_recompute() {
    let mut dialog = ChiralSpinTorqueMemoryDialog::new_fast();

    // 1. Initial cached metrics check
    assert!(dialog.cached_torque_metrics.switching_latency_ns <= 1.0);
    assert!(dialog.cached_torque_metrics.thermal_stability_factor >= 60.0);
    assert!(dialog.cached_head_metrics.directional_isolation_db >= 30.0);
    assert!(dialog.cached_head_metrics.focal_spot_fwhm_nm <= 45.0);
    assert!(dialog.cached_crossbar_metrics.measured_tmr_percent >= 180.0);
    assert!(dialog.cached_crossbar_metrics.crosstalk_isolation_db >= 35.0);

    // 2. Adjust acoustic spin torque parameters and recompute
    dialog.acoustic_strain_amplitude = 2.2e-4;
    dialog.operating_temp_k = 0.050; // 50 mK
    dialog.gilbert_damping_alpha = 0.015;
    dialog.recompute();

    assert!(dialog.cached_torque_metrics.switching_latency_ns <= 1.0);
    assert!(dialog.cached_torque_metrics.critical_strain_amplitude <= 2.5e-4);

    // 3. Adjust writing head parameters and recompute
    dialog.polariton_coupling_mhz = 55.0;
    dialog.spot_size_nm = 38.0;
    dialog.write_power_mw = 3.0;
    dialog.recompute();

    assert!(dialog.cached_head_metrics.directional_isolation_db >= 30.0);
    assert!(dialog.cached_head_metrics.focal_spot_fwhm_nm <= 45.0);

    // 4. Modify crossbar cells and recompute
    dialog.selected_row = 3;
    dialog.selected_col = 5;
    let target_idx = 3 * 8 + 5;
    dialog.cached_crossbar_cells[target_idx] = MemoryCellState::AntiParallelState1;
    dialog.tmr_ratio_percent = 220.0;
    dialog.recompute();

    assert_eq!(
        dialog.cached_crossbar_cells[target_idx],
        MemoryCellState::AntiParallelState1
    );
    assert!(dialog.cached_crossbar_metrics.measured_tmr_percent >= 200.0);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_chiral_spintorque_dialog_headless_render() {
    let mut dialog = ChiralSpinTorqueMemoryDialog::new_fast();
    dialog.is_open = true;

    let ctx = Context::default();

    // Test render pass for all 5 tabs
    let tabs = [
        MemoryDialogTab::AcousticSpinTorqueDynamics,
        MemoryDialogTab::PolaritonWritingHead,
        MemoryDialogTab::SpintronicCrossbarGrid,
        MemoryDialogTab::CryogenicReadout,
        MemoryDialogTab::AuditTelemetry,
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
