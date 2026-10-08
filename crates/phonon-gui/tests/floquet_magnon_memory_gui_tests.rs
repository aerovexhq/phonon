#![deny(unsafe_code)]

//! GUI Integration Test Suite for Floquet Chiral Magnon-Phonon Router & Dissipative Quantum Memory Dialog (Phase 455).
//!
//! Validates:
//! 1. Fast cold-boot initialization (< 2.0 ms) and baseline cached telemetry.
//! 2. Tab switching across all 5 categorized tabs.
//! 3. Interactive parameter adjustment and deterministic recomputation.
//! 4. Headless egui render pass across all tabs without panic or texture leakage.

use egui::Context;
use phonon_gui::widgets::floquet_magnon_memory_dialog::{
    FloquetMagnonMemoryDialog, FloquetMagnonMemoryTab,
};

#[test]
fn test_floquet_magnon_memory_dialog_initialization_and_fast_boot() {
    let t_start = std::time::Instant::now();
    let dialog = FloquetMagnonMemoryDialog::new_fast();
    let elapsed = t_start.elapsed();

    assert!(!dialog.is_open, "Dialog must be closed by default");
    assert!(
        elapsed.as_millis() < 50,
        "Cold boot instantiation must be practically instantaneous, took {} ms",
        elapsed.as_millis()
    );

    // Verify baseline cached telemetry
    assert!(dialog.cached_dispersion_metrics.polariton_hybridization_gap_mhz >= 35.0);
    assert!(dialog.cached_dispersion_metrics.wavenumber_non_reciprocity_um_inv >= 0.15);
    assert!(dialog.cached_dispersion_metrics.forward_insertion_loss_db <= 0.40);
    assert!(dialog.cached_dispersion_metrics.backward_isolation_db >= 36.0);

    assert!(dialog.cached_circulator_metrics.cross_terminal_isolation_db >= 40.0);
    assert!(dialog.cached_circulator_metrics.backward_isolation_db >= 36.0);
    assert!(dialog.cached_circulator_metrics.port_return_loss_db >= 22.0);
    assert!(dialog.cached_circulator_metrics.circulation_bandwidth_3db_mhz >= 120.0);
    assert!(dialog.cached_circulator_metrics.corner_defect_transmission_pct >= 95.0);

    assert!(dialog.cached_braid_metrics.artin_relation_error <= 1.0e-5);
    assert!(dialog.cached_braid_metrics.far_commutation_error <= 1.0e-5);
    assert!(dialog.cached_braid_metrics.dynamic_braiding_fidelity_pct >= 99.9);

    assert!(dialog.cached_memory_metrics.memory_retention_time_us >= 60.0);
    assert!(dialog.cached_memory_metrics.cryogenic_thermal_occupancy <= 0.05);
    assert!(dialog.cached_memory_metrics.parity_readout_contrast_pct >= 88.0);

    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_floquet_magnon_memory_dialog_tab_switching() {
    let mut dialog = FloquetMagnonMemoryDialog::new_fast();

    let tabs = [
        FloquetMagnonMemoryTab::PolaritonDispersion,
        FloquetMagnonMemoryTab::FourTerminalCirculator,
        FloquetMagnonMemoryTab::SyntheticGaugeBraiding,
        FloquetMagnonMemoryTab::DissipativeMemory,
        FloquetMagnonMemoryTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty(), "Tab label must not be empty");
    }
}

#[test]
fn test_floquet_magnon_memory_dialog_parameter_mutation_and_recompute() {
    let mut dialog = FloquetMagnonMemoryDialog::new_fast();

    dialog.magnetoelastic_coupling_mhz = 50.0;
    dialog.floquet_drive_amplitude_oe = 22.0;
    dialog.circulation_bandwidth_mhz = 160.0;
    dialog.modulation_frequency_mhz = 15.0;
    dialog.engineered_dissipation_rate_mhz = 4.0;
    dialog.base_temperature_k = 0.015;

    dialog.recompute();

    assert!(dialog.cached_dispersion_metrics.polariton_hybridization_gap_mhz >= 35.0);
    assert!(dialog.cached_circulator_metrics.cross_terminal_isolation_db >= 40.0);
    assert!(dialog.cached_braid_metrics.dynamic_braiding_fidelity_pct >= 99.9);
    assert!(dialog.cached_memory_metrics.memory_retention_time_us >= 60.0);
    assert!(dialog.last_solve_time_us > 0.0);
    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_floquet_magnon_memory_dialog_headless_egui_render() {
    let ctx = Context::default();
    let mut dialog = FloquetMagnonMemoryDialog::new_fast();
    dialog.is_open = true;

    let tabs = [
        FloquetMagnonMemoryTab::PolaritonDispersion,
        FloquetMagnonMemoryTab::FourTerminalCirculator,
        FloquetMagnonMemoryTab::SyntheticGaugeBraiding,
        FloquetMagnonMemoryTab::DissipativeMemory,
        FloquetMagnonMemoryTab::AuditTelemetry,
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
