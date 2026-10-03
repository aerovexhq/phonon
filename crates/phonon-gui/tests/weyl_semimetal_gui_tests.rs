#![deny(unsafe_code)]

//! Test suite for Phase 336: WeylSemimetalDialog in CAD Studio.
//!
//! Verifies:
//! - Dialog initialization and physical parameter defaults.
//! - Fermi arc and bulk dispersion updates upon parameter adjustments.
//! - Type-I to Type-II tilt transitions in GUI state.
//! - Headless egui Context execution pass and plot generation.

use phonon_gui::widgets::weyl_semimetal_dialog::{SemimetalMode, WeylSemimetalDialog};
use phonon_solver::weyl_semimetal::WeylNodeType;
use std::f64::consts::PI;

#[test]
fn test_dialog_initialization_defaults() {
    let dialog = WeylSemimetalDialog::new();

    // Dialog must be closed by default
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Physical parameter defaults
    assert_eq!(dialog.mode, SemimetalMode::TrsBroken2Node);
    assert!((dialog.tilt_parameter - 0.30).abs() < 1e-9);
    assert!((dialog.node_separation_norm - 0.60).abs() < 1e-9);
    assert!((dialog.magnetic_field_tesla - 3.0).abs() < 1e-9);
    assert!((dialog.angle_eb_deg - 0.0).abs() < 1e-9);
    assert_eq!(dialog.node_type, WeylNodeType::TypeI);

    // Initial simulation cache must be populated
    assert_eq!(dialog.total_monopole_charge, 0);
    assert!((dialog.node_separation_pi_a - 0.60).abs() < 1e-9);
    assert!(
        dialog.fermi_arc_transmission_pct >= 95.0,
        "Fermi arc transmission must exceed 95%: got {}",
        dialog.fermi_arc_transmission_pct
    );
    assert!(
        dialog.port_isolation_db >= 30.0,
        "Acoustic cross-talk isolation must exceed 30 dB: got {}",
        dialog.port_isolation_db
    );
    assert!(
        dialog.chiral_anomaly_enhancement > 1.0,
        "Chiral anomaly enhancement must exceed 1.0x"
    );

    // Plot curves must have sample points
    assert!(!dialog.fermi_arc_points.is_empty());
    assert_eq!(dialog.magnetoconductance_curve.len(), 100);
    assert_eq!(dialog.magnetoresistance_curve.len(), 100);
}

#[test]
fn test_fermi_arc_and_dispersion_update_on_parameter_changes() {
    let mut dialog = WeylSemimetalDialog::new();

    // 1. Update node separation
    dialog.node_separation_norm = 0.80;
    dialog.recompute();

    assert!((dialog.node_separation_pi_a - 0.80).abs() < 1e-9);
    let expected_half_sep = 0.80 * PI * 0.5;
    let first_pt = &dialog.fermi_arc_points[0];
    let last_pt = &dialog.fermi_arc_points[dialog.fermi_arc_points.len() - 1];

    assert!(
        (first_pt[0] - expected_half_sep).abs() < 1e-6,
        "Projected W+ node kx must match new separation: expected {}, got {}",
        expected_half_sep,
        first_pt[0]
    );
    assert!(
        (last_pt[0] - (-expected_half_sep)).abs() < 1e-6,
        "Projected W- node kx must match new separation: expected {}, got {}",
        -expected_half_sep,
        last_pt[0]
    );

    // 2. Switch to Inversion Broken 4-Node model
    dialog.mode = SemimetalMode::InversionBroken4Node;
    dialog.recompute();

    assert_eq!(dialog.model.nodes.len(), 4);
    assert_eq!(
        dialog.total_monopole_charge, 0,
        "4-node model must maintain zero net chiral charge"
    );
    assert!(!dialog.fermi_arc_points.is_empty());

    // 3. Switch to Dirac Semimetal mode
    dialog.mode = SemimetalMode::DiracSemimetal;
    dialog.recompute();

    assert!(dialog.model.is_dirac_mode);
    assert_eq!(dialog.model.nodes.len(), 4);
    assert_eq!(
        dialog.total_monopole_charge, 0,
        "Dirac semimetal must maintain zero net chiral charge"
    );
}

#[test]
fn test_type_i_to_type_ii_tilt_transition_in_gui() {
    let mut dialog = WeylSemimetalDialog::new();

    // Initial state: t = 0.30 -> Type-I
    assert_eq!(dialog.node_type, WeylNodeType::TypeI);

    // Increase tilt beyond overtilted critical boundary: t = 1.35 > 1.0 -> Type-II
    dialog.tilt_parameter = 1.35;
    dialog.recompute();

    assert_eq!(
        dialog.node_type,
        WeylNodeType::TypeII,
        "Tilt parameter 1.35 must classify as Type-II overtilted Weyl node"
    );

    // Decrease tilt back to upright cone: t = 0.15 < 1.0 -> Type-I
    dialog.tilt_parameter = 0.15;
    dialog.recompute();

    assert_eq!(
        dialog.node_type,
        WeylNodeType::TypeI,
        "Tilt parameter 0.15 must classify as Type-I point Fermi surface"
    );
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = WeylSemimetalDialog::new();
    dialog.is_open = true;

    // Headless egui Context execution pass
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    output.textures_delta.clear();

    assert!(dialog.is_open);
    assert!(!dialog.fermi_arc_points.is_empty());
    assert!(!dialog.magnetoconductance_curve.is_empty());
    assert!(!dialog.magnetoresistance_curve.is_empty());
}
