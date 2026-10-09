#![deny(unsafe_code)]

//! Comprehensive test suite for Phonon Studio Integrated Production Economics,
//! Live Schematic Canvas BOM Extraction, Multi-Currency Formatting, and Persistent Pricing Serialization.

use egui::Pos2;
use phonon_gui::schematic::{
    component_to_part_key_and_desc, deserialize_project, generate_bom_from_canvas,
    serialize_project, serialize_project_with_pricing, ComponentKind, SchematicCanvas,
    SchematicComponent, SchematicWire, SubcircuitDefinition, SubcircuitInstance, FLAG_HAS_PRICING,
};
use phonon_gui::widgets::production_economics_dialog::ProductionEconomicsDialog;
use phonon_solver::production_economics::{
    BomLineItem, CentralCostRegistry, Currency, HierarchicalBom,
};
use std::collections::HashMap;

fn create_test_comp(id: usize, kind: ComponentKind, name: &str, val: &str, pos: Pos2) -> SchematicComponent {
    let mut comp = SchematicComponent::new(id, kind, pos, id);
    comp.name = name.to_string();
    comp.value_str = val.to_string();
    comp
}

#[test]
fn test_component_to_part_key_and_desc_mapping() {
    let r1 = create_test_comp(1, ComponentKind::Resistor, "R1", "10k", Pos2::ZERO);
    let (key_r, desc_r) = component_to_part_key_and_desc(&r1);
    assert_eq!(key_r, "R_10k");
    assert!(desc_r.contains("Resistor 10k Ohm"));

    let c1 = create_test_comp(2, ComponentKind::Capacitor, "C1", "100nF", Pos2::ZERO);
    let (key_c, desc_c) = component_to_part_key_and_desc(&c1);
    assert_eq!(key_c, "C_100nF");
    assert!(desc_c.contains("Capacitor 100nF"));

    let l1 = create_test_comp(3, ComponentKind::Inductor, "L1", "10uH", Pos2::ZERO);
    let (key_l, desc_l) = component_to_part_key_and_desc(&l1);
    assert_eq!(key_l, "L_10uH");
    assert!(desc_l.contains("Inductor 10uH"));

    let mut d1 = create_test_comp(4, ComponentKind::Diode, "D1", "1N4148", Pos2::ZERO);
    d1.model_name = Some("1N4148".to_string());
    let (key_d, desc_d) = component_to_part_key_and_desc(&d1);
    assert_eq!(key_d, "D_1N4148");
    assert!(desc_d.contains("Switching Diode"));

    let q1 = create_test_comp(5, ComponentKind::BjtNpn, "Q1", "", Pos2::ZERO);
    let (key_q, desc_q) = component_to_part_key_and_desc(&q1);
    assert_eq!(key_q, "Q_BC547B");
    assert!(desc_q.contains("NPN"));

    let m1 = create_test_comp(6, ComponentKind::Nmos, "M1", "", Pos2::ZERO);
    let (key_m, desc_m) = component_to_part_key_and_desc(&m1);
    assert_eq!(key_m, "Q_2N7002");
    assert!(desc_m.contains("N-Channel MOSFET"));

    let op1 = create_test_comp(7, ComponentKind::OpAmp, "U1", "", Pos2::ZERO);
    let (key_op, desc_op) = component_to_part_key_and_desc(&op1);
    assert_eq!(key_op, "IC_LM358");
    assert!(desc_op.contains("Operational Amplifier"));

    let reg1 = create_test_comp(8, ComponentKind::VoltageRegulator, "VR1", "", Pos2::ZERO);
    let (key_reg, desc_reg) = component_to_part_key_and_desc(&reg1);
    assert_eq!(key_reg, "IC_LM7805");
    assert!(desc_reg.contains("Voltage Regulator"));
}

