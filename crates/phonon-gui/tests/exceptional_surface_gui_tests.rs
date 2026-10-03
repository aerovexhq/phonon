#![deny(unsafe_code)]

//! Test suite for Phase 346: ExceptionalSurfaceDialog in Phonon CAD Studio.
//!
//! Verifies:
//! - ExceptionalSurfaceDialog initialization and default parameters.
//! - Parameter adjustment and steady-state recomputation.
//! - Optimal operating point application and re-locking to the exceptional surface manifold.
//! - Headless egui Context execution pass and plot generation.

use phonon_gui::widgets::exceptional_surface_dialog::ExceptionalSurfaceDialog;

#[test]
fn test_dialog_initialization_and_defaults() {
    let dialog = ExceptionalSurfaceDialog::new();

    // Dialog must be closed by default
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Default physical parameters
    assert_eq!(dialog.omega_0_mhz, 10.0);
    assert_eq!(dialog.gamma_1_mhz, 2.0);
    assert_eq!(dialog.gamma_2_mhz, 1.0);
    assert!(
        (dialog.kappa_mhz - 0.5316).abs() < 0.01,
        "Default kappa should be ~0.5316 MHz: got {}",
        dialog.kappa_mhz
    );
    assert_eq!(dialog.incident_angle_deg, 0.0);
    assert_eq!(dialog.perturbation_eps, 1e-3);
    assert_eq!(dialog.num_elements, 8);

    // Initial exceptional surface metrics
    assert!(
        dialog.eigenvalues.min_splitting < 0.15,
        "System must initialize on the exceptional surface: got splitting {} MHz",
        dialog.eigenvalues.min_splitting
    );
    assert!(
        dialog.directional_metrics.enhancement_factor > 10.0,
        "Enhancement factor must be > 10x: got {}",
        dialog.directional_metrics.enhancement_factor
    );
    assert!(
        dialog.directional_metrics.directivity_db >= 25.0,
        "Directivity must be >= 25.0 dB: got {} dB",
        dialog.directional_metrics.directivity_db
    );

    // Cached curves verification
    assert!(!dialog.polar_curve.is_empty(), "Polar curve must be populated");
    assert!(!dialog.sensitivity_curve_nh.is_empty(), "NH sensitivity curve must be populated");
    assert!(!dialog.sensitivity_curve_herm.is_empty(), "Hermitian baseline curve must be populated");
    assert!(!dialog.riemann_branch_1.is_empty(), "Riemann sheet branch 1 must be populated");
}

#[test]
fn test_parameter_adjustment_and_recompute() {
    let mut dialog = ExceptionalSurfaceDialog::new();

    // Adjust incident acoustic angle and perturbation
    dialog.incident_angle_deg = 45.0;
    dialog.perturbation_eps = 5e-4;
    dialog.recompute();

    assert_eq!(dialog.incident_angle_deg, 45.0);
    assert_eq!(dialog.perturbation_eps, 5e-4);
    assert!(!dialog.polar_curve.is_empty());
    assert!(dialog.directional_metrics.enhancement_factor > 1.0);
}

#[test]
fn test_optimal_operating_point_preset() {
    let mut dialog = ExceptionalSurfaceDialog::new();

    // Change parameters to detuned point off the exceptional surface
    dialog.gamma_1_mhz = 2.5;
    dialog.gamma_2_mhz = 1.2;
    dialog.kappa_mhz = 4.0; // Deliberate detuning off-surface
    dialog.recompute();

    assert!(
        dialog.eigenvalues.min_splitting > 0.30,
        "Detuned point must have separated eigenvalues: got {} MHz",
        dialog.eigenvalues.min_splitting
    );

    // Re-lock to the exceptional surface manifold
    dialog.apply_optimal_operating_point();

    assert!(
        dialog.eigenvalues.min_splitting < 0.15,
        "After applying optimal operating point, must re-lock to exceptional surface: got {} MHz",
        dialog.eigenvalues.min_splitting
    );
    assert!(dialog.status_msg.contains("Locked to Exceptional Surface"));
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = ExceptionalSurfaceDialog::new();
    dialog.is_open = true;

    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    output.textures_delta.clear();

    assert!(dialog.is_open, "Dialog should remain open after render pass");
    assert!(!dialog.polar_curve.is_empty());
    assert!(!dialog.riemann_branch_1.is_empty());
}
