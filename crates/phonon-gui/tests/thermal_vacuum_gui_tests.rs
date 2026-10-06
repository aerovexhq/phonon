#![deny(unsafe_code)]

//! GUI test suite for Phase 378: Aerospace Thermal-Vacuum Radiation Dissipation & Orbital Cycling Co-Simulator Dialog.

use egui::Context;
use phonon_gui::widgets::thermal_vacuum_dialog::{ThermalVacuumDialog, ThermalVacuumTab};
use std::time::Instant;

#[test]
fn test_thermal_vacuum_dialog_initialization_and_cold_boot() {
    let start = Instant::now();
    let dialog = ThermalVacuumDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "ThermalVacuumDialog::new_fast() must initialize in sub-5ms, took {:?}",
        elapsed
    );

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, ThermalVacuumTab::OrbitalEnvironment);
    assert_eq!(dialog.mission_choice, 0); // LEO
    assert_eq!(dialog.coating_choice, 0); // White paint
    assert!(dialog.radiator_area_m2 > 0.0);
    assert!(!dialog.cached_cooldown.is_empty());
    assert!(!dialog.cached_orbital_profile.is_empty());
    assert!(!dialog.cached_freezeout.is_empty());
    assert!(!dialog.cached_subthreshold.is_empty());
    assert!(!dialog.cached_id_vds.is_empty());
}

#[test]
fn test_thermal_vacuum_dialog_recompute_and_parameter_updates() {
    let mut dialog = ThermalVacuumDialog::new();

    // Switch to Deep Space cruise with Gold finish and Indium solder bump
    dialog.mission_choice = 3; // Deep space
    dialog.coating_choice = 1; // Polished gold
    dialog.solder_alloy_choice = 2; // Pure Indium
    dialog.radiator_area_m2 = 0.20;
    dialog.package_heat_capacity = 120.0;
    dialog.internal_heat_w = 25.0;
    dialog.dnp_mm = 12.0;
    dialog.underfill_coupling = 0.15;
    dialog.dopant_choice = 4; // Indium deep acceptor
    dialog.cryo_temp_k = 4.2;

    dialog.recompute_sim();

    let report = dialog.sim.report();
    assert_eq!(report.mission_label, "Deep-Space Interplanetary Cruise");
    assert_eq!(report.radiator_coating_label, "Polished Gold / Aluminized Foil");
    assert!(report.projected_lifetime_years > 0.0);
    assert!(report.subthreshold_swing_4k_mv_per_dec < 5.0);

    // Verify refreshed plot cache lengths
    assert_eq!(dialog.cached_cooldown.len(), 60);
    assert_eq!(dialog.cached_orbital_profile.len(), 75);
    assert_eq!(dialog.cached_freezeout.len(), 50);
    assert_eq!(dialog.cached_subthreshold.len(), 50);
    assert_eq!(dialog.cached_id_vds.len(), 50);
}

#[test]
fn test_thermal_vacuum_dialog_headless_egui_render_all_tabs() {
    let ctx = Context::default();
    let mut dialog = ThermalVacuumDialog::new();
    dialog.is_open = true;

    let tabs = [
        ThermalVacuumTab::OrbitalEnvironment,
        ThermalVacuumTab::StefanBoltzmannCooldown,
        ThermalVacuumTab::OrbitalCyclingProfile,
        ThermalVacuumTab::MicroBumpFatigue,
        ThermalVacuumTab::CryogenicFreezeoutKink,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.render_content(ui);
        });
        out.textures_delta.clear();
    }

    // Also test alias ui(&ctx)
    let mut out_window = ctx.run_ui(Default::default(), |ui| {
        dialog.ui(ui.ctx());
    });
    out_window.textures_delta.clear();
}
