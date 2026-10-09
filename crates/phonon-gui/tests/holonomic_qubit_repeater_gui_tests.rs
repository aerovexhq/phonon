#![deny(unsafe_code)]

use egui::Context;
use phonon_gui::widgets::holonomic_qubit_repeater_dialog::{
    HolonomicQubitRepeaterDialog, HolonomicQubitRepeaterTab,
};
use phonon_solver::holonomic_qubit_repeater::HolonomicQubitGate;
use std::time::Instant;

#[test]
fn test_holonomic_qubit_repeater_dialog_initialization_and_fast_boot() {
    let start = Instant::now();
    let dialog = HolonomicQubitRepeaterDialog::new_fast();
    let duration = start.elapsed();

    assert!(
        duration.as_millis() < 5,
        "Fast boot latency must be < 5 ms, took {} ms",
        duration.as_millis()
    );

    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        HolonomicQubitRepeaterTab::HolonomicBraiding
    );

    assert_eq!(dialog.cached_audit.passed_count, 10);
    assert_eq!(dialog.cached_audit.total_count, 10);
    assert!(dialog.cached_audit.is_all_pass());

    assert!(dialog.cached_braiding_metrics.corner_confinement_ratio >= 0.90);
    assert!(dialog.cached_braiding_metrics.non_abelian_commutator_norm >= 0.70);
    assert!(dialog.cached_braiding_metrics.gate_process_fidelity >= 0.998);
    assert_eq!(dialog.cached_valley_metrics.valley_chern_contrast, 2);
    assert!((dialog.cached_valley_metrics.fractional_valley_charge - 1.0 / 3.0).abs() <= 0.01);
    assert!(dialog.cached_valley_metrics.reverse_chiral_isolation_db >= 42.0);
    assert!(dialog.cached_cryo_metrics.thermal_phonon_occupancy <= 1.0e-4);
    assert!(dialog.cached_cryo_metrics.quadrature_squeezing_db >= 7.0);
    assert!(dialog.cached_cryo_metrics.duan_simon_nullifier <= 0.35);
    assert!(dialog.cached_cryo_metrics.readout_snr_db >= 17.0);
}

#[test]
fn test_holonomic_qubit_repeater_dialog_parameter_mutation_and_recompute() {
    let mut dialog = HolonomicQubitRepeaterDialog::new_fast();

    dialog.intracell_gamma_mhz = 2.0;
    dialog.intercell_lambda_mhz = 14.0;
    dialog.braid_duration_ns = 110.0;
    dialog.selected_gate = HolonomicQubitGate::PhaseS;
    dialog.valley_staggering_mhz = 22.0;
    dialog.operating_temperature_mk = 10.0;
    dialog.num_repeater_nodes = 6;
    dialog.recompute();

    assert!(!dialog.cached_spatial_points.is_empty());
    assert!(!dialog.cached_loop_points.is_empty());
    assert!(!dialog.cached_valley_spectrum.is_empty());
    assert!(!dialog.cached_waveguide_points.is_empty());
    assert!(!dialog.cached_parity_spectrum.is_empty());
    assert_eq!(dialog.cached_repeater_nodes.len(), 6);

    assert!(dialog.cached_braiding_metrics.bulk_bandgap_mhz >= 18.0);
    assert!(dialog.cached_valley_metrics.reverse_chiral_isolation_db >= 42.0);
    assert!(dialog.cached_cryo_metrics.quadrature_squeezing_db >= 7.0);
    assert!(dialog.cached_audit.passed_count >= 9);
}

#[test]
fn test_holonomic_qubit_repeater_dialog_tab_switching() {
    let mut dialog = HolonomicQubitRepeaterDialog::new_fast();

    let tabs = [
        HolonomicQubitRepeaterTab::HolonomicBraiding,
        HolonomicQubitRepeaterTab::FractionalValleyRouter,
        HolonomicQubitRepeaterTab::CryogenicCoprocessor,
        HolonomicQubitRepeaterTab::CornerLatticeCanvas,
        HolonomicQubitRepeaterTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!dialog.active_tab.label().is_empty());
    }
}

#[test]
fn test_holonomic_qubit_repeater_dialog_headless_egui_render() {
    let mut dialog = HolonomicQubitRepeaterDialog::new_fast();
    dialog.is_open = true;
    dialog.recompute();

    let ctx = Context::default();

    for tab in [
        HolonomicQubitRepeaterTab::HolonomicBraiding,
        HolonomicQubitRepeaterTab::FractionalValleyRouter,
        HolonomicQubitRepeaterTab::CryogenicCoprocessor,
        HolonomicQubitRepeaterTab::CornerLatticeCanvas,
        HolonomicQubitRepeaterTab::AuditTelemetry,
    ] {
        dialog.active_tab = tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_content(ui);
        });
        output.textures_delta.clear();
    }
}
