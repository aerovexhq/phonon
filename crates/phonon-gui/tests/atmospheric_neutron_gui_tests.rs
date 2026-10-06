#![deny(unsafe_code)]

//! GUI test suite for the Atmospheric Secondary Neutron Cascade & DO-254 DAL-A SER Dialog.

use egui::Context;
use phonon_gui::widgets::atmospheric_neutron_dialog::{
    AtmosphericNeutronDialog, AtmosphericNeutronTab,
};
use phonon_solver::atmospheric_neutron::{
    Do254DalLevel, LightningSeverityLevel, LightningWaveformKind, SolarModulation,
};

#[test]
fn test_atmospheric_neutron_dialog_initialization() {
    let dialog = AtmosphericNeutronDialog::default();
    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, AtmosphericNeutronTab::FlightRouteCascade);
    assert_eq!(dialog.altitude_ft, 39_000.0);
    assert_eq!(dialog.latitude_deg, 45.0);
    assert_eq!(dialog.solar_modulation, SolarModulation::SolarModerate);
    assert_eq!(dialog.memory_mbits, 64);
    assert_eq!(dialog.mitigation_choice, 3); // TMR
    assert_eq!(dialog.target_dal, Do254DalLevel::DalA);

    // Verify telemetry report
    let report = dialog.sim.latest_report();
    assert!(report.is_dal_a_compliant);
    assert!(report.altitude_acceleration > 100.0);
    assert!(report.lightning_peak_v_clamp > 10.0);
    assert!(report.lightning_thermal_margin_c > 50.0);
}

#[test]
fn test_atmospheric_neutron_dialog_presets_and_recompute() {
    let mut dialog = AtmosphericNeutronDialog::default();

    // Preset 1: High-Altitude Polar Route
    dialog.altitude_ft = 43_000.0;
    dialog.latitude_deg = 75.0;
    dialog.solar_modulation = SolarModulation::SolarMinimum;
    dialog.recompute_sim();

    let report_polar = dialog.sim.latest_report();
    assert!(report_polar.altitude_acceleration > 450.0);
    assert!(!dialog.cached_spectrum.is_empty());
    assert!(!dialog.cached_alpha_cross_section.is_empty());

    // Switch to Simplex (unmitigated) -> should FAIL DO-254 DAL-A
    dialog.mitigation_choice = 0;
    dialog.recompute_sim();
    assert!(!dialog.sim.latest_report().is_dal_a_compliant, "Simplex at FL430 must fail DAL-A");

    // Switch back to TMR -> should PASS DO-254 DAL-A
    dialog.mitigation_choice = 3;
    dialog.recompute_sim();
    assert!(dialog.sim.latest_report().is_dal_a_compliant, "TMR must pass DAL-A");

    // Preset 2: Severe Level 5 Lightning Surge
    dialog.lightning_waveform = LightningWaveformKind::Waveform5A;
    dialog.lightning_severity = LightningSeverityLevel::Level5;
    dialog.clamp_device_choice = 0; // TVS
    dialog.recompute_sim();

    let report_lightning = dialog.sim.latest_report();
    assert!(report_lightning.lightning_peak_current_a > 1000.0);
    assert!(report_lightning.lightning_peak_junction_temp_c > 70.0);
    assert!(report_lightning.lightning_thermal_margin_c > 0.0);
    assert!(!dialog.cached_lightning_samples.is_empty());
}

#[test]
fn test_atmospheric_neutron_dialog_headless_render() {
    let ctx = Context::default();
    let mut dialog = AtmosphericNeutronDialog::default();
    dialog.is_open = true;

    let tabs = [
        AtmosphericNeutronTab::FlightRouteCascade,
        AtmosphericNeutronTab::NeutronSpectrumKinematics,
        AtmosphericNeutronTab::AvionicsSerCompliance,
        AtmosphericNeutronTab::LightningTransientScope,
        AtmosphericNeutronTab::ThermalDissipationMeter,
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
