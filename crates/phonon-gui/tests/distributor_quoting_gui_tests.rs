#![deny(unsafe_code)]

//! GUI and headless integration test suite for Electronic Distributor API & PCBA Quoting Dialog.

use std::time::Instant;
use egui::Context;
use phonon_gui::widgets::distributor_quoting_dialog::{
    DistributorQuotingDialog, DistributorQuotingTab, PoExportFormat,
};
use phonon_solver::distributor_api::{DistributorKind, SurfaceFinish, AssemblySourcingMode};

#[test]
fn test_distributor_dialog_cold_boot_latency() {
    let start = Instant::now();
    let dialog = DistributorQuotingDialog::new_fast();
    let elapsed = start.elapsed();

    // Verify sub-5.0 ms cold-boot latency threshold
    assert!(
        elapsed.as_secs_f64() < 0.005,
        "Cold boot latency took {:.2} ms, expected < 5.0 ms",
        elapsed.as_secs_f64() * 1000.0
    );
    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, DistributorQuotingTab::LivePricing);
    assert_eq!(dialog.engine.comparisons.len(), 5);
}

#[test]
fn test_distributor_dialog_tab_switching() {
    let mut dialog = DistributorQuotingDialog::new_fast();

    dialog.active_tab = DistributorQuotingTab::ParametricMpn;
    assert_eq!(dialog.active_tab, DistributorQuotingTab::ParametricMpn);

    dialog.active_tab = DistributorQuotingTab::TurnkeyPcba;
    assert_eq!(dialog.active_tab, DistributorQuotingTab::TurnkeyPcba);

    dialog.active_tab = DistributorQuotingTab::PurchaseOrders;
    assert_eq!(dialog.active_tab, DistributorQuotingTab::PurchaseOrders);

    dialog.active_tab = DistributorQuotingTab::AuditTelemetry;
    assert_eq!(dialog.active_tab, DistributorQuotingTab::AuditTelemetry);

    dialog.active_tab = DistributorQuotingTab::LivePricing;
    assert_eq!(dialog.active_tab, DistributorQuotingTab::LivePricing);
}

#[test]
fn test_distributor_dialog_volume_and_quoting_state() {
    let mut dialog = DistributorQuotingDialog::new_fast();

    dialog.target_volume = 500;
    dialog.engine.set_target_volume(500);

    let quote = dialog.engine.current_pcba_quote();
    assert_eq!(quote.batch_volume_units, 500);
    assert!(quote.total_pcba_unit_cost > 0.0);

    dialog.board_width_mm = 80.0;
    dialog.board_length_mm = 60.0;
    dialog.surface_finish = SurfaceFinish::EnigElectrolessNickelImmersionGold;
    dialog.sourcing_mode = AssemblySourcingMode::TurnkeyFull;

    dialog.engine.pcba_engine.board_width_mm = dialog.board_width_mm;
    dialog.engine.pcba_engine.board_length_mm = dialog.board_length_mm;
    dialog.engine.pcba_engine.surface_finish = dialog.surface_finish;

    let updated_quote = dialog.engine.current_pcba_quote();
    assert!(updated_quote.total_batch_cost > 0.0);
}

#[test]
fn test_distributor_dialog_purchase_order_formats() {
    let mut dialog = DistributorQuotingDialog::new_fast();

    dialog.selected_po_distributor = DistributorKind::DigiKey;
    dialog.selected_po_format = PoExportFormat::DistributorCsv;

    let dk_po = dialog.engine.procurement.digikey_po.as_ref().expect("DigiKey PO exists");
    let csv = dk_po.to_distributor_csv();
    assert!(csv.contains("DigiKeyPartNumber"));

    dialog.selected_po_distributor = DistributorKind::Mouser;
    dialog.selected_po_format = PoExportFormat::ErpJson;
    let ms_po = dialog.engine.procurement.mouser_po.as_ref().expect("Mouser PO exists");
    let json = ms_po.to_erp_json();
    assert!(json.contains("\"distributor\": \"MOUSER\""));

    dialog.selected_po_distributor = DistributorKind::Lcsc;
    dialog.selected_po_format = PoExportFormat::ErpXml;
    let lc_po = dialog.engine.procurement.lcsc_po.as_ref().expect("LCSC PO exists");
    let xml = lc_po.to_erp_xml();
    assert!(xml.contains("<Distributor>LCSC</Distributor>"));
}

#[test]
fn test_distributor_dialog_headless_render_pass() {
    let mut dialog = DistributorQuotingDialog::new_fast();
    dialog.is_open = true;

    let ctx = Context::default();

    // Render across all 5 tabs in a headless egui frame
    let tabs = [
        DistributorQuotingTab::LivePricing,
        DistributorQuotingTab::ParametricMpn,
        DistributorQuotingTab::TurnkeyPcba,
        DistributorQuotingTab::PurchaseOrders,
        DistributorQuotingTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        out.textures_delta.clear();
    }

    let mut out_window = ctx.run_ui(Default::default(), |ui| {
        dialog.ui(ui.ctx());
    });
    out_window.textures_delta.clear();

    assert!(dialog.is_open);
}
