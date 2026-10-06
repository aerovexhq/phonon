#![deny(unsafe_code)]

//! GUI test suite for the Topological Acoustic Floquet Synthetic Frequency Dimension Dialog.

use egui::Context;
use phonon_gui::widgets::floquet_frequency_dialog::{FloquetFrequencyDialog, FloquetFrequencyTab};
use phonon_solver::floquet_frequency_dimension::SolitonRegime;
use std::f64::consts::PI;

#[test]
fn test_floquet_frequency_dialog_initialization() {
    let dialog = FloquetFrequencyDialog::default();
    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, FloquetFrequencyTab::SyntheticLatticeCanvas);
    assert_eq!(dialog.base_frequency_khz, 10.0);
    assert_eq!(dialog.fsr_frequency_khz, 1.0);
    assert_eq!(dialog.num_frequency_modes, 11);
    assert_eq!(dialog.num_spatial_sites, 6);
    assert_eq!(dialog.current_regime, SolitonRegime::ChiralEdgeCurrent);
    assert_eq!(dialog.engine.metrics.synthetic_chern_number, 1);
    assert!(dialog.engine.metrics.forward_conversion_efficiency_percent > 90.0);
    assert!(dialog.engine.metrics.reverse_isolation_db < -25.0);
}

#[test]
fn test_floquet_frequency_dialog_presets() {
    let mut dialog = FloquetFrequencyDialog::default();

    // Preset: Localized Frequency Soliton
    dialog.current_regime = SolitonRegime::LocalizedFrequencySoliton;
    dialog.synthetic_gauge_flux_rad = std::f64::consts::FRAC_PI_2;
    dialog.kerr_nonlinearity = 0.08;
    dialog.recompute();
    assert_eq!(dialog.current_regime, SolitonRegime::LocalizedFrequencySoliton);
    assert!(dialog.engine.metrics.soliton_stability_fidelity_percent >= 90.0);

    // Preset: Linear Group Dispersion
    dialog.current_regime = SolitonRegime::LinearDispersion;
    dialog.synthetic_gauge_flux_rad = 0.0;
    dialog.kerr_nonlinearity = 0.0;
    dialog.recompute();
    assert_eq!(dialog.current_regime, SolitonRegime::LinearDispersion);
    assert_eq!(dialog.engine.metrics.synthetic_chern_number, 0);

    // Preset: High-Harmonic Frequency Pumper
    dialog.current_regime = SolitonRegime::ChiralEdgeCurrent;
    dialog.modulation_depth = 0.50;
    dialog.synthetic_gauge_flux_rad = 2.0 * PI / 3.0;
    dialog.recompute();
    assert_eq!(dialog.engine.metrics.synthetic_chern_number, 1);
    assert!(dialog.engine.metrics.forward_conversion_efficiency_percent > 90.0);
}

#[test]
fn test_floquet_frequency_dialog_headless_render() {
    let ctx = Context::default();
    let mut dialog = FloquetFrequencyDialog::default();
    dialog.is_open = true;

    let tabs = [
        FloquetFrequencyTab::SyntheticLatticeCanvas,
        FloquetFrequencyTab::QuasiEnergyDispersion,
        FloquetFrequencyTab::ConversionSpectrum,
        FloquetFrequencyTab::SolitonDynamics,
        FloquetFrequencyTab::SyntheticFluxSweep,
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
