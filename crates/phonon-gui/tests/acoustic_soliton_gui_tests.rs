#![deny(unsafe_code)]

use phonon_gui::widgets::acoustic_soliton_dialog::{AcousticSolitonDialog, SolitonDialogTab};
use phonon_solver::acoustic_domain_wall_soliton::SolitonKind;

#[test]
fn test_acoustic_soliton_dialog_initialization() {
    let dialog = AcousticSolitonDialog::default();
    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        SolitonDialogTab::PhaseAndEnergyProfile
    );
    assert_eq!(dialog.velocity_ratio, 0.25);
    assert_eq!(dialog.omega_0_khz, 40.0);

    let q = dialog.sg_solver.state.topological_charge;
    assert!(
        (q - 1.0).abs() < 0.08,
        "Initial moving kink must have Q approximately +1.0, got {}",
        q
    );
}

#[test]
fn test_acoustic_soliton_dialog_preset_switching() {
    let mut dialog = AcousticSolitonDialog::default();

    // Switch to Antikink
    dialog.reinit_soliton(SolitonKind::MovingAntikink {
        x0: 0.0,
        velocity_ratio: -0.25,
    });
    let q_anti = dialog.sg_solver.state.topological_charge;
    assert!(
        (q_anti - (-1.0)).abs() < 0.08,
        "Antikink must have Q approximately -1.0, got {}",
        q_anti
    );

    // Switch to Breather
    dialog.reinit_soliton(SolitonKind::Breather {
        x0: 0.0,
        frequency_ratio: 0.6,
    });
    let q_breather = dialog.sg_solver.state.topological_charge;
    assert!(
        q_breather.abs() < 0.08,
        "Breather must have Q approximately 0.0, got {}",
        q_breather
    );
}

#[test]
fn test_waveguide_s_bend_update() {
    let mut dialog = AcousticSolitonDialog::default();
    dialog.bend_offset_mm = 5.0;
    dialog.has_defect = true;
    dialog.update_waveguide();

    assert!(
        dialog.wg_router.s_parameters.confinement_factor >= 0.90,
        "Waveguide confinement factor must be >= 0.90, got {}",
        dialog.wg_router.s_parameters.confinement_factor
    );

    assert!(
        dialog.wg_router.s_parameters.s21_db >= -1.2,
        "Waveguide S21 must be >= -1.2 dB, got {}",
        dialog.wg_router.s_parameters.s21_db
    );
}

#[test]
fn test_acoustic_soliton_dialog_headless_render() {
    let mut dialog = AcousticSolitonDialog::default();
    let ctx = egui::Context::default();

    // Render Tab 1: PhaseAndEnergyProfile
    dialog.active_tab = SolitonDialogTab::PhaseAndEnergyProfile;
    let mut out1 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out1.textures_delta.clear();

    // Render Tab 2: Waveguide2DField
    dialog.active_tab = SolitonDialogTab::Waveguide2DField;
    let mut out2 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out2.textures_delta.clear();

    // Render Tab 3: SParameters
    dialog.active_tab = SolitonDialogTab::SParameters;
    let mut out3 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out3.textures_delta.clear();

    // Advance simulation step
    dialog.step_simulation(10);
    assert_eq!(dialog.total_steps_executed, 10);
}
