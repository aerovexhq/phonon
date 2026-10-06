#![deny(unsafe_code)]

//! Integration & Cold-Boot Performance Tests for Phonon-Magnon Polariton Transducer GUI.

use std::time::Instant;
use egui::Context;
use phonon_gui::{PhononMagnonDialog, PhononMagnonTab};

#[test]
fn test_polariton_dialog_cold_boot_latency() {
    let start = Instant::now();
    let dialog = PhononMagnonDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "Cold boot constructor must complete in < 5ms for fast startup, took {:?}",
        elapsed
    );
    assert!(!dialog.is_open, "Dialog must be closed by default on cold boot");
    assert_eq!(dialog.active_tab, PhononMagnonTab::PolaritonDispersion);
}

#[test]
fn test_polariton_dialog_recalculation_and_metrics() {
    let dialog = PhononMagnonDialog::new();

    assert!(dialog.dispersion_points.len() >= 50, "Dispersion points must be generated");
    assert!(dialog.s_params.len() >= 50, "S-parameters must be generated");
    assert!(dialog.track_snapshot.x_positions_mm.len() >= 50, "Track snapshot must be generated");

    assert!(
        dialog.peak_efficiency >= 0.60,
        "Peak conversion efficiency must be >= 60%, got {}",
        dialog.peak_efficiency
    );
    assert!(
        dialog.anticrossing_gap_mhz >= 50.0,
        "Avoided crossing gap must be >= 50 MHz, got {}",
        dialog.anticrossing_gap_mhz
    );
    assert!(
        dialog.quantum_cooperativity > 1.0,
        "Cooperativity must be > 1.0, got {}",
        dialog.quantum_cooperativity
    );
    assert!(
        dialog.quantum_fidelity >= 0.77,
        "Quantum fidelity must be >= 0.77, got {}",
        dialog.quantum_fidelity
    );
}

#[test]
fn test_polariton_transducer_audit_pass_rate() {
    let dialog = PhononMagnonDialog::new();

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
        "Transducer audit must pass 10/10 criteria, got {:?}",
        dialog.audit_score
    );
}

#[test]
fn test_polariton_dialog_parameter_adjustments() {
    let mut dialog = PhononMagnonDialog::new();

    let initial_gap = dialog.anticrossing_gap_mhz;
    dialog.coupling_g_mhz = 60.0; // Increase coupling rate
    dialog.recalculate();

    assert!(
        dialog.anticrossing_gap_mhz > initial_gap,
        "Higher coupling rate must increase avoided crossing splitting"
    );

    dialog.temperature_mk = 100.0;
    dialog.recalculate();
    assert!(dialog.added_noise_quanta > 0.1);
}

#[test]
fn test_polariton_dialog_headless_render_pass() {
    let ctx = Context::default();
    let mut dialog = PhononMagnonDialog::new();
    dialog.is_open = true;

    let tabs = [
        PhononMagnonTab::PolaritonDispersion,
        PhononMagnonTab::TransducerSParameters,
        PhononMagnonTab::MagnetoelasticStrain,
        PhononMagnonTab::QuantumNoiseFidelity,
        PhononMagnonTab::TransducerAudit,
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
