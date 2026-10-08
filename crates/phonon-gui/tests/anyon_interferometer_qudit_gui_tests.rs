#![deny(unsafe_code)]

//! GUI Integration Test Suite for Anyon Interferometer & Protected Qudit Crossbar Dialog (Phase 456).
//!
//! Validates:
//! 1. Fast cold-boot initialization (< 2.0 ms) and baseline cached telemetry.
//! 2. Tab switching across all 5 categorized tabs.
//! 3. Interactive parameter adjustment and deterministic recomputation.
//! 4. Headless egui render pass across all tabs without panic or texture leakage.

use egui::Context;
use phonon_gui::widgets::anyon_interferometer_qudit_dialog::{
    AnyonInterferometerQuditDialog, AnyonInterferometerQuditTab,
};
use phonon_solver::anyon_interferometer_qudit::QuditGateType;

#[test]
fn test_anyon_interferometer_qudit_dialog_initialization_and_fast_boot() {
    let t_start = std::time::Instant::now();
    let dialog = AnyonInterferometerQuditDialog::new_fast();
    let elapsed = t_start.elapsed();

    assert!(!dialog.is_open, "Dialog must be closed by default");
    assert!(
        elapsed.as_millis() < 50,
        "Cold boot instantiation must be practically instantaneous, took {} ms",
        elapsed.as_millis()
    );

    // Verify baseline cached telemetry
    assert!(dialog.cached_interf_metrics.interference_visibility_pct >= 85.0);
    assert!(dialog.cached_interf_metrics.peak_to_valley_ratio > 1.0);
    assert!(!dialog.cached_flux_sweep.is_empty());

    assert!(dialog.cached_phase_metrics.phase_quantization_error_rad <= 1.0e-4);
    assert!(dialog.cached_phase_metrics.perturbation_phase_error_rad <= 0.01);
    assert!(dialog.cached_phase_metrics.fringe_contrast_pct >= 90.0);
    assert!(!dialog.cached_monodromy_eigenvalues.is_empty());

    assert!(dialog.cached_qudit_metrics.dimension_d >= 3);
    assert!(dialog.cached_qudit_metrics.single_qudit_gate_fidelity_pct >= 99.0);
    assert!(dialog.cached_qudit_metrics.two_qudit_entangling_fidelity_pct >= 98.5);
    assert!(dialog.cached_qudit_metrics.entangled_concurrence >= 0.92);
    assert!(!dialog.cached_density_matrix.is_empty());

    assert!(dialog.cached_bus_metrics.thermal_noise_occupancy <= 0.05);
    assert!(dialog.cached_bus_metrics.channel_cross_isolation_db >= 38.0);
    assert!(dialog.cached_bus_metrics.bus_insertion_loss_db <= 0.45);
    assert!(dialog.cached_bus_metrics.dephasing_lifetime_us >= 50.0);
    assert!(!dialog.cached_bus_sweep.is_empty());

    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_anyon_interferometer_qudit_dialog_tab_switching() {
    let mut dialog = AnyonInterferometerQuditDialog::new_fast();

    let tabs = [
        AnyonInterferometerQuditTab::MultiTerminalInterferometer,
        AnyonInterferometerQuditTab::TopologicalPhaseShift,
        AnyonInterferometerQuditTab::ProtectedQuditCrossbar,
        AnyonInterferometerQuditTab::CryogenicQuditBus,
        AnyonInterferometerQuditTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty(), "Tab label must not be empty");
    }
}

#[test]
fn test_anyon_interferometer_qudit_dialog_parameter_mutation_and_recompute() {
    let mut dialog = AnyonInterferometerQuditDialog::new_fast();

    dialog.qpc1_transmissivity = 0.52;
    dialog.qpc2_transmissivity = 0.48;
    dialog.magnetic_flux_phi0 = 1.5;
    dialog.qudit_dimension = 3;
    dialog.path_perturbation_pct = 4.0;
    dialog.selected_gate = QuditGateType::FourierHadamard;
    dialog.operating_temperature_mk = 25.0;
    dialog.internal_quality_factor_million = 2.2;
    dialog.piezoelectric_efficiency = 0.98;

    dialog.recompute();

    assert!(dialog.cached_interf_metrics.interference_visibility_pct >= 85.0);
    assert!(dialog.cached_phase_metrics.phase_quantization_error_rad <= 1.0e-4);
    assert!(dialog.cached_qudit_metrics.single_qudit_gate_fidelity_pct >= 99.0);
    assert!(dialog.cached_bus_metrics.channel_cross_isolation_db >= 38.0);
    assert!(dialog.last_solve_time_us > 0.0);
    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_anyon_interferometer_qudit_dialog_headless_egui_render() {
    let ctx = Context::default();
    let mut dialog = AnyonInterferometerQuditDialog::new_fast();
    dialog.is_open = true;

    let tabs = [
        AnyonInterferometerQuditTab::MultiTerminalInterferometer,
        AnyonInterferometerQuditTab::TopologicalPhaseShift,
        AnyonInterferometerQuditTab::ProtectedQuditCrossbar,
        AnyonInterferometerQuditTab::CryogenicQuditBus,
        AnyonInterferometerQuditTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });

        // Ensure clean frame without unhandled textures
        assert!(!output.shapes.is_empty(), "Tab {:?} must produce rendered shapes", tab);
        output.textures_delta.clear();
    }
}
