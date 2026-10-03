#![deny(unsafe_code)]

//! Test suite for Phase 342: TwistedMoireDialog in CAD Studio.
//!
//! Verifies:
//! - TwistedMoireDialog initialization and default parameters.
//! - Twist angle slider adjustment and band structure recalculation.
//! - Parameter adjustments in GUI state (interlayer potentials, relaxation toggle, soliton amplitude).
//! - Headless egui Context execution pass and plot generation.

use phonon_gui::widgets::twisted_moire_dialog::TwistedMoireDialog;

#[test]
fn test_dialog_initialization_and_defaults() {
    let dialog = TwistedMoireDialog::new();

    // Dialog must be closed by default
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Default physical parameters at magic angle
    assert_eq!(dialog.params.theta_deg, 1.08);
    assert_eq!(dialog.params.a_0, 0.246);
    assert_eq!(dialog.params.w_aa, 80.0);
    assert_eq!(dialog.params.w_ab, 110.0);
    assert!(dialog.relaxation_enabled);
    assert_eq!(dialog.soliton_amplitude_mpa, 5.0);

    // Initial simulation telemetry
    assert!(
        dialog.flat_bandwidth_mev < 1.0,
        "Initial flat bandwidth at magic angle must be < 1.0 meV: got {} meV",
        dialog.flat_bandwidth_mev
    );
    assert!(
        dialog.group_velocity_ratio <= 0.05,
        "Initial group velocity ratio must be <= 0.05: got {}",
        dialog.group_velocity_ratio
    );
    assert!(
        dialog.aa_domain_fraction < 0.18,
        "Relaxed AA domain fraction must be < 18.0%: got {}",
        dialog.aa_domain_fraction
    );
    assert!(
        dialog.soliton_confinement_ratio < 0.4,
        "Soliton confinement ratio must be < 0.4: got {}",
        dialog.soliton_confinement_ratio
    );
    assert!(
        dialog.dos_peak_ratio > 10.0,
        "DOS peak-to-background ratio must be > 10: got {}",
        dialog.dos_peak_ratio
    );

    // Plot curves must be populated
    assert!(!dialog.flat_band_upper_curve.is_empty());
    assert!(!dialog.flat_band_lower_curve.is_empty());
    assert!(!dialog.dispersive_curves.is_empty());
    assert!(!dialog.dos_curve.is_empty());
    assert!(!dialog.soliton_curve.is_empty());
}

#[test]
fn test_twist_angle_slider_adjustment_and_recalculation() {
    let mut dialog = TwistedMoireDialog::new();
    let initial_lm = dialog.moire_period_nm;

    // 1. Adjust twist angle to large angle (1.80 deg)
    dialog.params.theta_deg = 1.80;
    dialog.recompute();

    assert_eq!(dialog.twist_angle_deg, 1.80);
    assert!(
        dialog.moire_period_nm < initial_lm,
        "Moiré period at 1.80 deg must be smaller than at 1.08 deg"
    );
    assert!(
        dialog.flat_bandwidth_mev > 5.0,
        "Bandwidth off-resonance must broaden to > 5.0 meV: got {}",
        dialog.flat_bandwidth_mev
    );
    assert!(
        dialog.group_velocity_ratio > 0.20,
        "Group velocity off-resonance must exceed 0.20: got {}",
        dialog.group_velocity_ratio
    );

    // 2. Return to magic angle (1.08 deg)
    dialog.params.theta_deg = 1.08;
    dialog.recompute();

    assert_eq!(dialog.twist_angle_deg, 1.08);
    assert!(
        dialog.flat_bandwidth_mev < 1.0,
        "Bandwidth must collapse back to < 1.0 meV at magic angle"
    );
    assert!(
        dialog.group_velocity_ratio <= 0.05,
        "Group velocity must re-quench to <= 0.05"
    );
}

#[test]
fn test_parameter_adjustment_in_gui_state() {
    let mut dialog = TwistedMoireDialog::new();

    // 1. Adjust interlayer shear potential contrast
    dialog.params.w_aa = 70.0;
    dialog.params.w_ab = 130.0;
    dialog.recompute();

    assert_eq!(dialog.params.w_aa, 70.0);
    assert_eq!(dialog.params.w_ab, 130.0);
    // Increased contrast sharpens relaxation further
    assert!(dialog.aa_domain_fraction < 0.16);

    // 2. Toggle atomic relaxation off
    dialog.relaxation_enabled = false;
    dialog.recompute();

    assert!(!dialog.relaxation_enabled);
    assert!(
        (dialog.aa_domain_fraction - 1.0 / 3.0).abs() < 0.02,
        "Unrelaxed AA domain fraction must be ~33.3%: got {}",
        dialog.aa_domain_fraction
    );

    // 3. Adjust soliton amplitude
    dialog.soliton_amplitude_mpa = 8.0;
    dialog.recompute();

    assert_eq!(dialog.soliton_amplitude_mpa, 8.0);
    assert_eq!(dialog.soliton.p_0_mpa, 8.0);
    assert!(dialog.soliton_confinement_ratio < 0.40);
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = TwistedMoireDialog::new();
    dialog.is_open = true;

    // Headless egui Context execution pass
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    output.textures_delta.clear();

    assert!(dialog.is_open);
    assert!(!dialog.flat_band_upper_curve.is_empty());
    assert!(!dialog.flat_band_lower_curve.is_empty());
    assert!(!dialog.dispersive_curves.is_empty());
    assert!(!dialog.dos_curve.is_empty());
    assert!(!dialog.soliton_curve.is_empty());
}
