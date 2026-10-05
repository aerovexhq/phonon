#![deny(unsafe_code)]

//! GUI test suite for Phase 359: OptomechanicalSqueezingDialog in Phonon CAD Studio.
//!
//! Verifies:
//! - OptomechanicalSqueezingDialog initialization and physical defaults.
//! - Preset switching (Fock |1> vs Squeezed Vacuum vs Ground State Cooled) and recomputation.
//! - 2D Wigner function calculation and quadrature angle sweep updates.
//! - Headless egui Context render pass across all studio visual tabs.

use phonon_gui::widgets::optomechanical_squeezing_dialog::{
    OptomechDialogTab, OptomechPreset, OptomechanicalSqueezingDialog, WignerColormap,
};
use phonon_solver::optomechanical_squeezing::{PhononStateKind, SQL_VARIANCE};

#[test]
fn test_optomechanical_squeezing_dialog_initialization_and_defaults() {
    let dialog = OptomechanicalSqueezingDialog::new();

    // Dialog must be closed by default
    assert!(!dialog.is_open, "Dialog must be closed by default");

    // Default tab
    assert_eq!(dialog.active_tab, OptomechDialogTab::StudioOverview);
    assert_eq!(dialog.active_tab.label(), "Studio Overview");

    // Default colormap
    assert_eq!(dialog.colormap, WignerColormap::CoolWarm);

    // Default preset
    assert_eq!(dialog.selected_preset, OptomechPreset::SqueezedVacuum);
    assert_eq!(dialog.state_kind, PhononStateKind::SqueezedVacuum);

    // Physical parameter defaults
    assert!((dialog.params.cavity_freq_ghz - 10.0).abs() < 1e-6);
    assert!((dialog.params.mech_freq_mhz - 15.0).abs() < 1e-6);
    assert!((dialog.params.optomech_coupling_g0_khz - 800.0).abs() < 1e-6);
    assert!((dialog.params.cavity_decay_kappa_mhz - 2.0).abs() < 1e-6);
    assert!((dialog.params.mech_damping_gamma_hz - 150.0).abs() < 1e-6);
    assert!((dialog.params.thermal_phonon_n_th - 20.0).abs() < 1e-6);
    assert!((dialog.params.drive_power_laser_mw - 2.5).abs() < 1e-6);
    assert!((dialog.params.squeezing_parameter_r - 0.80).abs() < 1e-6);
    assert!((dialog.params.quadrature_angle_rad - 0.0).abs() < 1e-6);

    // Initial squeezing validation: Squeezed vacuum must be below SQL (0.5)
    assert!(
        dialog.variance.var_min < SQL_VARIANCE,
        "Minimum variance {} must be below SQL",
        dialog.variance.var_min
    );
    assert!(
        dialog.variance.squeezing_db >= 3.0,
        "Squeezing {} dB must be >= 3.0 dB below SQL",
        dialog.variance.squeezing_db
    );

    // Quadrature scan points
    assert!(!dialog.quadrature_scan.is_empty());

    // Dispersive spectrum resolved peaks
    assert!(dialog.resolved_spectrum.is_resolved);
    assert_eq!(dialog.resolved_spectrum.peaks.len(), 12);

    // Non-classical metrics
    assert!(dialog.metrics.quantum_purity >= 0.999);
}

