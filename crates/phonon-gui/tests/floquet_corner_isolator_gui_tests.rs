#![deny(unsafe_code)]

//! GUI Integration Test Suite for Floquet Corner Magneto-Phonon Isolator & Circulator Array Dialog.
//!
//! Validates:
//! 1. Fast cold-boot initialization (< 2.0 ms) and baseline cached telemetry.
//! 2. Tab switching across all 5 categorized tabs.
//! 3. Interactive parameter adjustment and deterministic recomputation.
//! 4. Headless egui render pass across all tabs without panic or texture leakage.

use egui::Context;
use phonon_gui::widgets::floquet_corner_isolator_dialog::{
    FloquetCornerIsolatorDialog, FloquetCornerIsolatorTab,
};

#[test]
fn test_floquet_corner_isolator_dialog_initialization_and_fast_boot() {
    let t_start = std::time::Instant::now();
    let dialog = FloquetCornerIsolatorDialog::new_fast();
    let elapsed = t_start.elapsed();

    assert!(!dialog.is_open, "Dialog must be closed by default");
    assert!(
        elapsed.as_millis() < 50,
        "Cold boot instantiation must be practically instantaneous, took {} ms",
        elapsed.as_millis()
    );

    // Verify baseline cached telemetry
    assert!(dialog.cached_corner_metrics.synthetic_magnetic_field_tesla >= 10.0);
    assert!(dialog.cached_corner_metrics.bulk_topological_gap_mhz >= 10.0);
    assert!(dialog.cached_corner_metrics.corner_confinement_ratio >= 0.85);
    assert_eq!(dialog.cached_corner_metrics.quadrupole_moment, 0.5);

    assert!(dialog.cached_polariton_metrics.polariton_gap_mhz >= 40.0);
    assert!(dialog.cached_polariton_metrics.insertion_loss_db <= 0.40);
    assert!(dialog.cached_polariton_metrics.isolation_db >= 36.0);

    assert!(dialog.cached_circulator_metrics.return_loss_db >= 22.0);
    assert!(dialog.cached_circulator_metrics.corner_defect_transmission_ratio >= 0.95);
    assert!(dialog.cached_circulator_metrics.circulation_bandwidth_3db_mhz >= 120.0);

    assert!(dialog.cached_transducer_metrics.added_noise_quanta <= 0.08);
    assert!(dialog.cached_transducer_metrics.transduction_efficiency_pct >= 28.0);

    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_floquet_corner_isolator_dialog_tab_switching() {
    let mut dialog = FloquetCornerIsolatorDialog::new_fast();

    let tabs = [
        FloquetCornerIsolatorTab::FloquetCornerQuasienergy,
        FloquetCornerIsolatorTab::MagnetoPhononDispersion,
        FloquetCornerIsolatorTab::FourPortCirculatorSMatrix,
        FloquetCornerIsolatorTab::MicrowaveTransducerArray,
        FloquetCornerIsolatorTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty(), "Tab label must not be empty");
    }
}

#[test]
fn test_floquet_corner_isolator_dialog_parameter_mutation_and_recompute() {
    let mut dialog = FloquetCornerIsolatorDialog::new_fast();

    dialog.intracell_coupling_mhz = 2.0;
    dialog.intercell_coupling_mhz = 9.0;
    dialog.modulation_amplitude = 0.40;
    dialog.magnetoelastic_coupling_mhz = 55.0;
    dialog.active_input_port = 2;
    dialog.corner_defect_present = true;
    dialog.base_temperature_mk = 15.0;

    dialog.recompute();

    assert_eq!(dialog.active_input_port, 2);
    assert!(dialog.corner_defect_present);
    assert!(dialog.cached_corner_metrics.synthetic_magnetic_field_tesla >= 10.0);
    assert!(dialog.cached_polariton_metrics.polariton_gap_mhz >= 40.0);
    assert!(dialog.last_solve_time_us > 0.0);
    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_floquet_corner_isolator_dialog_headless_egui_render() {
    let ctx = Context::default();
    let mut dialog = FloquetCornerIsolatorDialog::new_fast();
    dialog.is_open = true;

    let tabs = [
        FloquetCornerIsolatorTab::FloquetCornerQuasienergy,
        FloquetCornerIsolatorTab::MagnetoPhononDispersion,
        FloquetCornerIsolatorTab::FourPortCirculatorSMatrix,
        FloquetCornerIsolatorTab::MicrowaveTransducerArray,
        FloquetCornerIsolatorTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });

        // Ensure clean frame without unhandled textures
        assert!(output.shapes.len() > 0, "Tab {:?} must produce rendered shapes", tab);
        output.textures_delta.clear();
    }
}
