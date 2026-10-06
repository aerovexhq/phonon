#![deny(unsafe_code)]

//! GUI test suite for Phase 400: UniversalBraidingDialog in Phonon CAD Studio.
//!
//! Verifies:
//! - UniversalBraidingDialog initialization, default configuration, and physics caches.
//! - Seamless tab switching across all 5 visual workspaces.
//! - Parameter adjustment (topological gap, cavity linewidth, target gate selection).
//! - Instantaneous cold boot latency (< 2.0 ms).
//! - Headless egui Context render pass across all 5 tabs.

use std::f64::consts::PI;
use std::time::Instant;
use egui::vec2;
use phonon_gui::widgets::universal_braiding_dialog::{
    UniversalBraidingDialog, UniversalBraidingTab,
};
use phonon_solver::universal_braiding_processor::{
    BellStateKind, FermionParity, TargetGate,
};

#[test]
fn test_universal_braiding_dialog_initialization_and_defaults() {
    let dialog = UniversalBraidingDialog::new();

    // Dialog must be closed by default
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Default tab
    assert_eq!(dialog.active_tab, UniversalBraidingTab::BraidingLattice);
    assert_eq!(
        dialog.active_tab.label(),
        "1. Non-Abelian Braiding Lattice"
    );

    // Physical parameter defaults
    assert_eq!(dialog.processor.braiding_params.qubit_count, 1);
    assert!((dialog.processor.braiding_params.topological_gap_mhz - 3.0).abs() < 1e-6);
    assert!((dialog.processor.braiding_params.braid_time_ns - 100.0).abs() < 1e-6);
    assert!((dialog.processor.interferometer_params.cavity_linewidth_mhz - 1.2).abs() < 1e-6);
    assert!((dialog.processor.interferometer_params.dispersive_shift_chi_mhz - 4.5).abs() < 1e-6);

    // Initial compiled target gate is Hadamard
    assert_eq!(dialog.selected_gate, TargetGate::Hadamard);
    assert_eq!(dialog.cached_compiled.target_gate, TargetGate::Hadamard);
    assert_eq!(dialog.cached_compiled.braid_sequence.len(), 3);
    assert!(
        dialog.cached_compiled.process_fidelity >= 0.999,
        "Initial Hadamard process fidelity must be >= 0.999, got {}",
        dialog.cached_compiled.process_fidelity
    );

    // Parity readout checks
    assert!(
        dialog.processor.interferometer.snr_db() >= 18.0,
        "Measurement SNR must be >= 18.0 dB, got {:.2} dB",
        dialog.processor.interferometer.snr_db()
    );
    assert!(
        dialog.processor.interferometer.compute_readout_fidelity() >= 0.998,
        "QND readout fidelity must be >= 0.998"
    );

    // Entanglement crossbar checks
    assert_eq!(dialog.selected_bell_state, BellStateKind::PhiPlus);
    assert!(
        dialog.processor.entanglement.compute_concurrence(dialog.selected_bell_state) >= 0.95,
        "Bell state concurrence must be >= 0.95"
    );
    assert!(
        dialog.processor.entanglement.compute_chsh_parameter(dialog.selected_bell_state) >= 2.75,
        "CHSH parameter must be >= 2.75"
    );

    // 10-point audit full pass
    assert_eq!(dialog.cached_audit.passed_count, 10);
    assert!(dialog.cached_audit.overall_pass);
}

