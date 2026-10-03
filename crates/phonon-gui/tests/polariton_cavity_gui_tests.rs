#![deny(unsafe_code)]

//! Verification test suite for Phase 331: Polariton Cavity & Waveguide Dialog.
//!
//! Tests dialog initialization, default physical parameters, dispersion curves,
//! transmission spectrum with vacuum Rabi splitting, defect toggle disorder immunity,
//! and headless egui rendering.

use phonon_gui::widgets::polariton_cavity_dialog::PolaritonCavityDialog;

#[test]
fn test_polariton_cavity_dialog_initialization_defaults() {
    let dialog = PolaritonCavityDialog::new();

    // Dialog should be closed by default
    assert!(!dialog.is_open, "Dialog should be closed by default");

    // Physical parameter defaults
    assert_eq!(dialog.params.bare_exciton_energy_mev, 1500.0);
    assert_eq!(dialog.params.cavity_cutoff_energy_mev, 1500.0);
    assert_eq!(dialog.params.rabi_coupling_g_mev, 15.0);
    assert_eq!(dialog.params.waveguide_width_um, 4.0);
    assert_eq!(dialog.params.refractive_index, 3.5);
    assert_eq!(dialog.params.chern_number, 1);
    assert!(!dialog.defect_enabled);

    // Initial simulation state must be fully pre-computed
    assert!(!dialog.dispersion_points_lp.is_empty());
    assert!(!dialog.dispersion_points_up.is_empty());
    assert!(!dialog.dispersion_points_edge.is_empty());
    assert!(!dialog.transmission_spectrum.is_empty());
    assert!(dialog.wavefunction_2d.is_some());

    // Vacuum Rabi splitting Delta = 2g = 30.0 meV
    assert!((dialog.rabi_splitting_mev - 30.0).abs() < 1e-9);

    // Clean non-reciprocal isolation must exceed 30 dB in topological regime
    assert!(
        dialog.non_reciprocal_isolation_db > 30.0,
        "Clean isolation {} dB must exceed 30 dB",
        dialog.non_reciprocal_isolation_db
    );

    // Initial defect transmission ratio on clean perimeter is 1.0 (100%)
    assert!((dialog.defect_transmission_ratio - 1.0).abs() < 1e-9);

    // Forward insertion loss must be < 0.5 dB
    assert!(
        dialog.insertion_loss_db < 0.5,
        "Insertion loss {} dB must be < 0.5 dB",
        dialog.insertion_loss_db
    );
}

#[test]
fn test_dispersion_transmission_and_mode_profile_calculation() {
    let mut dialog = PolaritonCavityDialog::new();
    dialog.params.rabi_coupling_g_mev = 20.0; // Rabi splitting 2g = 40 meV
    dialog.run_simulation();

    // Verify 40 meV splitting
    assert!((dialog.rabi_splitting_mev - 40.0).abs() < 1e-9);

    // Verify dispersion curves
    assert_eq!(dialog.dispersion_points_lp.len(), 81);
    assert_eq!(dialog.dispersion_points_up.len(), 81);
    assert_eq!(dialog.dispersion_points_edge.len(), 81);

    for i in 0..81 {
        let lp_e = dialog.dispersion_points_lp[i][1];
        let up_e = dialog.dispersion_points_up[i][1];
        assert!(up_e > lp_e, "UPB must lie strictly above LPB");
    }

    // Verify transmission spectrum has 101 points
    assert_eq!(dialog.transmission_spectrum.len(), 101);

    // Verify 2D wavefunction exists and has >= 0.70 perimeter concentration
    let wf = dialog.wavefunction_2d.as_ref().expect("Wavefunction must exist");
    assert_eq!(wf.nx, 20);
    assert_eq!(wf.ny, 20);
    assert_eq!(wf.intensity.len(), 400);
    assert!(
        wf.perimeter_confinement_ratio >= 0.70,
        "Perimeter concentration {} must be >= 0.70",
        wf.perimeter_confinement_ratio
    );
}

#[test]
fn test_defect_toggle_effect_on_telemetry() {
    let mut dialog = PolaritonCavityDialog::new();

    // 1. Clean state
    assert!(!dialog.defect_enabled);
    assert_eq!(dialog.defect_transmission_ratio, 1.0);

    // 2. Enable defect obstacle
    dialog.defect_enabled = true;
    dialog.run_simulation();

    assert!(dialog.defect_enabled);
    assert!(
        dialog.defect_transmission_ratio >= 0.90,
        "Topological protection must ensure T_defect / T_clean >= 0.90, got {}",
        dialog.defect_transmission_ratio
    );
    assert!(
        dialog.defect_transmission_ratio < 1.0,
        "Defect should produce minor radiation loss"
    );

    // Wavefunction must record defect presence
    let wf_def = dialog.wavefunction_2d.as_ref().expect("Wavefunction must exist");
    assert!(wf_def.has_defect);

    // 3. Disable defect obstacle again
    dialog.defect_enabled = false;
    dialog.run_simulation();

    assert!(!dialog.defect_enabled);
    assert_eq!(dialog.defect_transmission_ratio, 1.0);
    let wf_clean = dialog.wavefunction_2d.as_ref().expect("Wavefunction must exist");
    assert!(!wf_clean.has_defect);
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = PolaritonCavityDialog::new();
    dialog.is_open = true;

    // Headless egui Context execution pass
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    output.textures_delta.clear();

    assert!(dialog.is_open);
    assert!(!dialog.dispersion_points_lp.is_empty());
    assert!(!dialog.transmission_spectrum.is_empty());
    assert!(dialog.wavefunction_2d.is_some());
}
