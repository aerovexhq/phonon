#![deny(unsafe_code)]

//! GUI test suite for Phase 372: ValleyHallVortexDialog in Phonon CAD Studio.

use egui::Context;
use phonon_gui::widgets::valley_hall_vortex_dialog::{
    ValleyHallDialogTab, ValleyHallVortexDialog,
};
use phonon_solver::valley_hall_vortex::ValleyHallPhase;

#[test]
fn test_valley_hall_dialog_initialization() {
    let dialog = ValleyHallVortexDialog::default();

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, ValleyHallDialogTab::RealSpaceVortex);
    assert_eq!(dialog.mass_delta_khz, 0.8);
    assert_eq!(dialog.hopping_t0_khz, 2.5);
    assert_eq!(dialog.resonance_freq_khz, 5.0);
    assert_eq!(dialog.vortex_charge_l, 1);
    assert_eq!(dialog.defect_angle_deg, 0.0);

    // Initial calculation state
    assert!(!dialog.bulk_dispersion.is_empty());
    assert!(!dialog.landau_levels.is_empty());
    assert!(!dialog.engine.ribbon_modes.is_empty());
    assert!(!dialog.engine.pumping_cycle.is_empty());

    let m = &dialog.engine.metrics;
    assert!(m.valley_directivity_db >= 28.0, "Directivity must be >= 28 dB");
    assert!(m.edge_confinement_pct >= 82.0, "Confinement must be >= 82%");
    assert_eq!(m.quantized_pumped_charge, 1.0);
}

#[test]
fn test_valley_hall_dialog_presets() {
    let mut dialog = ValleyHallVortexDialog::default();

    // 1. Preset: Pseudo-Landau Quantization
    dialog.mass_delta_khz = 0.2;
    dialog.strain_gradient_khz_per_mm = 0.25;
    dialog.recompute();
    assert_eq!(dialog.hamiltonian.phase(), ValleyHallPhase::PseudoLandauQuantized);
    assert!(!dialog.landau_levels.is_empty());

    // 2. Preset: Quantized Pump (l = 2)
    dialog.vortex_charge_l = 2;
    dialog.recompute();
    assert_eq!(dialog.engine.metrics.quantized_pumped_charge, 2.0);

    // 3. Preset: 60 deg Defect Bend
    dialog.defect_angle_deg = 60.0;
    dialog.recompute();
    assert!(
        dialog.engine.metrics.defect_transmission_pct >= 90.0,
        "Transmission through 60-degree bend must be >= 90%"
    );

    // 4. Preset: Gapless Dirac Semimetal (Delta = 0)
    dialog.mass_delta_khz = 0.0;
    dialog.strain_gradient_khz_per_mm = 0.0;
    dialog.recompute();
    assert_eq!(dialog.hamiltonian.phase(), ValleyHallPhase::DiracSemimetal);
    assert_eq!(dialog.hamiltonian.valley_chern_difference(), 0.0);
}

#[test]
fn test_valley_hall_dialog_headless_render() {
    let ctx = Context::default();
    let mut dialog = ValleyHallVortexDialog::default();
    dialog.is_open = true;

    // Run headless UI pass across all 5 tabs
    let tabs = [
        ValleyHallDialogTab::RealSpaceVortex,
        ValleyHallDialogTab::RibbonDispersion,
        ValleyHallDialogTab::PseudoLandauLevels,
        ValleyHallDialogTab::VortexPumpingCycle,
        ValleyHallDialogTab::ValleyRouterSParams,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.render_content(ui);
        });
        out.textures_delta.clear();
    }

    let mut out_window = ctx.run_ui(Default::default(), |ui| {
        dialog.ui(ui.ctx());
    });
    out_window.textures_delta.clear();

    assert!(dialog.is_open);
}
