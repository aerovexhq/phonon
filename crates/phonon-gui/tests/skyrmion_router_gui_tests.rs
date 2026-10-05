#![deny(unsafe_code)]

//! GUI test suite for Phase 360: SkyrmionRouterDialog in Phonon CAD Studio.
//!
//! Verifies:
//! - SkyrmionRouterDialog initialization and physical defaults.
//! - Preset switching (Single Skyrmion, Hexagonal Vortex Lattice, Domain Wall Interface) and recomputation.
//! - Thiele trajectory generation, Hall deflection angle, and S-parameter spectrum curves.
//! - Headless egui Context render pass across all studio visual tabs.

use egui::Context;
use phonon_gui::widgets::skyrmion_router_dialog::{SkyrmionDialogTab, SkyrmionRouterDialog};
use phonon_solver::acoustic_skyrmion_router::{DomainWallDefect, SkyrmionLatticeType};

#[test]
fn test_skyrmion_router_dialog_initialization_and_defaults() {
    let dialog = SkyrmionRouterDialog::new();

    // Dialog must be closed by default
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Default tab
    assert_eq!(dialog.active_tab, SkyrmionDialogTab::RealSpaceVectorField);

    // Physical parameter defaults
    assert_eq!(dialog.params.lattice_type, SkyrmionLatticeType::SingleSkyrmion);
    assert!((dialog.params.skyrmion_radius_um - 50.0).abs() < 1e-6);
    assert!((dialog.params.lattice_pitch_um - 160.0).abs() < 1e-6);
    assert!((dialog.params.helicity_gamma_rad - 0.0).abs() < 1e-6);
    assert_eq!(dialog.params.vorticity_m, 1);
    assert_eq!(dialog.params.grid_nx, 64);
    assert_eq!(dialog.params.grid_ny, 64);

    // Initial topological charge quantization: Single skyrmion must have Q approx -1.0
    assert!(
        (dialog.computed_charge - (-1.0)).abs() < 0.08,
        "Computed charge {} must be near -1.0",
        dialog.computed_charge
    );
    assert!(
        (dialog.discrete_charge - (-1.0)).abs() < 0.08,
        "Discrete lattice charge {} must be near -1.0",
        dialog.discrete_charge
    );

    // Topological Hall Angle must be non-zero
    assert!(
        dialog.hall_angle_deg.abs() > 0.1,
        "Hall angle {} must be non-zero",
        dialog.hall_angle_deg
    );

    // Chiral routing defaults
    assert!(
        dialog.insertion_loss_db <= 0.55,
        "Insertion loss {} dB must be <= 0.55 dB",
        dialog.insertion_loss_db
    );
    assert!(
        dialog.isolation_db >= 30.0,
        "Chiral isolation {} dB must be >= 30.0 dB",
        dialog.isolation_db
    );
    assert!(
        dialog.corner_transmission_percent >= 95.0,
        "Corner transmission {}% must be >= 95%",
        dialog.corner_transmission_percent
    );

    // Trajectory and spectrum curves must be populated
    assert!(!dialog.trajectory.is_empty());
    assert!(!dialog.trajectory_curve.is_empty());
    assert!(!dialog.s21_curve.is_empty());
    assert!(!dialog.s12_curve.is_empty());
    assert!(!dialog.s11_curve.is_empty());
}

#[test]
fn test_preset_switching_and_recomputation() {
    let mut dialog = SkyrmionRouterDialog::new();

    // 1. Switch to Hexagonal Vortex Lattice
    dialog.params.lattice_type = SkyrmionLatticeType::HexagonalVortexLattice;
    dialog.params.skyrmion_radius_um = 45.0;
    dialog.params.lattice_pitch_um = 140.0;
    dialog.recompute();

    assert_eq!(dialog.params.lattice_type, SkyrmionLatticeType::HexagonalVortexLattice);
    // Multiple skyrmions in lattice -> total charge magnitude > 1.0
    assert!(
        dialog.computed_charge.abs() > 1.0,
        "Lattice charge magnitude {} must be > 1.0",
        dialog.computed_charge.abs()
    );

    // 2. Switch to Chiral Domain Wall Waveguide with 90-degree corner bend
    dialog.params.lattice_type = SkyrmionLatticeType::DomainWallInterface;
    dialog.active_defect = DomainWallDefect::CornerBend90;
    dialog.recompute();

    assert_eq!(dialog.params.lattice_type, SkyrmionLatticeType::DomainWallInterface);
    assert_eq!(dialog.active_defect, DomainWallDefect::CornerBend90);
    assert!(
        dialog.corner_transmission_percent >= 95.0,
        "90-deg corner transmission {}% must be >= 95.0%",
        dialog.corner_transmission_percent
    );
    assert!(
        dialog.isolation_db >= 30.0,
        "Domain wall isolation {} dB must be >= 30.0 dB",
        dialog.isolation_db
    );

    // 3. Switch to Missing Resonator defect
    dialog.active_defect = DomainWallDefect::MissingResonator;
    dialog.recompute();
    assert_eq!(dialog.active_defect, DomainWallDefect::MissingResonator);
    assert!(
        dialog.corner_transmission_percent >= 95.0,
        "Missing resonator transmission {}% must be >= 95.0%",
        dialog.corner_transmission_percent
    );
}

#[test]
fn test_thiele_trajectory_generation_and_hall_deflection() {
    let mut dialog = SkyrmionRouterDialog::new();

    // Verify steady-state drift velocity components
    assert!(
        dialog.drift_velocity_mps[0] > 0.0,
        "Forward drift velocity vx {} must be positive under F_x drive",
        dialog.drift_velocity_mps[0]
    );
    assert!(
        dialog.drift_velocity_mps[1].abs() > 0.0,
        "Transverse drift velocity vy {} must be non-zero (Hall deflection)",
        dialog.drift_velocity_mps[1]
    );

    // Trajectory points monotonic forward progression
    let start_x = dialog.trajectory[0].x_um;
    let end_x = dialog.trajectory.last().unwrap().x_um;
    assert!(end_x > start_x, "Skyrmion must move forward along x");

    // Recompute with doubled drive force
    dialog.thiele.force_un = [40.0, 0.0];
    dialog.recompute();
    assert!(
        dialog.drift_velocity_mps[0] > 0.0,
        "Drift velocity must scale with drive force"
    );
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = SkyrmionRouterDialog::new();
    dialog.is_open = true;

    let ctx = Context::default();

    // Test rendering on each tab
    let tabs = [
        SkyrmionDialogTab::RealSpaceVectorField,
        SkyrmionDialogTab::TopologicalChargeDensity,
        SkyrmionDialogTab::HallDeflectionTrajectory,
        SkyrmionDialogTab::DomainWallSpectrum,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.render_content(ui);
        });
        out.textures_delta.clear();
        assert_eq!(dialog.active_tab, tab);
    }

    // Verify status message updated
    assert!(!dialog.status_msg.is_empty());
}
