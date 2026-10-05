#![deny(unsafe_code)]

//! GUI test suite for Phase 357: FloquetTimeCrystalDialog in Phonon CAD Studio.
//!
//! Verifies:
//! - FloquetTimeCrystalDialog initialization, default parameters, and cache population.
//! - Preset switching (Stable DTC, Zero Perturbation, Thermal Ergodic, Trivial Paramagnet) and recomputation.
//! - Rigidity plateau sweep update in GUI state.
//! - Stroboscopic cycle step progression.
//! - Headless egui Context execution pass and plot generation.

use phonon_gui::widgets::floquet_time_crystal_dialog::{
    FloquetPlotTab, FloquetTimeCrystalDialog,
};

#[test]
fn test_floquet_time_crystal_dialog_initialization_and_defaults() {
    let dialog = FloquetTimeCrystalDialog::new();

    // Dialog must be closed initially
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Physical parameter defaults
    assert_eq!(dialog.chain_length, 8);
    assert!((dialog.drive_period_us - 1.0).abs() < 1e-6);
    assert!((dialog.pulse_error_epsilon - 0.05).abs() < 1e-6);
    assert_eq!(dialog.coupling_ratio, 1.0);
    assert!((dialog.disorder_ratio - 2.0).abs() < 1e-3);

    // Initial phase metrics
    assert!(
        dialog.spectral_analysis.subharmonic_fraction >= 0.65,
        "Default initial DTC state must exhibit dominant subharmonic power"
    );
    assert!(
        (dialog.pi_pairing_gap_rad - std::f64::consts::PI).abs() < 0.35,
        "Quasi-energy pi-gap must be close to pi"
    );

    // Cached curves must be populated
    assert_eq!(dialog.dynamics_points.len(), 60);
    assert_eq!(dialog.edwards_anderson_points.len(), 60);
    assert!(!dialog.fourier_points.is_empty());
    assert!(!dialog.plateau_points.is_empty());
}

#[test]
fn test_preset_switching_and_recomputation() {
    let mut dialog = FloquetTimeCrystalDialog::new();

    // 1. Switch to Zero Perturbation DTC
    dialog.set_preset_zero_perturbation();
    assert_eq!(dialog.pulse_error_epsilon, 0.0);
    assert!(dialog.spectral_analysis.subharmonic_fraction >= 0.80);
    assert!(dialog.status_msg.contains("Topological DTC phase"));

    // 2. Switch to Thermal Ergodic (W = 0, no MBL)
    dialog.set_preset_thermal_ergodic();
    assert_eq!(dialog.disorder_ratio, 0.0);
    assert!(dialog.status_msg.contains("Thermal Ergodic"));

    // 3. Switch to Trivial Paramagnet (J = 0)
    dialog.set_preset_trivial_paramagnet();
    assert_eq!(dialog.coupling_ratio, 0.0);
    assert!(dialog.status_msg.contains("Trivial Paramagnet"));

    // 4. Switch back to Stable DTC
    dialog.set_preset_stable_dtc();
    assert_eq!(dialog.pulse_error_epsilon, 0.05);
    assert_eq!(dialog.disorder_ratio, 2.0);
    assert_eq!(dialog.coupling_ratio, 1.0);
    assert!(dialog.spectral_analysis.is_rigidly_locked);
    assert!(dialog.status_msg.contains("Topological DTC phase"));
}

#[test]
fn test_rigidity_plateau_sweep_update() {
    let mut dialog = FloquetTimeCrystalDialog::new();
    let old_len = dialog.plateau_points.len();

    dialog.sweep_rigidity_plateau();

    assert!(!dialog.plateau_points.is_empty());
    assert!(
        dialog.plateau_points.len() >= old_len,
        "Sweeping plateau should update sample points"
    );
    assert!(
        dialog.phase_diagram.plateau_width >= 0.15,
        "Stable DTC preset must have broad plateau width >= 0.15"
    );
    assert!(dialog.status_msg.contains("Rigidity scan complete"));
}

#[test]
fn test_cycle_stroboscopic_step() {
    let mut dialog = FloquetTimeCrystalDialog::new();
    assert_eq!(dialog.stroboscopic_step, 0);

    dialog.cycle_stroboscopic_step();
    assert_eq!(dialog.stroboscopic_step, 1);

    dialog.cycle_stroboscopic_step();
    assert_eq!(dialog.stroboscopic_step, 2);

    // Tab labels
    dialog.active_plot_tab = FloquetPlotTab::FourierSpectrum;
    assert_eq!(dialog.active_plot_tab.label(), "Subharmonic S(omega)");

    dialog.active_plot_tab = FloquetPlotTab::RigidityPlateau;
    assert_eq!(dialog.active_plot_tab.label(), "Rigidity Plateau");

    dialog.active_plot_tab = FloquetPlotTab::AllPlotsCombined;
    assert_eq!(dialog.active_plot_tab.label(), "Combined View");
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = FloquetTimeCrystalDialog::new();
    dialog.is_open = true;

    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    output.textures_delta.clear();

    assert!(dialog.is_open, "Dialog should remain open after render pass");
    assert_eq!(dialog.dynamics_points.len(), 60);
    assert!(!dialog.fourier_points.is_empty());
}
