#![deny(unsafe_code)]

//! Test suite for Phase 337: FqhBraidingDialog in CAD Studio.
//!
//! Verifies:
//! - FqhBraidingDialog initialization and default parameters.
//! - Braid sequence compilation and worldline generation.
//! - Parameter adjustment (filling fraction, B field, gate selection).
//! - Headless egui Context execution pass and plot generation.

use phonon_gui::widgets::fqh_braiding_dialog::FqhBraidingDialog;
use phonon_solver::fqh_braiding::{AnyonModelKind, FillingFraction, InterferometerType, TargetGate};

#[test]
fn test_dialog_initialization_and_default_parameters() {
    let dialog = FqhBraidingDialog::new();

    // Dialog must be closed by default
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Physical parameter defaults
    assert_eq!(dialog.anyon_model, AnyonModelKind::MooreReadPfaffian);
    assert_eq!(dialog.target_gate, TargetGate::Hadamard);
    assert_eq!(dialog.filling_fraction, FillingFraction::Nu5_2);
    assert_eq!(dialog.interferometer_type, InterferometerType::FabryPerot);
    assert!((dialog.magnetic_field_tesla - 4.0).abs() < 1e-9);
    assert!((dialog.gate_voltage_v - (-0.5)).abs() < 1e-9);
    assert!((dialog.temperature_mk - 20.0).abs() < 1e-9);
    assert!((dialog.area_um2 - 2.0).abs() < 1e-9);
    assert_eq!(dialog.bulk_anyon_count, 0);

    // Initial simulation cache must be populated
    assert!(
        dialog.synthesis_result.fidelity >= 0.99,
        "Default gate synthesis fidelity must exceed 99%: got {}",
        dialog.synthesis_result.fidelity
    );
    assert!(
        (dialog.measured_fano_factor - 0.25).abs() < 1e-6,
        "Default Fano factor for nu=5/2 must equal 0.25: got {}",
        dialog.measured_fano_factor
    );
    assert!(
        dialog.yb_residual_norm < 1e-10,
        "Yang-Baxter residual norm must be < 1e-10: got {}",
        dialog.yb_residual_norm
    );
    assert!(dialog.tunneling_prob > 0.0 && dialog.tunneling_prob < 1.0);
    assert!(dialog.shot_noise_density > 0.0);
    assert!(dialog.thermal_noise_floor > 0.0);

    // Plot curves must have sample points
    assert!(!dialog.ab_curve_active.is_empty());
    assert!(!dialog.ab_curve_alt.is_empty());
}

#[test]
fn test_braid_sequence_compilation_and_worldline_generation() {
    let mut dialog = FqhBraidingDialog::new();

    // 1. Compile Phase S gate
    dialog.target_gate = TargetGate::PhaseS;
    dialog.recompute();
    assert!(dialog.synthesis_result.fidelity >= 0.99);
    assert!(!dialog.synthesis_result.braid_sequence.generators.is_empty());

    // 2. Compile Pauli X gate
    dialog.target_gate = TargetGate::PauliX;
    dialog.recompute();
    assert!(dialog.synthesis_result.fidelity >= 0.99);

    // 3. Compile Pauli Z gate
    dialog.target_gate = TargetGate::PauliZ;
    dialog.recompute();
    assert!(dialog.synthesis_result.fidelity >= 0.99);

    // 4. Compile T gate
    dialog.target_gate = TargetGate::TGate;
    dialog.recompute();
    assert!(
        dialog.synthesis_result.fidelity >= 0.99,
        "T-gate synthesis fidelity must exceed 0.99: got {}",
        dialog.synthesis_result.fidelity
    );

    // 5. Compile two-qubit CNOT gate
    dialog.target_gate = TargetGate::Cnot;
    dialog.recompute();
    assert!(dialog.synthesis_result.fidelity >= 0.99);
    assert_eq!(dialog.synthesis_result.num_strands, 8);

    // 6. Switch to Fibonacci anyon model and compile Hadamard
    dialog.anyon_model = AnyonModelKind::Fibonacci;
    dialog.target_gate = TargetGate::Hadamard;
    dialog.recompute();
    assert!(
        dialog.synthesis_result.fidelity >= 0.99,
        "Fibonacci Hadamard fidelity must exceed 0.99: got {}",
        dialog.synthesis_result.fidelity
    );
    assert_eq!(dialog.synthesis_result.num_strands, 3);
}

#[test]
fn test_parameter_adjustment_in_gui_state() {
    let mut dialog = FqhBraidingDialog::new();

    // 1. Change filling fraction to Laughlin nu = 1/3
    dialog.filling_fraction = FillingFraction::Nu1_3;
    dialog.recompute();
    assert!(
        (dialog.measured_fano_factor - (1.0 / 3.0)).abs() < 1e-6,
        "Fano factor must update to 1/3 for nu=1/3: got {}",
        dialog.measured_fano_factor
    );

    // 2. Adjust enclosed area from 2.0 to 4.0 um^2 -> oscillation period Delta B must halve
    let initial_delta_b = dialog.interferometer.oscillation_period_delta_b();
    dialog.area_um2 = 4.0;
    dialog.recompute();
    let new_delta_b = dialog.interferometer.oscillation_period_delta_b();
    assert!(
        (new_delta_b - 0.5 * initial_delta_b).abs() < 1e-6,
        "Doubling area must halve Delta B: initial {}, new {}",
        initial_delta_b,
        new_delta_b
    );

    // 3. Switch back to nu = 5/2 and change bulk anyon count to odd parity (n_bulk = 1)
    dialog.filling_fraction = FillingFraction::Nu5_2;
    dialog.bulk_anyon_count = 1;
    dialog.recompute();
    assert_eq!(
        dialog.interferometer.bulk_parity_visibility(),
        0.0,
        "Odd bulk parity in nu=5/2 must extinguish visibility (V = 0.0)"
    );

    // Even parity (n_bulk = 2) must restore full visibility
    dialog.bulk_anyon_count = 2;
    dialog.recompute();
    assert_eq!(
        dialog.interferometer.bulk_parity_visibility(),
        1.0,
        "Even bulk parity must restore visibility (V = 1.0)"
    );
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = FqhBraidingDialog::new();
    dialog.is_open = true;

    // Headless egui Context execution pass
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    output.textures_delta.clear();

    assert!(dialog.is_open);
    assert!(!dialog.ab_curve_active.is_empty());
    assert!(!dialog.ab_curve_alt.is_empty());
}
