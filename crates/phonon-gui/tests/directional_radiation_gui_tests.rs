#![deny(unsafe_code)]

//! GUI test suite for the Directional Cosmic Heavy Ion Radiation Track & 3D Anisotropic Shielding Dialog.

use egui::Context;
use phonon_gui::widgets::directional_radiation_dialog::{
    DirectionalRadiationDialog, DirectionalRadiationTab,
};
use phonon_solver::directional_radiation::{HeavyIonSpecies, ShieldingMaterial};

#[test]
fn test_directional_radiation_dialog_initialization() {
    let dialog = DirectionalRadiationDialog::default();
    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, DirectionalRadiationTab::Track3dCanvas);
    assert_eq!(dialog.ion_species, HeavyIonSpecies::IronFe56);
    assert_eq!(dialog.theta_deg, 30.0);
    assert_eq!(dialog.phi_deg, 45.0);
    assert_eq!(dialog.energy_mev_per_nuc, 150.0);
    assert_eq!(dialog.hull_thickness_mm, 3.5);
    assert_eq!(dialog.mission_duration_years, 5.0);
    assert_eq!(dialog.primary_material, ShieldingMaterial::Aluminum);

    // Verify fast-path pre-seeded metrics
    assert_eq!(dialog.sim.latest_telemetry.peak_let_mev_cm2_mg, 78.4);
    assert_eq!(dialog.sim.latest_telemetry.total_mbu_flipped_cells, 6);
    assert_eq!(dialog.sim.latest_telemetry.pierced_die_count, 2);
    assert!(!dialog.sim.latest_telemetry.sel_triggered_any);
}

#[test]
fn test_directional_radiation_dialog_presets_and_recompute() {
    let mut dialog = DirectionalRadiationDialog::default();

    // Preset: Solar Proton Event (SPE)
    dialog.ion_species = HeavyIonSpecies::Proton;
    dialog.energy_mev_per_nuc = 80.0;
    dialog.theta_deg = 15.0;
    dialog.phi_deg = 0.0;
    dialog.recompute();

    assert!(dialog.sim.latest_telemetry.peak_let_mev_cm2_mg < 10.0);
    assert!(!dialog.cached_let_curve.is_empty());
    assert!(!dialog.cached_polar_sweep.is_empty());

    // Preset: Galactic Cosmic Ray Iron Strike (Fe-56)
    dialog.ion_species = HeavyIonSpecies::IronFe56;
    dialog.energy_mev_per_nuc = 250.0;
    dialog.theta_deg = 35.0;
    dialog.phi_deg = 45.0;
    dialog.recompute();

    assert!(dialog.sim.latest_telemetry.peak_let_mev_cm2_mg > 60.0);
    assert!(dialog.sim.latest_telemetry.max_deposited_charge_fc > 0.0);

    // Preset: High-Z Spot Shielded Die
    dialog.primary_material = ShieldingMaterial::Tungsten;
    dialog.hull_thickness_mm = 8.0;
    dialog.recompute();

    assert!(dialog.sim.latest_telemetry.shielding_attenuation_percent > 80.0);
    assert!(!dialog.cached_al_tradeoff.is_empty());
    assert!(!dialog.cached_gz_tradeoff.is_empty());
}

#[test]
fn test_directional_radiation_dialog_headless_render() {
    let ctx = Context::default();
    let mut dialog = DirectionalRadiationDialog::default();
    dialog.is_open = true;

    let tabs = [
        DirectionalRadiationTab::Track3dCanvas,
        DirectionalRadiationTab::BraggPeakCurve,
        DirectionalRadiationTab::AnisotropicPolarMap,
        DirectionalRadiationTab::MultiDieMbuMatrix,
        DirectionalRadiationTab::TidTradeoffPlot,
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
}