#[test]
fn test_live_canvas_bom_generation_grouping_and_ground_exclusion() {
    let mut canvas = SchematicCanvas::new();

    // Place 3 identical 10k resistors
    canvas.components.push(create_test_comp(1, ComponentKind::Resistor, "R1", "10k", Pos2::new(100.0, 100.0)));
    canvas.components.push(create_test_comp(2, ComponentKind::Resistor, "R2", "10k", Pos2::new(100.0, 150.0)));
    canvas.components.push(create_test_comp(3, ComponentKind::Resistor, "R3", "10k", Pos2::new(100.0, 200.0)));

    // Place 1 different resistor (4.7k)
    canvas.components.push(create_test_comp(4, ComponentKind::Resistor, "R4", "4.7k", Pos2::new(200.0, 100.0)));

    // Place 2 identical 100nF capacitors
    canvas.components.push(create_test_comp(5, ComponentKind::Capacitor, "C1", "100nF", Pos2::new(300.0, 100.0)));
    canvas.components.push(create_test_comp(6, ComponentKind::Capacitor, "C2", "100nF", Pos2::new(300.0, 150.0)));

    // Place ground symbols (should be excluded from purchasing BOM)
    canvas.components.push(create_test_comp(7, ComponentKind::Ground, "GND1", "", Pos2::new(100.0, 250.0)));
    canvas.components.push(create_test_comp(8, ComponentKind::Ground, "GND2", "", Pos2::new(200.0, 250.0)));

    let subcircuits = HashMap::new();
    let bom = generate_bom_from_canvas(&canvas, &subcircuits);

    // 3 unique lines: R_10k (qty 3), R_4.7k (qty 1), C_100nF (qty 2)
    assert_eq!(bom.items.len(), 3);
    assert_eq!(bom.total_component_count(), 6);

    let r_10k = bom.items.iter().find(|i| i.part_key == "R_10k").unwrap();
    assert_eq!(r_10k.quantity, 3);
    assert_eq!(r_10k.designators, "R1, R2, R3");

    let r_4k7 = bom.items.iter().find(|i| i.part_key == "R_4.7k").unwrap();
    assert_eq!(r_4k7.quantity, 1);
    assert_eq!(r_4k7.designators, "R4");

    let c_100n = bom.items.iter().find(|i| i.part_key == "C_100nF").unwrap();
    assert_eq!(c_100n.quantity, 2);
    assert_eq!(c_100n.designators, "C1, C2");
}

#[test]
fn test_live_canvas_hierarchical_subcircuit_bom_decomposition() {
    let mut canvas = SchematicCanvas::new();

    // Add discrete component to parent
    canvas.components.push(create_test_comp(1, ComponentKind::Resistor, "R_IN", "1k", Pos2::new(50.0, 50.0)));

    // Add subcircuit definition
    let mut sub_canvas = SchematicCanvas::new();
    sub_canvas.components.push(create_test_comp(10, ComponentKind::OpAmp, "U_AMP", "", Pos2::new(100.0, 100.0)));
    sub_canvas.components.push(create_test_comp(11, ComponentKind::Resistor, "R_FB", "100k", Pos2::new(150.0, 100.0)));

    let mut subcircuits = HashMap::new();
    let sub_def = SubcircuitDefinition::new("OpAmpGainStage", Vec::new(), sub_canvas);
    subcircuits.insert("OpAmpGainStage".to_string(), sub_def);

    // Place an instance of this subcircuit on parent canvas
    let inst = SubcircuitInstance::new(1, "OpAmpGainStage", Pos2::new(200.0, 200.0));
    canvas.subcircuit_instances.push(inst);

    let bom = generate_bom_from_canvas(&canvas, &subcircuits);

    // 2 top-level items: R_IN and U_SUB_1 (OpAmpGainStage)
    assert_eq!(bom.items.len(), 2);

    let sub_item = bom.items.iter().find(|i| i.is_subcircuit).unwrap();
    assert_eq!(sub_item.designators, "U_SUB_1");
    assert_eq!(sub_item.part_key, "PKG_OpAmpGainStage");
    assert_eq!(sub_item.children.len(), 2);

    let child_keys: Vec<String> = sub_item.children.iter().map(|c| c.part_key.clone()).collect();
    assert!(child_keys.contains(&"IC_LM358".to_string()));
    assert!(child_keys.contains(&"R_100k".to_string()));
}

