#![deny(unsafe_code)]

//! GUI Integration and Headless Render Test Suite for Phase 418:
//! Phonon Studio Quantum Metamaterial Topological Acoustic Floquet Spin-Hall
//! Insulator & Non-Reciprocal Cryogenic Circulator.

use egui::Context;
use phonon_gui::widgets::{FloquetSpinHallCirculatorDialog, FloquetSpinHallTab};

#[test]
fn test_floquet_spinhall_dialog_initialization_and_cold_boot() {
    let dialog = FloquetSpinHallCirculatorDialog::new_fast();

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, FloquetSpinHallTab::SpinHallWaveguide);
    assert_eq!(dialog.cached_dispersion.len(), 50); // 25 up + 25 down
    assert_eq!(dialog.cached_circulator_spectrum.len(), 25);
    assert_eq!(dialog.cached_readout_spectrum.len(), 25);
    assert!(!dialog.cached_realspace_field.is_empty());
    assert_eq!(dialog.cached_audit.total_pass_score, 10);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_floquet_spinhall_dialog_tab_switching() {
    let mut dialog = FloquetSpinHallCirculatorDialog::new_fast();

    let tabs = [
        (FloquetSpinHallTab::SpinHallWaveguide, "Spin-Hall Waveguide"),
        (FloquetSpinHallTab::FloquetCirculator, "Floquet Circulator"),
        (FloquetSpinHallTab::CryogenicReadout, "Cryogenic Qubit Readout"),
        (FloquetSpinHallTab::RealSpaceMetamaterial, "2D Real-Space Metamaterial"),
        (FloquetSpinHallTab::AuditTelemetry, "Physics Audit & Telemetry"),
    ];

    for (tab, label) in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert_eq!(dialog.active_tab.label(), label);
    }
}

#[test]
fn test_floquet_spinhall_dialog_recompute_and_parameter_updates() {
    let mut dialog = FloquetSpinHallCirculatorDialog::new_fast();

    // Modify parameters
    dialog.inter_intra_ratio = 1.20;
    dialog.pump_freq_mhz = 55.0;
    dialog.modulation_depth = 0.30;
    dialog.temperature_k = 0.020;
    dialog.obstacle_defect_enabled = true;

    dialog.recompute();

    assert_eq!(dialog.cached_dispersion.len(), 70); // 35 * 2
    assert_eq!(dialog.cached_circulator_spectrum.len(), 45);
    assert_eq!(dialog.cached_readout_spectrum.len(), 45);
    assert_eq!(dialog.cached_realspace_field.len(), 18);
    assert_eq!(dialog.cached_realspace_field[0].len(), 28);

    assert!(dialog.last_solve_time_us > 0.0);
    assert_eq!(dialog.cached_audit.total_pass_score, 10);
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_floquet_spinhall_dialog_headless_render() {
    let ctx = Context::default();
    let mut dialog = FloquetSpinHallCirculatorDialog::new_fast();
    dialog.is_open = true;

    let tabs = [
        FloquetSpinHallTab::SpinHallWaveguide,
        FloquetSpinHallTab::FloquetCirculator,
        FloquetSpinHallTab::CryogenicReadout,
        FloquetSpinHallTab::RealSpaceMetamaterial,
        FloquetSpinHallTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        output.textures_delta.clear();
    }
}
