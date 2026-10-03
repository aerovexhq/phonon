#![deny(unsafe_code)]

//! Test suite for Phase 344: KerrMicrocombDialog in Phonon CAD Studio.
//!
//! Verifies:
//! - KerrMicrocombDialog initialization and default parameters.
//! - Preset selection (Single Soliton vs Turing Rolls) and state updates.
//! - Detuning slider adjustment and steady-state recomputation.
//! - Detuning sweep scan and soliton step plateau mapping.
//! - Headless egui Context execution pass and plot generation.

use phonon_gui::widgets::kerr_microcomb_dialog::KerrMicrocombDialog;
use phonon_solver::kerr_microcomb::MicrocombRegime;

#[test]
fn test_dialog_initialization_and_defaults() {
    let dialog = KerrMicrocombDialog::new();

    // Dialog must be closed by default
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Default physical parameters
    assert_eq!(dialog.detuning_alpha, 3.5);
    assert_eq!(dialog.pump_power_f2, 4.0);
    assert_eq!(dialog.dispersion_d2_khz, 10.0);

    // Initial microcomb regime and spectrum metrics
    assert_eq!(dialog.solver.state.regime, MicrocombRegime::DissipativeSoliton);
    assert!(
        dialog.spectrum.peak_intensity > 1.0,
        "Soliton peak intensity should be > 1.0, got {}",
        dialog.spectrum.peak_intensity
    );
    assert!(
        dialog.spectrum.tau_fwhm_ps > 0.0,
        "Pulse FWHM should be positive, got {}",
        dialog.spectrum.tau_fwhm_ps
    );
    assert!(
        dialog.spectrum.bandwidth_3db_ghz() > 0.0,
        "3-dB bandwidth should be positive, got {}",
        dialog.spectrum.bandwidth_3db_ghz()
    );

    // Cached curves verification
    assert!(!dialog.temporal_curve.is_empty(), "Temporal curve must be populated");
    assert!(!dialog.sech2_curve.is_empty(), "Sech^2 fit curve must be populated");
    assert!(!dialog.osa_bars.is_empty(), "OSA comb bars must be populated");
    assert!(!dialog.scan_curve.is_empty(), "Detuning scan curve must be populated");
    assert!(dialog.scan_result.is_some(), "Scan result must be precomputed");
}

#[test]
fn test_preset_selection() {
    let mut dialog = KerrMicrocombDialog::new();

    // Switch to Turing Rolls preset
    dialog.apply_turing_roll_preset();
    assert_eq!(dialog.detuning_alpha, 1.8);
    assert_eq!(dialog.pump_power_f2, 2.56);
    assert_eq!(dialog.solver.state.regime, MicrocombRegime::TuringRolls);
    assert!(!dialog.temporal_curve.is_empty());
    assert!(dialog.status_msg.contains("Turing"));

    // Switch back to Single Soliton preset
    dialog.apply_single_soliton_preset();
    assert_eq!(dialog.detuning_alpha, 3.5);
    assert_eq!(dialog.pump_power_f2, 4.0);
    assert_eq!(dialog.solver.state.regime, MicrocombRegime::DissipativeSoliton);
    assert!(dialog.spectrum.peak_intensity > 1.0);
    assert!(dialog.status_msg.contains("Single Soliton"));
}

#[test]
fn test_detuning_adjustment_and_recompute() {
    let mut dialog = KerrMicrocombDialog::new();

    // Adjust detuning to slightly larger value in soliton existence range
    dialog.detuning_alpha = 4.2;
    dialog.pump_power_f2 = 4.5;
    dialog.recompute();

    assert_eq!(dialog.solver.params.alpha, 4.2);
    assert!(!dialog.temporal_curve.is_empty());
    assert!(!dialog.osa_bars.is_empty());
    assert!(dialog.solver.state.mean_power > 0.0);
}

#[test]
fn test_sweep_detuning() {
    let mut dialog = KerrMicrocombDialog::new();

    dialog.sweep_detuning();

    assert!(dialog.scan_result.is_some());
    let scan = dialog.scan_result.as_ref().unwrap();
    assert!(!scan.alphas.is_empty());
    assert_eq!(scan.alphas.len(), scan.powers.len());
    assert!(!dialog.scan_curve.is_empty());
    assert!(dialog.status_msg.contains("sweep complete"));
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = KerrMicrocombDialog::new();
    dialog.is_open = true;

    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    output.textures_delta.clear();

    assert!(dialog.is_open, "Dialog should remain open after render pass");
    assert!(!dialog.temporal_curve.is_empty());
}