#[test]
fn test_preset_switching_and_state_recomputation() {
    let mut dialog = OptomechanicalSqueezingDialog::new();

    // 1. Switch to Single Phonon Fock State |1> preset
    dialog.apply_preset(OptomechPreset::SinglePhononFock1);
    assert_eq!(dialog.selected_preset, OptomechPreset::SinglePhononFock1);
    assert_eq!(dialog.state_kind, PhononStateKind::SinglePhononFock1);
    assert_eq!(dialog.fock_dist.probability(1), 1.0);
    assert_eq!(dialog.fock_dist.probability(0), 0.0);
    assert_eq!(dialog.fock_dist.mean_phonon_number, 1.0);

    // Wigner function must show strictly negative core
    assert!(
        dialog.wigner.w_min < -0.30,
        "Fock |1> W_min {} must reach near -1/pi (-0.318)",
        dialog.wigner.w_min
    );
    assert!(dialog.wigner.is_nonclassical);

    // g^(2)(0) correlation must be 0.0 for single-phonon Fock state
    assert_eq!(dialog.metrics.second_order_correlation_g2, 0.0);
    assert!(dialog.metrics.is_sub_poissonian);
    assert!(dialog.metrics.is_quantum_pure);

    // 2. Switch to Ground State Cooled preset
    dialog.apply_preset(OptomechPreset::GroundStateCooled);
    assert_eq!(dialog.selected_preset, OptomechPreset::GroundStateCooled);
    assert_eq!(dialog.state_kind, PhononStateKind::GroundState);
    assert_eq!(dialog.fock_dist.probability(0), 1.0);
    assert_eq!(dialog.fock_dist.mean_phonon_number, 0.0);
    assert!(
        dialog.metrics.sideband_cooled_n_final < 0.10,
        "Sideband cooled final occupancy must be < 0.10 phonons"
    );

    // 3. Switch back to Squeezed Vacuum preset
    dialog.apply_preset(OptomechPreset::SqueezedVacuum);
    assert_eq!(dialog.selected_preset, OptomechPreset::SqueezedVacuum);
    assert_eq!(dialog.state_kind, PhononStateKind::SqueezedVacuum);
    assert!(dialog.variance.var_min < SQL_VARIANCE);
    assert!(dialog.variance.squeezing_db >= 3.0);
    // Parity must be strictly even
    assert_eq!(dialog.fock_dist.probability(1), 0.0);
    assert!(dialog.fock_dist.probability(2) > 0.0);
}

#[test]
fn test_wigner_generation_and_quadrature_angle_sweep() {
    let mut dialog = OptomechanicalSqueezingDialog::new();

    let initial_angle = dialog.params.quadrature_angle_rad;
    let initial_var_x = dialog.variance.var_x;

    // Sweep quadrature angle
    dialog.sweep_quadrature_angle();

    assert!(
        (dialog.params.quadrature_angle_rad - (initial_angle + 0.1)).abs() < 1e-6,
        "Quadrature angle must advance by 0.1 rad"
    );

    // Var X should change after rotation
    assert!(
        (dialog.variance.var_x - initial_var_x).abs() > 1e-4,
        "Var X must rotate with quadrature angle"
    );

    // Squeezing level in dB below SQL must remain invariant under coordinate rotation
    assert!(
        dialog.variance.squeezing_db >= 3.0,
        "Eigenvalue squeezing must be preserved under angle sweep"
    );

    // Recompute test
    dialog.recompute_all();
    assert_eq!(dialog.wigner.grid_size, 51);
    assert!(!dialog.wigner.values.is_empty());
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = OptomechanicalSqueezingDialog::new();
    dialog.is_open = true;

    let ctx = egui::Context::default();

    // 1. Render default Studio Overview tab
    let mut output = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    output.textures_delta.clear();
    assert!(dialog.is_open);

    // 2. Render all individual view tabs
    for tab in [
        OptomechDialogTab::WignerPhaseSpace,
        OptomechDialogTab::FockDistribution,
        OptomechDialogTab::QuadratureScan,
        OptomechDialogTab::DispersiveSpectrum,
    ] {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.render_content(ui);
        });
        out.textures_delta.clear();
        assert_eq!(dialog.active_tab, tab);
    }

    // 3. Render across all colormaps
    for colormap in [
        WignerColormap::CoolWarm,
        WignerColormap::Turbo,
        WignerColormap::Magma,
    ] {
        dialog.colormap = colormap;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.render_content(ui);
        });
        out.textures_delta.clear();
    }
}
