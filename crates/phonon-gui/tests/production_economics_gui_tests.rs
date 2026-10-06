#![deny(unsafe_code)]

//! GUI test suite for Production Economics & Hierarchical BOM Cost Estimator Dialog.

use egui::Context;
use phonon_gui::widgets::production_economics_dialog::{EconomicsTab, ProductionEconomicsDialog};
use std::time::Instant;

#[test]
fn test_production_economics_dialog_initialization_and_cold_boot() {
    let start = Instant::now();
    let dialog = ProductionEconomicsDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "ProductionEconomicsDialog::new_fast() must initialize in sub-5ms, took {:?}",
        elapsed
    );

    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, EconomicsTab::HierarchicalBomTree);
    assert_eq!(dialog.export_volume_choice, 3);
    assert!(!dialog.cached_unit_cost_curve.is_empty());
    assert!(!dialog.cached_margin_curve.is_empty());
    assert!(!dialog.cached_breakpoints.is_empty());

    let rep = dialog.sim.report();
    assert!(rep.prototype_bom_unit_cost > 0.0);
    assert!(rep.mass_prod_bom_unit_cost_10k < rep.prototype_bom_unit_cost);
    assert!(rep.target_msrp > 0.0);
}

#[test]
fn test_production_economics_dialog_recompute_and_parameter_updates() {
    let mut dialog = ProductionEconomicsDialog::new();

    // Verify initial values
    let initial_proto_cost = dialog.sim.report().prototype_bom_unit_cost;
    assert!(initial_proto_cost > 0.0);

    // Toggle split on subcircuit in hierarchical BOM
    if let Some(sub) = dialog
        .sim
        .bom
        .items
        .iter_mut()
        .find(|item| item.is_subcircuit)
    {
        sub.is_split = !sub.is_split;
    }

    // Update part price in cost registry
    dialog.sim.registry.set_cost("R_10k", 0.015);

    // Recompute simulation
    dialog.recompute_sim();

    let updated_rep = dialog.sim.report();
    assert!(updated_rep.prototype_bom_unit_cost > 0.0);
    assert!(!dialog.cached_breakpoints.is_empty());
    assert_eq!(dialog.cached_unit_cost_curve.len(), dialog.cached_breakpoints.len());
    assert_eq!(dialog.cached_margin_curve.len(), dialog.cached_breakpoints.len());

    // Verify volume scaling curve monotonic decrease in unit cost
    let mut prev_cost = f64::INFINITY;
    for point in &dialog.cached_unit_cost_curve {
        assert!(
            point[1] <= prev_cost + 1e-6,
            "Unit cost should monotonically decrease with volume: {} vs prev {}",
            point[1],
            prev_cost
        );
        prev_cost = point[1];
    }
}

#[test]
fn test_production_economics_dialog_headless_egui_render_all_tabs() {
    let ctx = Context::default();
    let mut dialog = ProductionEconomicsDialog::new();
    dialog.is_open = true;

    let tabs = [
        EconomicsTab::HierarchicalBomTree,
        EconomicsTab::CentralCostRegistry,
        EconomicsTab::VolumeScalingCurves,
        EconomicsTab::ManufacturingCostModel,
        EconomicsTab::ProductionExport,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.render_content(ui);
        });
        out.textures_delta.clear();
    }

    // Test alias ui(&ctx) and window show
    let mut out_window = ctx.run_ui(Default::default(), |ui| {
        dialog.ui(ui.ctx());
    });
    out_window.textures_delta.clear();
}
