#![deny(unsafe_code)]

use phonon_gui::widgets::quadrupole_shg_dialog::{
    QuadrupoleShgDialog, QuadrupoleShgDialogTab,
};

#[test]
fn test_quadrupole_shg_dialog_initialization() {
    let dialog = QuadrupoleShgDialog::default();
    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        QuadrupoleShgDialogTab::LatticeSpatialIntensity
    );
    assert_eq!(dialog.intracell_gamma_khz, 2.0);
    assert_eq!(dialog.intercell_lambda_khz, 10.0);
    assert_eq!(dialog.pump_power_w, 0.5);
    assert_eq!(dialog.fundamental_freq_hz, 2500.0);
    assert_eq!(dialog.disorder_w, 0.0);

    // Initial topological physics
    assert!(dialog.engine.lattice_solver.params.is_topological_soti());
    assert_eq!(dialog.engine.lattice_solver.params.bulk_quadrupole_moment(), 0.5);
    assert!(
        dialog.engine.metrics.conversion_efficiency_percent >= 15.0,
        "Default SOTI efficiency should be >= 15%, got {:.2}%",
        dialog.engine.metrics.conversion_efficiency_percent
    );
    assert!(
        dialog.engine.lattice_solver.metrics.fundamental_corner_confinement >= 0.80,
        "Fundamental corner confinement should be >= 80%, got {:.3}",
        dialog.engine.lattice_solver.metrics.fundamental_corner_confinement
    );
}

#[test]
fn test_quadrupole_shg_dialog_presets() {
    let mut dialog = QuadrupoleShgDialog::default();

    // Preset 1: Optimal Topological SOTI
    dialog.intracell_gamma_khz = 2.0;
    dialog.intercell_lambda_khz = 10.0;
    dialog.pump_power_w = 0.5;
    dialog.disorder_w = 0.0;
    dialog.recompute();

    assert!(dialog.engine.lattice_solver.params.is_topological_soti());
    assert_eq!(dialog.engine.lattice_solver.params.bulk_quadrupole_moment(), 0.5);
    assert!(dialog.engine.metrics.conversion_efficiency_percent >= 15.0);

    // Preset 2: Trivial Phase Limit
    dialog.intracell_gamma_khz = 12.0;
    dialog.intercell_lambda_khz = 10.0;
    dialog.recompute();

    assert!(!dialog.engine.lattice_solver.params.is_topological_soti());
    assert_eq!(dialog.engine.lattice_solver.params.bulk_quadrupole_moment(), 0.0);
    assert!(
        dialog.engine.metrics.conversion_efficiency_percent < 0.1,
        "Trivial phase efficiency must be quenched (< 0.1%), got {:.4}%",
        dialog.engine.metrics.conversion_efficiency_percent
    );

    // Preset 3: High-Power Saturated Emission
    dialog.intracell_gamma_khz = 2.0;
    dialog.intercell_lambda_khz = 10.0;
    dialog.pump_power_w = 1.5;
    dialog.recompute();

    assert!(dialog.engine.metrics.second_harmonic_power_w > 0.1);
    assert!(dialog.engine.metrics.conversion_efficiency_percent >= 20.0);

    // Preset 4: Disorder Immunity Test
    dialog.pump_power_w = 0.5;
    dialog.disorder_w = 0.20;
    dialog.recompute();

    assert!(
        dialog.engine.metrics.disorder_immunity_ratio >= 0.90,
        "Disorder immunity retention must be >= 90%, got {:.3}",
        dialog.engine.metrics.disorder_immunity_ratio
    );
}

#[test]
fn test_quadrupole_shg_headless_render() {
    let mut dialog = QuadrupoleShgDialog::default();
    let ctx = egui::Context::default();

    // Tab 1: LatticeSpatialIntensity
    dialog.active_tab = QuadrupoleShgDialogTab::LatticeSpatialIntensity;
    let mut out1 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out1.textures_delta.clear();

    // Tab 2: DualHarmonicSpectrum
    dialog.active_tab = QuadrupoleShgDialogTab::DualHarmonicSpectrum;
    let mut out2 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out2.textures_delta.clear();

    // Tab 3: EfficiencySaturationCurve
    dialog.active_tab = QuadrupoleShgDialogTab::EfficiencySaturationCurve;
    let mut out3 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out3.textures_delta.clear();

    // Tab 4: CornerDecayProfile
    dialog.active_tab = QuadrupoleShgDialogTab::CornerDecayProfile;
    let mut out4 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out4.textures_delta.clear();

    // Tab 5: DisorderRobustness
    dialog.active_tab = QuadrupoleShgDialogTab::DisorderRobustness;
    let mut out5 = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    out5.textures_delta.clear();
}
