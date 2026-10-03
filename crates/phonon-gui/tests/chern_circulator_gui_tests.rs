#![deny(unsafe_code)]

//! Test suite for Phase 343: ChernCirculatorDialog in Phonon CAD Studio.
//!
//! Verifies:
//! - ChernCirculatorDialog initialization and default parameters.
//! - Input port switching (Port 1, 2, 3) and cyclic scattering matrix recalculation.
//! - Corner defect toggle in GUI state and backscattering immunity.
//! - Headless egui Context execution pass and plot generation.

use phonon_gui::widgets::chern_circulator_dialog::ChernCirculatorDialog;
use phonon_solver::acoustic_chern_circulator::{CirculatorPort, ObstacleKind};

#[test]
fn test_dialog_initialization_and_defaults() {
    let dialog = ChernCirculatorDialog::new();

    // Dialog must be closed by default
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Default physical parameters
    assert_eq!(dialog.params.a_0_mm, 20.0);
    assert_eq!(dialog.params.c_0_mps, 343.0);
    assert_eq!(dialog.params.r_cyl_mm, 4.0);
    assert_eq!(dialog.params.omega_rad_s, 1200.0);
    assert_eq!(dialog.params.kappa, 0.15);
    assert_eq!(dialog.params.f_0_khz, 4.0);

    // Initial controls state
    assert_eq!(dialog.active_port, CirculatorPort::Port1);
    assert_eq!(dialog.obstacle, ObstacleKind::None);
    assert!(!dialog.corner_defect_enabled);

    // Initial simulation metrics
    assert_eq!(dialog.chern_number, 1);
    assert!(
        dialog.insertion_loss_db <= 0.50,
        "Initial insertion loss must be <= 0.50 dB: got {} dB",
        dialog.insertion_loss_db
    );
    assert!(
        dialog.isolation_db >= 35.0,
        "Initial isolation must be >= 35.0 dB: got {} dB",
        dialog.isolation_db
    );
    assert!(
        dialog.return_loss_db <= -25.0,
        "Initial return loss must be <= -25.0 dB: got {} dB",
        dialog.return_loss_db
    );
    assert!(
        (dialog.topological_gap_khz - 1.50).abs() < 0.05,
        "Initial topological gap should be ~1.50 kHz: got {} kHz",
        dialog.topological_gap_khz
    );
    assert!(
        dialog.corner_transmission_percent >= 95.0,
        "Corner transmission must be >= 95.0%: got {}%",
        dialog.corner_transmission_percent
    );

    // Plot curves must be populated
    assert!(!dialog.s21_curve.is_empty());
    assert!(!dialog.s31_curve.is_empty());
    assert!(!dialog.s11_curve.is_empty());
}