#[test]
fn test_hierarchical_bom_split_vs_lump_sum_rollup() {
    let child1 = BomLineItem::new_component("C1", "R_A", 2, "R_10k", "Resistor 10k");
    let child2 = BomLineItem::new_component("C2", "C_A", 1, "C_100nF", "Capacitor 100nF");

    let mut reg = CentralCostRegistry::new_with_standard_defaults();
    reg.set_cost("R_10k", 0.05);   // 2 * 0.05 = 0.10
    reg.set_cost("C_100nF", 0.15); // 1 * 0.15 = 0.15
    // Sum of children = 0.25

    let sub = BomLineItem::new_subcircuit(
        "S1",
        "U1",
        1,
        "PKG_SUB",
        "Subcircuit",
        1.50, // Lump-sum estimated cost
        vec![child1, child2],
    );

    let mut bom = HierarchicalBom { items: vec![sub] };

    // Initially unsplit: should evaluate to lump sum (1.50)
    assert!(!bom.items[0].is_split);
    assert!((bom.total_prototype_unit_cost(&reg) - 1.50).abs() < 1e-6);

    // Split subcircuit: should evaluate to bottom-up sum of children (0.25)
    bom.split_all();
    assert!(bom.items[0].is_split);
    assert!((bom.total_prototype_unit_cost(&reg) - 0.25).abs() < 1e-6);

    // Unsplit: should return to lump sum (1.50)
    bom.unsplit_all();
    assert!(!bom.items[0].is_split);
    assert!((bom.total_prototype_unit_cost(&reg) - 1.50).abs() < 1e-6);
}

#[test]
fn test_currency_conversions_and_formatting() {
    let usd = Currency::USD;
    let eur = Currency::EUR;
    let gbp = Currency::GBP;
    let jpy = Currency::JPY;

    assert_eq!(usd.symbol(), "$");
    assert_eq!(eur.symbol(), "EUR ");
    assert_eq!(gbp.symbol(), "GBP ");
    assert_eq!(jpy.symbol(), "JPY ");

    let cost_usd = 100.0;
    assert!((usd.convert_from_usd(cost_usd) - 100.0).abs() < 1e-6);
    assert!((eur.convert_from_usd(cost_usd) - 92.0).abs() < 1e-6);
    assert!((gbp.convert_from_usd(cost_usd) - 79.0).abs() < 1e-6);
    assert!((jpy.convert_from_usd(cost_usd) - 15200.0).abs() < 1e-6);

    assert_eq!(usd.format_total(10.0), "$10.00");
    assert_eq!(eur.format_total(10.0), "EUR 9.20");
    assert_eq!(gbp.format_total(10.0), "GBP 7.90");
    assert_eq!(jpy.format_total(10.0), "JPY 1520");
}

#[test]
fn test_bom_csv_and_markdown_exports_with_currency() {
    let bom = HierarchicalBom::new_sample_avionics_power_supply();
    let reg = CentralCostRegistry::new_with_standard_defaults();

    // CSV in EUR
    let csv_eur = bom.export_csv_with_currency(&reg, 1000, Currency::EUR);
    assert!(csv_eur.contains("Item ID,Designators,Qty,Part Key,Description,Unit Cost (EUR),Extended Cost (EUR)"));
    assert!(csv_eur.contains("TOTAL PROTOTYPE UNIT COST (EUR)"));

    // CSV in JPY
    let csv_jpy = bom.export_csv_with_currency(&reg, 1000, Currency::JPY);
    assert!(csv_jpy.contains("Item ID,Designators,Qty,Part Key,Description,Unit Cost (JPY),Extended Cost (JPY)"));
    assert!(csv_jpy.contains("TOTAL BATCH INVESTMENT (N=1000, JPY)"));

    // Markdown in GBP
    let md_gbp = bom.export_markdown_table_with_currency(&reg, 1000, Currency::GBP);
    assert!(md_gbp.contains("| Designator | Qty | Part Key | Description | Unit Price (GBP) | Extended (GBP) |"));
    assert!(md_gbp.contains("GBP "));
}