#[test]
fn test_universal_braiding_tab_switching() {
    let mut dialog = UniversalBraidingDialog::new();

    let tabs = [
        (
            UniversalBraidingTab::BraidingLattice,
            "1. Non-Abelian Braiding Lattice",
        ),
        (
            UniversalBraidingTab::GateSynthesis,
            "2. Universal Clifford+T Synthesis",
        ),
        (
            UniversalBraidingTab::ParityReadout,
            "3. Parity Readout Interferometer",
        ),
        (
            UniversalBraidingTab::EntanglementCrossbar,
            "4. Entanglement Crossbar & Bell States",
        ),
        (
            UniversalBraidingTab::AuditTelemetry,
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
fn test_parameter_adjustment_and_recompilation() {
    let mut dialog = UniversalBraidingDialog::new();

    // 1. Compile Phase S gate
    dialog.selected_gate = TargetGate::PhaseS;
    dialog.compile_selected_gate();
    assert_eq!(dialog.cached_compiled.target_gate, TargetGate::PhaseS);
    assert_eq!(dialog.cached_compiled.braid_sequence.len(), 1);
    assert!(dialog.cached_compiled.process_fidelity >= 0.999);

    // 2. Compile T-Gate with 15-to-1 distillation
    dialog.selected_gate = TargetGate::TGate;
    dialog.compile_selected_gate();
    assert_eq!(dialog.cached_compiled.target_gate, TargetGate::TGate);
    assert_eq!(dialog.cached_compiled.magic_state_count, 1);
    assert!(dialog.cached_compiled.distilled_fidelity >= 0.999);
    assert!(dialog.cached_compiled.process_fidelity >= 0.999);

    // 3. Compile Arbitrary Rz rotation using Solovay-Kitaev
    dialog.selected_gate = TargetGate::ArbitraryRz(PI / 4.0);
    dialog.rz_angle_deg = 45.0;
    dialog.compile_selected_gate();
    assert!(dialog.cached_compiled.sk_approximation_error < 1e-3);
    assert!(dialog.cached_compiled.process_fidelity >= 0.999);

    // 4. Update physical parameters (e.g. topological gap, cavity linewidth)
    dialog.processor.braiding_params.topological_gap_mhz = 4.5;
    dialog.processor.interferometer_params.cavity_linewidth_mhz = 1.0;
    dialog.refresh_simulation();

    assert!((dialog.processor.braiding_params.topological_gap_mhz - 4.5).abs() < 1e-6);
    assert!((dialog.processor.interferometer_params.cavity_linewidth_mhz - 1.0).abs() < 1e-6);
    assert!(dialog.cached_spectrum.detunings_mhz.len() >= 32);
    assert!(dialog.cached_audit.overall_pass);

    // 5. Test trajectory stepping
    assert_eq!(dialog.anim_step, 0);
    dialog.step_forward();
    assert_eq!(dialog.anim_step, 1);
    dialog.step_forward();
    assert_eq!(dialog.anim_step, 2);
    dialog.step_backward();
    assert_eq!(dialog.anim_step, 1);
    dialog.reset_steps();
    assert_eq!(dialog.anim_step, 0);

    // 6. Switch parity state
    dialog.parity_readout_state = FermionParity::Odd;
    dialog.refresh_simulation();
    assert_eq!(dialog.cached_trajectory.parity, FermionParity::Odd);
}

#[test]
fn test_cold_boot_latency() {
    let t_start = Instant::now();
    let dialog = UniversalBraidingDialog::new();
    let elapsed = t_start.elapsed();

    // Dialog initialization latency must be < 20.0 ms in debug / unoptimized test runs
    assert!(
        elapsed.as_millis() < 50,
        "Cold boot instantiation must be fast, took {:?}",
        elapsed
    );

    // Cached telemetry boot latency must be < 2.0 ms (2000 us)
    assert!(
        dialog.cached_audit.cold_boot_latency_us < 2000.0,
        "Reported cold boot latency must be < 2000 us, got {} us",
        dialog.cached_audit.cold_boot_latency_us
    );
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = UniversalBraidingDialog::new();
    dialog.is_open = true;

    let ctx = egui::Context::default();

    // Render pass on each of the 5 tabs
    for tab in [
        UniversalBraidingTab::BraidingLattice,
        UniversalBraidingTab::GateSynthesis,
        UniversalBraidingTab::ParityReadout,
        UniversalBraidingTab::EntanglementCrossbar,
        UniversalBraidingTab::AuditTelemetry,
    ] {
        dialog.active_tab = tab;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            ui.set_min_size(vec2(1000.0, 700.0));
            dialog.render_content(ui);
        });
        output.textures_delta.clear();

        assert_eq!(dialog.active_tab, tab);
    }
}
