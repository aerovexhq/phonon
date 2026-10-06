#![deny(unsafe_code)]

//! GUI test suite for Closed-Loop Dynamic Electro-Thermal & Power Throttling CAD Dialog.

use egui::Context;
use phonon_gui::widgets::electrothermal_throttling_dialog::{
    ElectrothermalThrottlingDialog, ThrottlingTab,
};
use phonon_solver::electrothermal_throttling::{CoolantFluid, CoolingArchitecture, ImmersionFluidKind};
use std::time::Instant;

#[test]
fn test_electrothermal_throttling_dialog_initialization_and_cold_boot() {
    let start = Instant::now();
    let dialog = ElectrothermalThrottlingDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "ElectrothermalThrottlingDialog::new_fast() must initialize in sub-5ms, took {:?}",
        elapsed
    );

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, ThrottlingTab::DvfsTransientScope);
    assert!(!dialog.cached_transient_time_ms.is_empty());
    assert!(!dialog.cached_transient_temp_c.is_empty());
    assert!(!dialog.cached_transient_freq_ghz.is_empty());
    assert!(!dialog.cached_transient_power_w.is_empty());
    assert!(!dialog.cached_bifurcation_temps_c.is_empty());
    assert!(!dialog.cached_boiling_delta_t.is_empty());
    assert!(!dialog.cached_tim_cycles.is_empty());

    let telemetry = dialog.sim.compute_telemetry();
    assert!(telemetry.junction_temperature_c > 0.0);
    assert!(telemetry.total_power_w > 0.0);
}

#[test]
fn test_electrothermal_throttling_dialog_recompute_and_parameter_updates() {
    let mut dialog = ElectrothermalThrottlingDialog::new_fast();

    // Adjust target temperature and proportional gain
    dialog.sim.dvfs_config.target_temp_c = 75.0;
    dialog.sim.dvfs_config.kp = 0.08;
    dialog.sim.microchannel.flow_rate_lpm = 2.5;
    dialog.sim.microchannel.coolant = CoolantFluid::PropyleneGlycol50;

    dialog.recompute_all();

    assert!(!dialog.cached_transient_temp_c.is_empty());
    assert!(!dialog.cached_bifurcation_temps_c.is_empty());

    // Switch to two-phase immersion
    dialog.sim.cooling_architecture = CoolingArchitecture::DielectricTwoPhaseImmersion;
    dialog.sim.immersion.fluid = ImmersionFluidKind::Novec7100;
    dialog.recompute_all();

    let telemetry = dialog.sim.compute_telemetry();
    assert_eq!(
        telemetry.cooling_architecture,
        CoolingArchitecture::DielectricTwoPhaseImmersion
    );
    assert!(telemetry.chf_margin_pct.is_some());
}

#[test]
fn test_electrothermal_throttling_dialog_headless_egui_render_all_tabs() {
    let ctx = Context::default();
    let mut dialog = ElectrothermalThrottlingDialog::new_fast();
    dialog.is_open = true;

    let tabs = [
        ThrottlingTab::DvfsTransientScope,
        ThrottlingTab::ThermalRunawayBifurcation,
        ThrottlingTab::LiquidColdPlate,
        ThrottlingTab::TwoPhaseImmersion,
        ThrottlingTab::TimPumpOutAging,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            // Direct call to render_content via ui() when open
            dialog.ui(ui.ctx());
        });
        out.textures_delta.clear();
    }

    // Test show alias
    let mut out_window = ctx.run_ui(Default::default(), |ui| {
        dialog.show(ui.ctx());
    });
    out_window.textures_delta.clear();
}