#[test]
fn test_bom_json_export_structure() {
    let bom = HierarchicalBom::new_sample_avionics_power_supply();
    let reg = CentralCostRegistry::new_with_standard_defaults();

    let json_str = bom.export_json(&reg, 1000, Currency::USD);

    assert!(json_str.starts_with('{'));
    assert!(json_str.ends_with('}'));
    assert!(json_str.contains("\"currency\": \"USD\""));
    assert!(json_str.contains("\"batch_volume\": 1000"));
    assert!(json_str.contains("\"items\": ["));
    assert!(json_str.contains("\"prototype_unit_cost_usd\":"));
    assert!(json_str.contains("\"batch_total_cost_local\":"));
    assert!(json_str.contains("\"total_component_count\":"));
}

#[test]
fn test_binary_project_serialization_with_pricing_roundtrip() {
    let title = "TestAvionicsPowerProject";
    let comps = vec![
        create_test_comp(1, ComponentKind::Resistor, "R1", "10k", Pos2::new(100.0, 100.0)),
        create_test_comp(2, ComponentKind::Capacitor, "C1", "100nF", Pos2::new(150.0, 100.0)),
    ];
    let wires = vec![SchematicWire::manhattan_route_with_net(
        1,
        Pos2::new(100.0, 100.0),
        Pos2::new(150.0, 100.0),
        None,
    )];

    let mut reg = CentralCostRegistry::new();
    reg.set_cost("R_10k", 0.045);
    reg.set_cost("C_100nF", 0.125);
    let pricing_text = reg.serialize_to_text();

    // 1. Serialize with pricing
    let bytes = serialize_project_with_pricing(title, &comps, &wires, Some(&pricing_text));

    // Verify flag
    let flags = u16::from_le_bytes([bytes[10], bytes[11]]);
    assert_ne!(flags & FLAG_HAS_PRICING, 0);

    // 2. Deserialize
    let proj = deserialize_project(&bytes).expect("Failed to deserialize project with pricing");
    assert_eq!(proj.title, title);
    assert_eq!(proj.components.len(), 2);
    assert_eq!(proj.wires.len(), 1);
    assert!(proj.pricing_metadata.is_some());

    let restored_reg = CentralCostRegistry::deserialize_from_text(&proj.pricing_metadata.unwrap());
    assert!((restored_reg.get_cost("R_10k") - 0.045).abs() < 1e-6);
    assert!((restored_reg.get_cost("C_100nF") - 0.125).abs() < 1e-6);
}

#[test]
fn test_binary_project_backward_compatibility_without_pricing() {
    let title = "LegacyProject";
    let comps = vec![
        create_test_comp(1, ComponentKind::Resistor, "R1", "1k", Pos2::new(50.0, 50.0)),
    ];
    let wires = vec![];

    // Serialize using legacy serialize_project without pricing
    let bytes = serialize_project(title, &comps, &wires);

    let flags = u16::from_le_bytes([bytes[10], bytes[11]]);
    assert_eq!(flags & FLAG_HAS_PRICING, 0);

    // Deserialize
    let proj = deserialize_project(&bytes).expect("Failed to deserialize legacy project");
    assert_eq!(proj.title, title);
    assert_eq!(proj.components.len(), 1);
    assert_eq!(proj.wires.len(), 0);
    assert!(proj.pricing_metadata.is_none());
}

#[test]
fn test_production_economics_dialog_sync_and_currency_toggle() {
    let mut dialog = ProductionEconomicsDialog::new_fast();

    let mut canvas = SchematicCanvas::new();
    canvas.components.push(create_test_comp(1, ComponentKind::Resistor, "R1", "10k", Pos2::new(10.0, 10.0)));
    canvas.components.push(create_test_comp(2, ComponentKind::Resistor, "R2", "10k", Pos2::new(20.0, 10.0)));
    canvas.components.push(create_test_comp(3, ComponentKind::OpAmp, "U1", "", Pos2::new(50.0, 50.0)));

    let subcircuits = HashMap::new();

    // Trigger sync
    dialog.sync_from_canvas(&canvas, &subcircuits);

    assert_eq!(dialog.sim.bom.items.len(), 2);
    assert_eq!(dialog.sim.bom.total_component_count(), 3);
    assert!(!dialog.request_sync_canvas);

    // Currency toggle test
    dialog.selected_currency = Currency::EUR;
    assert_eq!(dialog.selected_currency.code(), "EUR");
    dialog.selected_currency = Currency::JPY;
    assert_eq!(dialog.selected_currency.symbol(), "JPY ");
}
