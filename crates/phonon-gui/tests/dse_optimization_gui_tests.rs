#![deny(unsafe_code)]

//! Automated GUI test suite for Multi-Objective PPA-C Design Space Exploration (DSE) Dialog.

use egui::Context;
use phonon_gui::widgets::dse_optimization_dialog::{DseOptimizationDialog, DseTab};
use std::time::Instant;

#[test]
fn test_dse_dialog_cold_boot_latency() {
    let start = Instant::now();
    let dlg = DseOptimizationDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "DseOptimizationDialog::new_fast() must boot in sub-5ms, took {:?}",
        elapsed
    );

    assert!(!dlg.is_open);
    assert_eq!(dlg.active_tab, DseTab::ParetoFrontier);
    assert!(!dlg.cached_frontier.is_empty());
    assert!(!dlg.cached_gp_slice.is_empty());
    assert_eq!(dlg.cached_packaging.len(), 4);
    assert!(dlg.cached_report.best_energy_efficiency_gflops_per_w > 0.0);
}

#[test]
fn test_dse_dialog_solve_and_telemetry() {
    let mut dlg = DseOptimizationDialog::new_fast();

    // Execute full optimization run
    dlg.run_solve();
    assert!(!dlg.cached_frontier.is_empty());
    assert!(dlg.cached_report.pareto_frontier_count > 0);
    assert!(dlg.cached_report.min_power_w > 0.0);
    assert!(dlg.cached_report.max_freq_ghz > 1.5);
    assert!(dlg.cached_report.min_cost_usd > 0.0);
    assert!(!dlg.sim.population.is_empty());
}

#[test]
fn test_dse_dialog_tab_navigation_and_candidate_selection() {
    let mut dlg = DseOptimizationDialog::new_fast();

    dlg.active_tab = DseTab::BayesianSurrogate;
    assert_eq!(dlg.active_tab, DseTab::BayesianSurrogate);

    dlg.active_tab = DseTab::PackagingTrades;
    assert_eq!(dlg.active_tab, DseTab::PackagingTrades);

    dlg.active_tab = DseTab::DesignSpaceGenome;
    assert_eq!(dlg.active_tab, DseTab::DesignSpaceGenome);

    dlg.active_tab = DseTab::OptimalScorecard;
    assert_eq!(dlg.active_tab, DseTab::OptimalScorecard);

    dlg.active_tab = DseTab::ParetoFrontier;
    assert_eq!(dlg.active_tab, DseTab::ParetoFrontier);

    dlg.selected_candidate_idx = Some(0);
    assert_eq!(dlg.selected_candidate_idx, Some(0));
}

#[test]
fn test_dse_dialog_headless_egui_render_all_tabs() {
    let ctx = Context::default();
    let mut dlg = DseOptimizationDialog::new_with_baseline();
    dlg.is_open = true;

    let tabs = [
        DseTab::ParetoFrontier,
        DseTab::BayesianSurrogate,
        DseTab::PackagingTrades,
        DseTab::DesignSpaceGenome,
        DseTab::OptimalScorecard,
    ];

    for tab in tabs {
        dlg.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dlg.ui(ui.ctx());
        });
        out.textures_delta.clear();
    }

    // Test show alias
    let mut out_window = ctx.run_ui(Default::default(), |ui| {
        dlg.show(ui.ctx());
    });
    out_window.textures_delta.clear();
}
