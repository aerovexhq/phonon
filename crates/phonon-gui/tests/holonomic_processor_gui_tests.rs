#![deny(unsafe_code)]

//! Test suite for Phase 341: HolonomicProcessorDialog in CAD Studio.
//!
//! Verifies:
//! - HolonomicProcessorDialog initialization and default parameters.
//! - Gate selection and trajectory cache updates.
//! - Parameter adjustments in GUI state (pulse amplitude, duration, decay rates).
//! - Headless egui Context execution pass and plot generation.

use phonon_gui::widgets::holonomic_processor_dialog::HolonomicProcessorDialog;
use phonon_solver::non_abelian_holonomic::HolonomicGateType;
use std::f64::consts::PI;

#[test]
fn test_dialog_initialization_and_defaults() {
    let dialog = HolonomicProcessorDialog::new();

    // Dialog must be closed by default
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Default target gate is Hadamard
    assert_eq!(dialog.target_gate, HolonomicGateType::Hadamard);

    // Default physical cavity parameters
    assert!((dialog.cavity_params.omega_0 - 50.0).abs() < 1e-9);
    assert!((dialog.cavity_params.tau_ns - 40.0).abs() < 1e-9);
    assert!((dialog.cavity_params.omega_1 - 5.0).abs() < 1e-9);
    assert!((dialog.cavity_params.kappa_1 - 10.0).abs() < 1e-9);

    // Initial simulation cache must be populated
    assert!(
        dialog.simulation.final_gate_fidelity >= 0.99,
        "Initial Hadamard gate fidelity must be >= 0.99: got {}",
        dialog.simulation.final_gate_fidelity
    );

    assert!(
        dialog.simulation.final_dynamical_phase_error.abs() < 1e-4,
        "Initial dynamical phase error must cancel to < 1e-4 rad: got {}",
        dialog.simulation.final_dynamical_phase_error
    );

    // Plot curves must have sample points
    assert!(!dialog.pulse_1_curve.is_empty());
    assert!(!dialog.pulse_2_curve.is_empty());
    assert!(!dialog.pulse_3_curve.is_empty());
    assert!(!dialog.geom_phase_curve.is_empty());
    assert!(!dialog.dyn_phase_curve.is_empty());
}

#[test]
fn test_gate_selection_and_trajectory_update() {
    let mut dialog = HolonomicProcessorDialog::new();

    // 1. Switch to Phase S gate
    dialog.target_gate = HolonomicGateType::PhaseS;
    dialog.recompute();
    assert_eq!(dialog.target_gate, HolonomicGateType::PhaseS);
    assert!(
        dialog.simulation.final_gate_fidelity >= 0.99,
        "Phase S gate fidelity must be >= 0.99: got {}",
        dialog.simulation.final_gate_fidelity
    );
    assert!(dialog.simulation.final_dynamical_phase_error.abs() < 1e-4);

    // 2. Switch to Pauli X gate
    dialog.target_gate = HolonomicGateType::PauliX;
    dialog.recompute();
    assert_eq!(dialog.target_gate, HolonomicGateType::PauliX);
    assert!(
        dialog.simulation.final_gate_fidelity >= 0.99,
        "Pauli X gate fidelity must be >= 0.99: got {}",
        dialog.simulation.final_gate_fidelity
    );

    // 3. Switch to Pauli Z gate
    dialog.target_gate = HolonomicGateType::PauliZ;
    dialog.recompute();
    assert_eq!(dialog.target_gate, HolonomicGateType::PauliZ);
    assert!(
        dialog.simulation.final_gate_fidelity >= 0.99,
        "Pauli Z gate fidelity must be >= 0.99: got {}",
        dialog.simulation.final_gate_fidelity
    );

    // 4. Switch to Rotation R_z(pi/2)
    dialog.target_gate = HolonomicGateType::RotationZ(PI / 2.0);
    dialog.recompute();
    assert!(
        dialog.simulation.final_gate_fidelity >= 0.99,
        "R_z(pi/2) fidelity must be >= 0.99: got {}",
        dialog.simulation.final_gate_fidelity
    );

    // 5. Switch to Rotation R_x(pi/2)
    dialog.target_gate = HolonomicGateType::RotationX(PI / 2.0);
    dialog.recompute();
    assert!(
        dialog.simulation.final_gate_fidelity >= 0.99,
        "R_x(pi/2) fidelity must be >= 0.99: got {}",
        dialog.simulation.final_gate_fidelity
    );
}

#[test]
fn test_parameter_adjustment_in_gui_state() {
    let mut dialog = HolonomicProcessorDialog::new();

    // 1. Adjust coupling pulse amplitude Omega_0 from 50 to 80 MHz
    dialog.cavity_params.omega_0 = 80.0;
    dialog.recompute();
    assert_eq!(dialog.cavity_params.omega_0, 80.0);
    assert!(dialog.simulation.final_gate_fidelity >= 0.99);

    // 2. Adjust gate duration tau from 40 to 60 ns
    dialog.cavity_params.tau_ns = 60.0;
    dialog.recompute();
    assert_eq!(dialog.cavity_params.tau_ns, 60.0);
    assert_eq!(
        *dialog.simulation.time_points_ns.last().unwrap(),
        60.0
    );

    // 3. Adjust cavity decay rates kappa to 25 kHz
    dialog.cavity_params.kappa_1 = 25.0;
    dialog.cavity_params.kappa_2 = 25.0;
    dialog.cavity_params.kappa_3 = 25.0;
    dialog.recompute();
    assert!(dialog.simulation.cavity_loss_decoupling_db > 10.0);

    // Verify dark state purity remains high (> 0.999)
    for &p in &dialog.simulation.dark_state_purity {
        assert!(p >= 0.999);
    }
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = HolonomicProcessorDialog::new();
    dialog.is_open = true;

    // Headless egui Context execution pass
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    output.textures_delta.clear();

    assert!(dialog.is_open);
    assert!(!dialog.pulse_1_curve.is_empty());
    assert!(!dialog.pulse_2_curve.is_empty());
    assert!(!dialog.pulse_3_curve.is_empty());
    assert!(!dialog.geom_phase_curve.is_empty());
    assert!(!dialog.dyn_phase_curve.is_empty());
}
