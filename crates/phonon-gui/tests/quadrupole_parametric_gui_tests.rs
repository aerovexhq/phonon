#![deny(unsafe_code)]

//! Integration & Cold-Boot Performance Tests for Topological Quadrupole Parametric Waveguide GUI.

use std::time::Instant;
use egui::Context;
use phonon_gui::{
    CanvasColormap, QuadrupoleParametricDialog, QuadrupoleParametricTab, RealSpaceMode,
};

#[test]
fn test_quadrupole_dialog_cold_boot_latency() {
    let start = Instant::now();
    let dialog = QuadrupoleParametricDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "Cold boot constructor must complete in < 5ms for fast startup, took {:?}",
        elapsed
    );
    assert!(!dialog.is_open, "Dialog must be closed by default on cold boot");
    assert_eq!(
        dialog.active_tab,
        QuadrupoleParametricTab::WaveguideTopology,
        "Dialog must open to Waveguide Topology tab by default"
    );
}

#[test]
fn test_quadrupole_dialog_recalculation_and_metrics() {
    let dialog = QuadrupoleParametricDialog::new();

    // 1. Simulation artifacts generated
    assert!(dialog.band_structure.len() >= 48, "Bulk band structure points must be generated");
    assert!(dialog.boundary_dispersion.len() >= 30, "Boundary dispersion points must be generated");
    assert!(dialog.shg_solver.trajectory.len() >= 50, "SHG trajectory points must be generated");
    assert!(dialog.shg_phase_curve.len() >= 30, "SHG phase curve points must be generated");
    assert!(!dialog.parametric_amp.gain_spectrum.is_empty(), "Parametric gain spectrum must be generated");

    // 2. Topological metrics
    assert!(
        dialog.waveguide.bulk_gap_mhz >= 10.0,
        "Bulk bandgap must be >= 10.0 MHz, got {}",
        dialog.waveguide.bulk_gap_mhz
    );
    assert_eq!(
        dialog.waveguide.quadrupole_moment, 0.5,
        "Quantized quadrupole moment must be 0.5 in SOTI phase"
    );
    assert!(
        dialog.waveguide.boundary_confinement >= 0.80,
        "Boundary confinement must be >= 80%, got {:.2}%",
        dialog.waveguide.boundary_confinement * 100.0
    );

    // 3. SHG non-linear metrics
    assert!(
        dialog.shg_solver.modal_overlap_integral >= 0.85,
        "Modal overlap integral must be >= 0.85, got {}",
        dialog.shg_solver.modal_overlap_integral
    );
    assert!(
        dialog.shg_solver.conversion_efficiency >= 0.10,
        "SHG conversion efficiency must be >= 10%, got {:.2}%",
        dialog.shg_solver.conversion_efficiency * 100.0
    );

    // 4. Parametric amplification metrics
    assert!(
        dialog.parametric_amp.metrics.gain_forward_db >= 20.0,
        "Forward gain must be >= 20.0 dB, got {:.2} dB",
        dialog.parametric_amp.metrics.gain_forward_db
    );
    assert!(
        dialog.parametric_amp.metrics.gain_backward_db <= 0.5,
        "Backward gain must be <= 0.5 dB, got {:.2} dB",
        dialog.parametric_amp.metrics.gain_backward_db
    );
    assert!(
        dialog.parametric_amp.metrics.isolation_db >= 25.0,
        "Directional isolation must be >= 25.0 dB, got {:.2} dB",
        dialog.parametric_amp.metrics.isolation_db
    );
    assert!(
        dialog.parametric_amp.metrics.added_noise_quanta <= 0.55,
        "Quantum added noise must be <= 0.55 quanta, got {:.5}",
        dialog.parametric_amp.metrics.added_noise_quanta
    );
}

#[test]
fn test_quadrupole_audit_checklist_full_pass() {
    let dialog = QuadrupoleParametricDialog::new();

    for crit in &dialog.audit_criteria {
        assert!(
            crit.is_passed,
            "Criterion '{}' failed: spec='{}', obs='{}'",
            crit.criterion,
            crit.specification,
            crit.observed_state
        );
    }

    assert_eq!(
        dialog.audit_score,
        (10, 10),
        "Waveguide audit must pass 10/10 criteria, got {:?}",
        dialog.audit_score
    );
}

#[test]
fn test_quadrupole_dialog_parameter_adjustments() {
    let mut dialog = QuadrupoleParametricDialog::new();

    // 1. Parametric coupling increase
    let initial_gain = dialog.parametric_amp.metrics.gain_forward_db;
    dialog.coupling_rate_mhz = 22.0;
    dialog.recalculate();
    assert!(
        dialog.parametric_amp.metrics.gain_forward_db > initial_gain,
        "Higher parametric coupling rate must increase forward gain"
    );

    // 2. Topological transition to trivial phase (gamma > lambda)
    dialog.gamma_mhz = 12.0;
    dialog.lambda_mhz = 4.0;
    dialog.recalculate();

    assert!(!dialog.waveguide.params.is_topological());
    assert_eq!(dialog.waveguide.quadrupole_moment, 0.0);
    assert_eq!(dialog.waveguide.edge_dipoles, (0.0, 0.0));
    assert!(dialog.waveguide.boundary_confinement < 0.40);

    // 3. Colormap and mode toggles
    dialog.selected_colormap = CanvasColormap::Turbo;
    dialog.selected_mode = RealSpaceMode::SecondHarmonic;
    assert_eq!(dialog.selected_colormap, CanvasColormap::Turbo);
    assert_eq!(dialog.selected_mode, RealSpaceMode::SecondHarmonic);
}

#[test]
fn test_quadrupole_dialog_headless_render_pass() {
    let ctx = Context::default();
    let mut dialog = QuadrupoleParametricDialog::new();
    dialog.is_open = true;

    let tabs = [
        QuadrupoleParametricTab::WaveguideTopology,
        QuadrupoleParametricTab::SecondHarmonicGeneration,
        QuadrupoleParametricTab::ParametricAmplification,
        QuadrupoleParametricTab::RealSpaceMetamaterial,
        QuadrupoleParametricTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.ui(ui.ctx());
        });
        out.textures_delta.clear();
        assert!(dialog.is_open, "Dialog must remain open across tab switching");
    }
}
