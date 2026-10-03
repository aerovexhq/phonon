#![deny(unsafe_code)]

//! Test suite for Phase 334: ExceptionalPointDialog in CAD Studio.
//!
//! Verifies:
//! - Dialog initialization and physical parameter defaults.
//! - Perturbation sweep and sub-threshold sensitivity enhancement calculation.
//! - PT-symmetric coupled RLC circuit transient waveform simulation.
//! - Headless egui UI execution pass and plot generation.

use phonon_gui::widgets::exceptional_point_dialog::ExceptionalPointDialog;
use phonon_solver::ep_sensor::{EpOrder, PtPhase};

#[test]
fn test_dialog_initialization_defaults() {
    let dialog = ExceptionalPointDialog::new();

    // Dialog is closed by default
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Physical parameter defaults
    assert_eq!(dialog.ep_order, EpOrder::EP2);
    assert!((dialog.perturbation_epsilon - 0.005).abs() < 1e-9);
    assert_eq!(dialog.gain_loss_gamma_mhz, 5.0);
    assert_eq!(dialog.coupling_kappa_mhz, 5.0);
    assert_eq!(dialog.bare_frequency_w0_mhz, 100.0);

    // Initial simulation cache must be populated
    assert!(
        dialog.circuit_state.is_some(),
        "Circuit state must be pre-computed"
    );
    assert!(
        !dialog.riemann_loop_points.is_empty(),
        "Riemann surface loop points must be pre-computed"
    );
    assert!(
        !dialog.bifurcation_branch_1.is_empty(),
        "Bifurcation branch 1 must be pre-computed"
    );
    assert!(
        !dialog.bifurcation_branch_2.is_empty(),
        "Bifurcation branch 2 must be pre-computed"
    );
    assert!(
        !dialog.sensitivity_curve_ep.is_empty(),
        "EP sensitivity curve must be pre-computed"
    );
    assert!(
        !dialog.sensitivity_curve_hermitian.is_empty(),
        "Hermitian sensitivity curve must be pre-computed"
    );

    // Telemetry defaults
    assert_eq!(dialog.pt_phase, PtPhase::ExceptionalPoint);
    assert!(dialog.petermann_factor > 10.0);
    assert!(
        dialog.sensitivity_enhancement > 50.0,
        "EP2 enhancement {} must exceed 50x for epsilon=0.005",
        dialog.sensitivity_enhancement
    );
    assert!(dialog.eigenvalue_splitting_mhz > 0.0);
}

#[test]
fn test_perturbation_sweep_and_sensitivity_ratio_calculation() {
    let mut dialog = ExceptionalPointDialog::new();
    dialog.ep_order = EpOrder::EP3;

    // 1. Moderate perturbation epsilon = 1e-3
    dialog.perturbation_epsilon = 1e-3;
    dialog.run_simulation();
    let enh_moderate = dialog.sensitivity_enhancement;
    assert!(
        enh_moderate > 100.0,
        "EP3 enhancement at 1e-3 must exceed 100x: got {}",
        enh_moderate
    );

    // 2. Weak perturbation epsilon = 1e-5
    dialog.perturbation_epsilon = 1e-5;
    dialog.run_simulation();
    let enh_weak = dialog.sensitivity_enhancement;

    // Power-law sensitivity scaling: Delta lambda ~ epsilon^(1/3) implies enh = Delta lambda / epsilon ~ epsilon^(-2/3)
    // Decreasing epsilon by 100x should increase sensitivity enhancement by ~ 100^(2/3) ~ 21.5x
    assert!(
        enh_weak > enh_moderate * 10.0,
        "Sub-threshold EP3 enhancement {} must dramatically exceed moderate enhancement {}",
        enh_weak,
        enh_moderate
    );

    // Verify sensitivity curve curves have 51 sample points
    assert_eq!(dialog.sensitivity_curve_ep.len(), 51);
    assert_eq!(dialog.sensitivity_curve_hermitian.len(), 51);

    // Across small perturbations, EP sensitivity curve must strictly lie above linear Hermitian curve
    for (ep_pt, herm_pt) in dialog
        .sensitivity_curve_ep
        .iter()
        .zip(dialog.sensitivity_curve_hermitian.iter())
    {
        assert!(
            ep_pt[1] >= herm_pt[1] - 1e-9,
            "EP sensitivity {} must dominate Hermitian baseline {}",
            ep_pt[1],
            herm_pt[1]
        );
    }
}

#[test]
fn test_pt_circuit_waveform_simulation() {
    let mut dialog = ExceptionalPointDialog::new();

    // 1. Exact PT Phase: gamma = 2.0 MHz, kappa = 5.0 MHz
    dialog.gain_loss_gamma_mhz = 2.0;
    dialog.coupling_kappa_mhz = 5.0;
    dialog.run_simulation();

    assert_eq!(dialog.pt_phase, PtPhase::Exact);
    let state_exact = dialog.circuit_state.as_ref().expect("State must exist");
    assert_eq!(state_exact.time.len(), 1001);
    assert_eq!(state_exact.v1.len(), 1001);
    assert_eq!(state_exact.v2.len(), 1001);

    let max_env_exact = state_exact.envelope.iter().cloned().fold(0.0f64, f64::max);
    assert!(
        max_env_exact < 20.0,
        "Exact phase envelope {} must remain bounded",
        max_env_exact
    );

    // 2. Broken PT Phase: gamma = 8.0 MHz, kappa = 5.0 MHz
    dialog.gain_loss_gamma_mhz = 8.0;
    dialog.coupling_kappa_mhz = 5.0;
    dialog.run_simulation();

    assert_eq!(dialog.pt_phase, PtPhase::Broken);
    let state_broken = dialog.circuit_state.as_ref().expect("State must exist");
    let final_env_broken = *state_broken.envelope.last().unwrap();
    assert!(
        final_env_broken > 2.0,
        "Broken phase final envelope {} must undergo exponential growth",
        final_env_broken
    );
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = ExceptionalPointDialog::new();
    dialog.is_open = true;

    // Headless egui Context execution pass
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    output.textures_delta.clear();

    assert!(dialog.is_open);
    assert!(!dialog.riemann_loop_points.is_empty());
    assert!(!dialog.bifurcation_branch_1.is_empty());
    assert!(!dialog.sensitivity_curve_ep.is_empty());
    assert!(dialog.circuit_state.is_some());
}
