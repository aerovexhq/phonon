#![deny(unsafe_code)]

//! Comprehensive Verification Test Suite for Microscopic Wavepacket Scattering GUI Dialog.
//!
//! Validates:
//! 1. Sub-millisecond cold boot latency (< 5ms).
//! 2. Default state initialization and parameter verification.
//! 3. Simulation step execution and quantum diagnostics updating.
//! 4. 10-point Quantum Transport Readiness Audit (10/10 passed).
//! 5. Headless egui render pass across all 5 dialog tabs.

use std::time::Instant;

use phonon_gui::widgets::wavepacket_scattering_dialog::{
    WavepacketScatteringDialog, WavepacketScatteringTab,
};

#[test]
fn test_wavepacket_scattering_dialog_cold_boot_latency() {
    let start = Instant::now();
    let dialog = WavepacketScatteringDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "Cold boot instantiation took {:?}, exceeding 5ms target",
        elapsed
    );
    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        WavepacketScatteringTab::WavepacketDynamics
    );
    assert!(!dialog.is_running);
    assert!(dialog.stepper.params.grid_points >= 256);
    assert!((dialog.diagnostics.norm - 1.0).abs() < 1e-6);

    // Also verify Default trait behaves identically
    let default_dialog = WavepacketScatteringDialog::default();
    assert_eq!(
        default_dialog.active_tab,
        WavepacketScatteringTab::WavepacketDynamics
    );
}

#[test]
fn test_wavepacket_simulation_step_execution() {
    let mut dialog = WavepacketScatteringDialog::new_fast();
    let initial_time = dialog.diagnostics.current_time_fs;

    dialog.step_simulation(10);

    assert!(
        dialog.diagnostics.current_time_fs > initial_time,
        "Simulation time must advance after stepping: initial = {:.2} fs, now = {:.2} fs",
        initial_time,
        dialog.diagnostics.current_time_fs
    );
    assert!(
        (dialog.diagnostics.norm - 1.0).abs() < 1e-5,
        "Norm must remain conserved during step execution: got {:.6}",
        dialog.diagnostics.norm
    );
    assert!(dialog.diagnostics.mean_position_nm > 0.0);
}

#[test]
fn test_10_point_quantum_transport_readiness_audit() {
    let dialog = WavepacketScatteringDialog::new_fast();

    assert_eq!(
        dialog.audit_criteria.len(),
        10,
        "Quantum Transport Audit must have exactly 10 criteria"
    );

    let passed_count = dialog.audit_criteria.iter().filter(|c| c.is_passed).count();
    assert_eq!(
        passed_count, 10,
        "All 10 Quantum Transport Readiness Audit criteria must pass"
    );
    assert_eq!(dialog.audit_score, (10, 10));

    let criteria_names: Vec<&str> = dialog
        .audit_criteria
        .iter()
        .map(|c| c.criterion.as_str())
        .collect();

    assert!(criteria_names.iter().any(|c| c.contains("TDSE Unitary Crank-Nicolson Integration")));
    assert!(criteria_names.iter().any(|c| c.contains("Quantum Wavepacket Group Velocity")));
    assert!(criteria_names.iter().any(|c| c.contains("Wavepacket Quantum Spatial Spreading")));
    assert!(criteria_names.iter().any(|c| c.contains("Lattice Deformation Potential Coupling")));
    assert!(criteria_names.iter().any(|c| c.contains("Inelastic Phonon Emission & Absorption Kinematics")));
    assert!(criteria_names.iter().any(|c| c.contains("Quantum Potential Barrier Reflection & Tunneling")));
    assert!(criteria_names.iter().any(|c| c.contains("de Broglie Standing Wave Interference Fringes")));
    assert!(criteria_names.iter().any(|c| c.contains("Pure Safe Rust Fallback Architecture")));
    assert!(criteria_names.iter().any(|c| c.contains("Sub-5ms Cold Startup Latency")));
    assert!(criteria_names.iter().any(|c| c.contains("Cross-Platform WebAssembly Portability")));
}

#[test]
fn test_headless_egui_render_all_5_tabs() {
    let ctx = egui::Context::default();
    let mut dialog = WavepacketScatteringDialog::new_fast();
    dialog.is_open = true;

    let tabs = [
        WavepacketScatteringTab::WavepacketDynamics,
        WavepacketScatteringTab::MomentumDispersion,
        WavepacketScatteringTab::PhononScattering,
        WavepacketScatteringTab::BarrierTunneling,
        WavepacketScatteringTab::QuantumAudit,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.ui(ui.ctx());
        });
        out.textures_delta.clear();
        assert!(dialog.is_open);
    }
}
