#![deny(unsafe_code)]

//! Comprehensive Automated Test Suite for Electronic Distributor API & PCBA Quoting Pipeline.

use phonon_solver::distributor_api::{
    audit_distributor_quoting, AssemblySourcingMode, ComponentGrade, ComponentMarketComparison,
    DistributorKind, DistributorQuote, DistributorQuotingEngine, LifecycleStatus, MpnResolverEngine,
    PackagingType, PcbaQuotingEngine, PurchaseOrderLineItem, SupplierPurchaseOrder, SurfaceFinish,
};

#[test]
fn test_multi_distributor_quote_and_price_breaks() {
    let mut comp = ComponentMarketComparison::new("RC0805FR-0710KL", "Resistor 10k", "0805");

    let dk_quote = DistributorQuote {
        distributor: DistributorKind::DigiKey,
        distributor_part_number: "311-10.0KCRCT-ND".to_string(),
        manufacturer_part_number: "RC0805FR-0710KL".to_string(),
        manufacturer_name: "Yageo".to_string(),
        description: "RES 10K OHM 1% 1/8W 0805".to_string(),
        in_stock_quantity: 50000,
        lead_time_weeks: 1.0,
        minimum_order_quantity: 1,
        order_multiple: 1,
        packaging: PackagingType::CutTape,
        lifecycle: LifecycleStatus::Active,
        price_breaks: vec![
            (1, 0.05),
            (10, 0.04),
            (100, 0.025),
            (1000, 0.012),
            (10000, 0.009),
        ],
        datasheet_url: "https://example.com/datasheet.pdf".to_string(),
    };

    let p1 = dk_quote.unit_price_at_qty(1);
    let p10 = dk_quote.unit_price_at_qty(10);
    let p100 = dk_quote.unit_price_at_qty(100);
    let p1000 = dk_quote.unit_price_at_qty(1000);
    let p10000 = dk_quote.unit_price_at_qty(10000);

    assert!(p1 >= p10);
    assert!(p10 >= p100);
    assert!(p100 >= p1000);
    assert!(p1000 >= p10000);

    let (qty_ordered, cost) = dk_quote.extended_cost_at_qty(250);
    assert_eq!(qty_ordered, 250);
    assert!((cost - 250.0 * 0.025).abs() < 1e-6);

    comp.quotes.push(dk_quote);

    let lc_quote = DistributorQuote {
        distributor: DistributorKind::Lcsc,
        distributor_part_number: "C17414".to_string(),
        manufacturer_part_number: "RC0805FR-0710KL".to_string(),
        manufacturer_name: "Yageo".to_string(),
        description: "RES 10K OHM 1% 1/8W 0805".to_string(),
        in_stock_quantity: 100000,
        lead_time_weeks: 1.5,
        minimum_order_quantity: 10,
        order_multiple: 10,
        packaging: PackagingType::CutTape,
        lifecycle: LifecycleStatus::Active,
        price_breaks: vec![
            (1, 0.03),
            (100, 0.015),
            (1000, 0.007),
        ],
        datasheet_url: "https://example.com/datasheet.pdf".to_string(),
    };
    comp.quotes.push(lc_quote);

    let lowest = comp.find_lowest_cost_supplier(1000).expect("lowest quote");
    assert_eq!(lowest.distributor, DistributorKind::Lcsc);
}

#[test]
fn test_mpn_resolver_accuracy() {
    let resolver = MpnResolverEngine::new();

    let res_r = resolver.resolve("R1", "Resistor", "10k", "0805");
    assert!(res_r.match_confidence >= 0.90);
    assert!(!res_r.primary_mpn.is_empty());
    assert!(!res_r.second_source_alternatives.is_empty());
    assert_eq!(res_r.grade, ComponentGrade::Industrial);

    let res_c = resolver.resolve("C1", "Capacitor", "100nF", "0805");
    assert!(res_c.match_confidence >= 0.90);
    assert!(res_c.second_source_alternatives.iter().any(|a| a.is_drop_in_compatible));

    let res_ic = resolver.resolve("U1", "OpAmp", "LM358", "SOIC-8");
    assert!(res_ic.match_confidence >= 0.85);
    assert!(res_ic.primary_mpn.contains("358"));
}

#[test]
fn test_pcba_panelization_and_costing() {
    let engine = PcbaQuotingEngine::new(45.0, 45.0, 2, 70, 2);

    assert!(engine.panelization.panel_utilization_efficiency >= 0.70);
    assert!(engine.panelization.boards_per_panel >= 4);

    let q1 = engine.quote_at_volume(1, 10.0);
    let q100 = engine.quote_at_volume(100, 10.0);
    let q1000 = engine.quote_at_volume(1000, 10.0);

    assert!(q1.total_pcba_unit_cost > q100.total_pcba_unit_cost);
    assert!(q100.total_pcba_unit_cost > q1000.total_pcba_unit_cost);

    let mut consigned = engine.clone();
    consigned.sourcing_mode = AssemblySourcingMode::ConsignedCustomer;
    let qc = consigned.quote_at_volume(100, 10.0);
    assert_eq!(qc.component_bom_unit_cost, 0.0);
    assert!(qc.total_pcba_unit_cost < q100.total_pcba_unit_cost);

    let mut hasl = engine.clone();
    hasl.surface_finish = SurfaceFinish::HaslLeadFree;
    let q_hasl = hasl.quote_at_volume(100, 10.0);
    assert!(q100.bare_pcb_unit_cost > q_hasl.bare_pcb_unit_cost);
}

#[test]
fn test_purchase_order_export() {
    let mut po = SupplierPurchaseOrder::new(DistributorKind::Mouser, "PO-2026-TEST");
    po.add_item(PurchaseOrderLineItem {
        line_number: 1,
        manufacturer_part_number: "RC0805FR-0710KL".to_string(),
        distributor_sku: "603-RC0805FR-0710KL".to_string(),
        manufacturer_name: "Yageo".to_string(),
        designators_reference: "R1, R2".to_string(),
        description: "RES 10K 0805".to_string(),
        requested_quantity: 100,
        ordered_quantity: 100,
        unit_price_usd: 0.02,
        extended_cost_usd: 2.0,
    });

    let csv = po.to_distributor_csv();
    assert!(csv.contains("MouserPartNumber"));
    assert!(csv.contains("603-RC0805FR-0710KL"));

    let json = po.to_erp_json();
    assert!(json.contains("\"distributor\": \"MOUSER\""));
    assert!(json.contains("\"po_reference\": \"PO-2026-TEST\""));

    let xml = po.to_erp_xml();
    assert!(xml.contains("<Distributor>MOUSER</Distributor>"));
    assert!(xml.contains("<PurchaseOrder>"));
}

#[test]
fn test_distributor_audit_full_pass() {
    let report = audit_distributor_quoting();
    assert_eq!(report.passed_count, 10);
    assert_eq!(report.total_count, 10);
    assert!(report.overall_pass);
    assert!(report.is_all_pass());
    assert!(report.cold_boot_latency_us < 2000.0);
}

#[test]
fn test_quoting_engine_orchestration() {
    let mut engine = DistributorQuotingEngine::new_fast();
    assert_eq!(engine.comparisons.len(), 5);
    assert!(engine.procurement.total_procurement_cost() > 0.0);

    engine.set_target_volume(500);
    assert_eq!(engine.target_volume, 500);

    let pcba_quote = engine.current_pcba_quote();
    assert_eq!(pcba_quote.batch_volume_units, 500);
    assert!(pcba_quote.total_batch_cost > 0.0);

    let audit = engine.run_full_audit();
    assert!(audit.overall_pass);
}
