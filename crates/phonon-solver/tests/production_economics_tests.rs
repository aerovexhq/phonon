#![deny(unsafe_code)]

//! Integration tests for Production Economics, Cost Registry & Hierarchical BOM Co-Simulator.

use phonon_solver::production_economics::{
    BomLineItem, CentralCostRegistry, PriceEntry, ProductionEconomicsCoSimulator,
    ProductionVolumeModel,
};
use std::time::Instant;

#[test]
fn test_central_cost_registry_instant_synchronous_propagation() {
    let mut registry = CentralCostRegistry::new_with_standard_defaults();

    // Verify initial pre-seeded values
    assert_eq!(registry.get_cost("R_10k"), 0.012);
    assert_eq!(registry.get_cost("C_100nF"), 0.025);

    // Build line items referencing the same part key "R_10k"
    let item1 = BomLineItem::new_component("R1", "R1", 2, "R_10k", "Pullup");
    let item2 = BomLineItem::new_component("R2", "R2, R3", 2, "R_10k", "Divider");

    assert_eq!(item1.effective_unit_cost(&registry), 0.012);
    assert_eq!(item2.effective_unit_cost(&registry), 0.012);
    assert_eq!(item1.extended_cost(&registry), 0.024);

    // Update price in central registry
    let prev = registry.set_cost("R_10k", 0.035);
    assert_eq!(prev, Some(0.012));

    // Both items immediately and synchronously inherit updated cost
    assert_eq!(item1.effective_unit_cost(&registry), 0.035);
    assert_eq!(item2.effective_unit_cost(&registry), 0.035);
    assert_eq!(item1.extended_cost(&registry), 0.070);
}

#[test]
fn test_hierarchical_bom_split_and_bottom_up_rollup() {
    let registry = CentralCostRegistry::new_with_standard_defaults();

    // Subcircuit with lump sum $2.00 and 3 children
    // Child 1: R_10k (qty 2) @ 0.012 = 0.024
    // Child 2: C_100nF (qty 1) @ 0.025 = 0.025
    // Child 3: IC_LM358 (qty 1) @ 0.280 = 0.280
    // Expected bottom-up child sum = 0.024 + 0.025 + 0.280 = 0.329
    let children = vec![
        BomLineItem::new_component("C_R", "R101, R102", 2, "R_10k", "Resistors"),
        BomLineItem::new_component("C_C", "C101", 1, "C_100nF", "Capacitor"),
        BomLineItem::new_component("C_U", "U101", 1, "IC_LM358", "OpAmp"),
    ];

    let mut sub = BomLineItem::new_subcircuit(
        "SUB_FILTER",
        "MOD1",
        1,
        "SUB_ACTIVE_FILTER",
        "Active Filter Subcircuit",
        2.00,
        children,
    );

    // When un-split, effective cost is the parent lump sum
    assert!(!sub.is_split);
    assert_eq!(sub.effective_unit_cost(&registry), 2.00);
    assert_eq!(sub.extended_cost(&registry), 2.00);

    // Click "Split" button
    sub.toggle_split();
    assert!(sub.is_split);

    // When split, effective cost is bottom-up sum of children
    let split_cost = sub.effective_unit_cost(&registry);
    let expected = 2.0 * 0.012 + 0.025 + 0.280;
    assert!(
        (split_cost - expected).abs() < 1e-6,
        "Split cost should equal bottom-up sum (got {:.4}, expected {:.4})",
        split_cost,
        expected
    );

    // Toggle split off: collapses back to lump sum
    sub.toggle_split();
    assert!(!sub.is_split);
    assert_eq!(sub.effective_unit_cost(&registry), 2.00);
}

#[test]
fn test_volume_breakpoint_curves_and_margin() {
    let model = ProductionVolumeModel {
        pcb_setup_fee: 50.0,
        smt_setup_fee: 100.0,
        board_area_cm2: 20.0,
        layer_count: 2,
        solder_joints_count: 50,
        placement_cost_per_joint: 0.008,
        testing_cost_per_unit: 0.50,
        target_msrp: 39.99,
    };

    let proto_bom = 3.00;
    let breakpoints = model.calculate_breakpoints(proto_bom);

    assert_eq!(breakpoints.len(), 6);

    // Check prototype (N = 1)
    let bp_1 = &breakpoints[0];
    assert_eq!(bp_1.quantity, 1);
    assert_eq!(bp_1.component_bom_unit_cost, 3.00);
    // At N=1, setup fees dominate: 50 + 100 = 150 USD
    assert!(bp_1.total_unit_cost > 150.0);
    assert!(bp_1.gross_margin_pct < 0.0);

    // Check mass production (N = 10,000)
    let bp_10k = &breakpoints[4];
    assert_eq!(bp_10k.quantity, 10_000);
    // 3.00 * 0.34 = 1.02
    assert!((bp_10k.component_bom_unit_cost - 1.02).abs() < 1e-2);
    // Total unit cost at 10k is small and gross margin is very healthy
    assert!(bp_10k.total_unit_cost < 5.0);
    assert!(bp_10k.gross_margin_pct > 80.0);

    // Verify monotonic decay of total unit cost
    for i in 1..breakpoints.len() {
        assert!(
            breakpoints[i].total_unit_cost < breakpoints[i - 1].total_unit_cost,
            "Unit cost must strictly decrease with production volume"
        );
        assert!(
            breakpoints[i].gross_margin_pct > breakpoints[i - 1].gross_margin_pct,
            "Gross margin must strictly increase with production volume"
        );
    }

    // Breakeven check
    let be = model.breakeven_volume_for_margin(proto_bom, 50.0);
    assert!(be.is_some());
    assert!(be.unwrap() <= 1_000);
}

#[test]
fn test_cost_registry_text_serialization_roundtrip() {
    let mut reg1 = CentralCostRegistry::new();
    reg1.insert("R_1k", PriceEntry::new(0.015, "LCSC: C1", "Yageo", "1k 1%"));
    reg1.insert("C_1u", PriceEntry::new(0.040, "Digi: C2", "TDK", "1uF 50V"));

    let text = reg1.serialize_to_text();
    assert!(text.contains("R_1k"));
    assert!(text.contains("C_1u"));

    let reg2 = CentralCostRegistry::deserialize_from_text(&text);
    assert_eq!(reg2.len(), 2);
    assert!((reg2.get_cost("R_1k") - 0.015).abs() < 1e-4);
    assert!((reg2.get_cost("C_1u") - 0.040).abs() < 1e-4);
}

#[test]
fn test_production_economics_co_simulator_new_fast() {
    let start = Instant::now();
    let mut co_sim = ProductionEconomicsCoSimulator::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "new_fast() must instantiate within 5ms (took {:?}",
        elapsed
    );

    let report = co_sim.report();
    assert!(report.prototype_bom_unit_cost > 0.0);
    assert!(report.mass_prod_bom_unit_cost_10k < report.prototype_bom_unit_cost);
    assert!(report.total_component_count > 0);
    assert_eq!(report.subcircuit_packages_count, 1);

    // Recompute
    let updated = co_sim.recompute();
    assert!(updated.gross_margin_pct_10k > 50.0);
    assert!(updated.breakeven_volume_units <= 1_000);

    // Test CSV and Markdown exports
    let csv = co_sim.bom.export_csv(&co_sim.registry, 100);
    assert!(csv.contains("Item ID"));
    assert!(csv.contains("TOTAL PROTOTYPE UNIT COST"));

    let md = co_sim.bom.export_markdown_table(&co_sim.registry, 100);
    assert!(md.contains("| Designator | Qty |"));
}
