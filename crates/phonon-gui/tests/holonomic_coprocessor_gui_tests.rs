#![deny(unsafe_code)]

//! Automated GUI test suite for Phase 416: Phonon Studio Quantum Metamaterial
//! Non-Abelian Holonomic Geometric Braiding & Monolithic CMOS-MEMS Co-Processor Dialog.

use egui::Context;
use phonon_gui::widgets::{HolonomicCoprocessorDialog, HolonomicCoprocessorTab};
use phonon_solver::holonomic_braiding_coprocessor::HolonomicGateKind;

#[test]
fn test_dialog_initialization_and_fast_boot() {
    let dialog = HolonomicCoprocessorDialog::new_fast();
    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        HolonomicCoprocessorTab::HolonomicGates
    );
    assert_eq!(dialog.gate_kind, HolonomicGateKind::Hadamard);
    assert!(dialog.cached_gate_metrics.is_purely_geometric);
    assert!(dialog.cached_braid_metrics.artin_relation_verified);
    assert!(dialog.cached_cmos_metrics.rise_time_ns <= 5.0);
    assert!(dialog.cached_cmos_metrics.total_cryogenic_dissipation_uw <= 50.0);

    // Verify precomputed caches
    assert!(!dialog.cached_gate_trajectory.is_empty());
    assert!(!dialog.cached_braid_trajectory.is_empty());
    assert!(!dialog.cached_pulse_waveform.is_empty());
    assert!(!dialog.cached_channel_statuses.is_empty());
    assert!(!dialog.cached_parity_spectrum.is_empty());
    assert!(dialog.cached_audit.all_passed);
}

#[test]
fn test_tab_switching() {
    let mut dialog = HolonomicCoprocessorDialog::new();
    let tabs = [
        HolonomicCoprocessorTab::HolonomicGates,
        HolonomicCoprocessorTab::MajoranaBraiding,
        HolonomicCoprocessorTab::CmosMemsDriver,
        HolonomicCoprocessorTab::ParityReadout,
        HolonomicCoprocessorTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_parameter_adjustment_and_recomputation() {
    let mut dialog = HolonomicCoprocessorDialog::new();

    // Adjust parameters
    dialog.gate_kind = HolonomicGateKind::PhaseS;
    dialog.drive_rabi_freq_mhz = 35.0;
    dialog.topological_gap_mhz = 4.0;
    dialog.switching_rise_time_ns = 2.0;
    dialog.gate_voltage_v = 1.0;
    dialog.recompute();

    assert!(dialog.cached_gate_metrics.process_fidelity >= 0.999);
    assert!(dialog.cached_braid_metrics.topological_gap_mhz >= 2.0);
    assert!(dialog.cached_cmos_metrics.crosstalk_isolation_db >= 35.0);
    assert!(dialog.last_solve_time_us > 0.0);
}

#[test]
fn test_preset_switching() {
    let mut dialog = HolonomicCoprocessorDialog::new();

    // Preset 2: Fast Hadamard gate
    dialog.gate_kind = HolonomicGateKind::Hadamard;
    dialog.loop_duration_ns = 25.0;
    dialog.recompute();
    assert_eq!(dialog.cached_gate_metrics.gate_duration_ns, 25.0);
    assert!(dialog.cached_gate_metrics.process_fidelity >= 0.999);

    // Preset 3: High-SNR Readout
    dialog.dispersive_shift_chi_mhz = 6.0;
    dialog.recompute();
    assert!(dialog.cached_braid_metrics.parity_readout_snr_db >= 16.0);
    assert_eq!(dialog.cached_braid_metrics.cavity_splitting_mhz, 12.0);
}

#[test]
fn test_headless_egui_render_pass() {
    let ctx = Context::default();
    let mut dialog = HolonomicCoprocessorDialog::new();
    dialog.is_open = true;

    // Render across all 5 tabs in headless mode
    let tabs = [
        HolonomicCoprocessorTab::HolonomicGates,
        HolonomicCoprocessorTab::MajoranaBraiding,
        HolonomicCoprocessorTab::CmosMemsDriver,
        HolonomicCoprocessorTab::ParityReadout,
        HolonomicCoprocessorTab::AuditTelemetry,
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
