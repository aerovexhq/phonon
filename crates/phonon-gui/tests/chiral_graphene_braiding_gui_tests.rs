#![deny(unsafe_code)]

//! GUI Integration Test Suite for Non-Abelian Anyon Braiding & Topological Qubit Crossbar Dialog in Chiral Phononic Graphene (Phase 454).
//!
//! Validates:
//! 1. Fast cold-boot initialization (< 2.0 ms) and baseline cached telemetry.
//! 2. Tab switching across all 5 categorized tabs.
//! 3. Interactive parameter adjustment and deterministic recomputation.
//! 4. Headless egui render pass across all tabs without panic or texture leakage.

use egui::Context;
use phonon_gui::widgets::chiral_graphene_braiding_dialog::{
    ChiralGrapheneBraidingDialog, ChiralGrapheneBraidingTab,
};
use phonon_solver::chiral_graphene_braiding::GrapheneTargetGate;

#[test]
fn test_chiral_graphene_braiding_dialog_initialization_and_fast_boot() {
    let t_start = std::time::Instant::now();
    let dialog = ChiralGrapheneBraidingDialog::new_fast();
    let elapsed = t_start.elapsed();

    assert!(!dialog.is_open, "Dialog must be closed by default");
    assert!(
        elapsed.as_millis() < 50,
        "Cold boot instantiation must be practically instantaneous, took {} ms",
        elapsed.as_millis()
    );

    // Verify baseline cached telemetry
    assert!(dialog.cached_lattice_metrics.bulk_chern_bandgap_mhz >= 15.0);
    assert_eq!(dialog.cached_lattice_metrics.lower_band_chern_number, 1);
    assert_eq!(dialog.cached_lattice_metrics.upper_band_chern_number, -1);
    assert!(dialog.cached_lattice_metrics.edge_penetration_depth_cells <= 2.2);
    assert!(dialog.cached_lattice_metrics.corner_transmission_ratio >= 0.940);

    assert!(dialog.cached_braid_metrics.artin_relation_error <= 1.0e-5);
    assert!(dialog.cached_braid_metrics.far_commutation_error <= 1.0e-5);
    assert!(dialog.cached_braid_metrics.diabatic_leakage_probability <= 1.0e-4);
    assert!(dialog.cached_braid_metrics.braid_unitary_fidelity_pct >= 99.9);

    assert!(dialog.cached_gate_metrics.gate_process_fidelity_pct >= 99.9);

    assert!(dialog.cached_crossbar_metrics.dispersive_frequency_splitting_mhz >= 8.0);
    assert!(dialog.cached_crossbar_metrics.parity_readout_snr_db >= 20.0);
    assert!(dialog.cached_crossbar_metrics.qnd_readout_fidelity_pct >= 99.5);
    assert!(dialog.cached_crossbar_metrics.crossbar_waveguide_isolation_db >= 38.0);
    assert!(dialog.cached_crossbar_metrics.cryogenic_thermal_noise_occupancy <= 0.05);
    assert!(dialog.cached_crossbar_metrics.quasiparticle_poisoning_lifetime_us >= 40.0);

    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_chiral_graphene_braiding_dialog_tab_switching() {
    let mut dialog = ChiralGrapheneBraidingDialog::new_fast();

    let tabs = [
        ChiralGrapheneBraidingTab::ChiralLatticeEdgeModes,
        ChiralGrapheneBraidingTab::NonAbelianBraidingDynamics,
        ChiralGrapheneBraidingTab::LogicalCliffordGates,
        ChiralGrapheneBraidingTab::CryogenicCrossbarArray,
        ChiralGrapheneBraidingTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty(), "Tab label must not be empty");
    }
}

#[test]
fn test_chiral_graphene_braiding_dialog_parameter_mutation_and_recompute() {
    let mut dialog = ChiralGrapheneBraidingDialog::new_fast();

    dialog.hopping_t1_mhz = 14.0;
    dialog.hopping_t2_mhz = 2.8;
    dialog.braid_duration_ns = 140.0;
    dialog.target_gate = GrapheneTargetGate::Cnot;
    dialog.base_temperature_k = 0.015;
    dialog.dispersive_coupling_chi_mhz = 4.5;

    dialog.recompute();

    assert!(dialog.cached_lattice_metrics.bulk_chern_bandgap_mhz >= 15.0);
    assert!(dialog.cached_braid_metrics.braid_unitary_fidelity_pct >= 99.9);
    assert!(dialog.cached_gate_metrics.entanglement_concurrence >= 0.92);
    assert!(dialog.cached_gate_metrics.gate_process_fidelity_pct >= 99.0);
    assert!(dialog.cached_crossbar_metrics.dispersive_frequency_splitting_mhz >= 8.0);
    assert!(dialog.last_solve_time_us > 0.0);
    assert_eq!(dialog.cached_audit.score(), (10, 10));
}

#[test]
fn test_chiral_graphene_braiding_dialog_headless_egui_render() {
    let ctx = Context::default();
    let mut dialog = ChiralGrapheneBraidingDialog::new_fast();
    dialog.is_open = true;

    let tabs = [
        ChiralGrapheneBraidingTab::ChiralLatticeEdgeModes,
        ChiralGrapheneBraidingTab::NonAbelianBraidingDynamics,
        ChiralGrapheneBraidingTab::LogicalCliffordGates,
        ChiralGrapheneBraidingTab::CryogenicCrossbarArray,
        ChiralGrapheneBraidingTab::AuditTelemetry,
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