#[test]
fn test_input_port_switching_and_cyclic_recalculation() {
    let mut dialog = ChernCirculatorDialog::new();

    // 1. Port 1 -> Output Port 2, Isolated Port 3
    assert_eq!(dialog.active_port, CirculatorPort::Port1);
    let out_p1 = dialog.circulator.output_port_for(CirculatorPort::Port1);
    let iso_p1 = dialog.circulator.isolated_port_for(CirculatorPort::Port1);
    assert_eq!(out_p1, CirculatorPort::Port2);
    assert_eq!(iso_p1, CirculatorPort::Port3);
    assert!(dialog.insertion_loss_db <= 0.50);
    assert!(dialog.isolation_db >= 35.0);

    // 2. Switch to Port 2 -> Output Port 3, Isolated Port 1
    dialog.active_port = CirculatorPort::Port2;
    dialog.recompute();

    assert_eq!(dialog.active_port, CirculatorPort::Port2);
    let out_p2 = dialog.circulator.output_port_for(CirculatorPort::Port2);
    let iso_p2 = dialog.circulator.isolated_port_for(CirculatorPort::Port2);
    assert_eq!(out_p2, CirculatorPort::Port3);
    assert_eq!(iso_p2, CirculatorPort::Port1);
    assert!(
        dialog.insertion_loss_db <= 0.50,
        "Port 2 -> 3 insertion loss must be <= 0.50 dB: got {} dB",
        dialog.insertion_loss_db
    );
    assert!(
        dialog.isolation_db >= 35.0,
        "Port 2 -> 1 isolation must be >= 35.0 dB: got {} dB",
        dialog.isolation_db
    );

    // 3. Switch to Port 3 -> Output Port 1, Isolated Port 2
    dialog.active_port = CirculatorPort::Port3;
    dialog.recompute();

    assert_eq!(dialog.active_port, CirculatorPort::Port3);
    let out_p3 = dialog.circulator.output_port_for(CirculatorPort::Port3);
    let iso_p3 = dialog.circulator.isolated_port_for(CirculatorPort::Port3);
    assert_eq!(out_p3, CirculatorPort::Port1);
    assert_eq!(iso_p3, CirculatorPort::Port2);
    assert!(
        dialog.insertion_loss_db <= 0.50,
        "Port 3 -> 1 insertion loss must be <= 0.50 dB: got {} dB",
        dialog.insertion_loss_db
    );
    assert!(
        dialog.isolation_db >= 35.0,
        "Port 3 -> 2 isolation must be >= 35.0 dB: got {} dB",
        dialog.isolation_db
    );
}

#[test]
fn test_corner_defect_toggle_in_gui_state() {
    let mut dialog = ChernCirculatorDialog::new();

    // Baseline: Clean boundary
    assert!(!dialog.corner_defect_enabled);
    assert_eq!(dialog.obstacle, ObstacleKind::None);
    assert_eq!(dialog.corner_transmission_percent, 100.0);

    // 1. Toggle 90-degree corner defect
    dialog.corner_defect_enabled = true;
    dialog.recompute();

    assert_eq!(dialog.obstacle, ObstacleKind::SharpCorner90);
    assert!(
        dialog.corner_transmission_percent >= 95.0,
        "Transmission around sharp 90-deg corner must be >= 95.0%: got {}%",
        dialog.corner_transmission_percent
    );
    assert!(dialog.defect_result.is_immune);
    assert!(dialog.defect_result.backscattering_reflection_db <= -30.0);

    // 2. Select missing cavity vacancy
    dialog.obstacle = ObstacleKind::MissingSiteVacancy;
    dialog.recompute();

    assert_eq!(dialog.obstacle, ObstacleKind::MissingSiteVacancy);
    assert!(
        dialog.corner_transmission_percent >= 95.0,
        "Transmission around vacancy must be >= 95.0%: got {}%",
        dialog.corner_transmission_percent
    );
    assert!(dialog.defect_result.is_immune);

    // 3. Select 120-degree bend
    dialog.obstacle = ObstacleKind::SharpBend120;
    dialog.recompute();

    assert_eq!(dialog.obstacle, ObstacleKind::SharpBend120);
    assert!(dialog.corner_transmission_percent >= 95.0);
    assert!(dialog.defect_result.is_immune);

    // 4. Toggle back to clean boundary
    dialog.corner_defect_enabled = false;
    dialog.obstacle = ObstacleKind::None;
    dialog.recompute();

    assert_eq!(dialog.obstacle, ObstacleKind::None);
    assert_eq!(dialog.corner_transmission_percent, 100.0);
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = ChernCirculatorDialog::new();
    dialog.is_open = true;

    // Headless egui Context execution pass
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    output.textures_delta.clear();

    assert!(dialog.is_open);
    assert!(!dialog.s21_curve.is_empty());
    assert!(!dialog.s31_curve.is_empty());
    assert!(!dialog.s11_curve.is_empty());
    assert_eq!(dialog.chern_number, 1);
    assert!(dialog.isolation_db >= 35.0);
}
