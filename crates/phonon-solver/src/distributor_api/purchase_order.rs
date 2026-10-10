#![deny(unsafe_code)]

//! Consolidated Multi-Distributor Purchase Order (PO) Generator and ERP Exporter.
//!
//! Generates supplier-split purchase orders with automated MOQ padding, price break
//! optimization, and standard CSV/JSON/XML formatting for Digi-Key, Mouser, and LCSC.

use super::distributors::DistributorKind;

/// An individual line item in a consolidated or supplier-specific purchase order.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PurchaseOrderLineItem {
    pub line_number: usize,
    pub manufacturer_part_number: String,
    pub distributor_sku: String,
    pub manufacturer_name: String,
    pub designators_reference: String,
    pub description: String,
    pub requested_quantity: usize,
    pub ordered_quantity: usize,
    pub unit_price_usd: f64,
    pub extended_cost_usd: f64,
}

/// A supplier-specific purchase order ready for online checkout or ERP injection.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SupplierPurchaseOrder {
    pub distributor: DistributorKind,
    pub po_reference_number: String,
    pub created_timestamp: String,
    pub line_items: Vec<PurchaseOrderLineItem>,
    pub subtotal_usd: f64,
    pub estimated_shipping_usd: f64,
    pub total_cost_usd: f64,
}

impl SupplierPurchaseOrder {
    pub fn new(distributor: DistributorKind, po_ref: &str) -> Self {
        Self {
            distributor,
            po_reference_number: po_ref.to_string(),
            created_timestamp: "2026-10-09T23:45:00Z".to_string(),
            line_items: Vec::new(),
            subtotal_usd: 0.0,
            estimated_shipping_usd: 12.0, // Standard priority ground/courier
            total_cost_usd: 12.0,
        }
    }

    pub fn add_item(&mut self, item: PurchaseOrderLineItem) {
        self.subtotal_usd += item.extended_cost_usd;
        self.total_cost_usd = self.subtotal_usd + self.estimated_shipping_usd;
        self.line_items.push(item);
    }

    /// Formats the purchase order as a distributor-specific CSV file.
    pub fn to_distributor_csv(&self) -> String {
        match self.distributor {
            DistributorKind::DigiKey => {
                let mut out = String::from("DigiKeyPartNumber,Quantity,CustomerReference\n");
                for item in &self.line_items {
                    out.push_str(&format!(
                        "{},{},{}\n",
                        item.distributor_sku, item.ordered_quantity, item.designators_reference
                    ));
                }
                out
            }
            DistributorKind::Mouser => {
                let mut out = String::from("MouserPartNumber,Quantity,CustomerPartNumber\n");
                for item in &self.line_items {
                    out.push_str(&format!(
                        "{},{},{}\n",
                        item.distributor_sku, item.ordered_quantity, item.designators_reference
                    ));
                }
                out
            }
            DistributorKind::Lcsc => {
                let mut out = String::from("LCSCPartNumber,Quantity,Designator\n");
                for item in &self.line_items {
                    out.push_str(&format!(
                        "{},{},{}\n",
                        item.distributor_sku, item.ordered_quantity, item.designators_reference
                    ));
                }
                out
            }
        }
    }

    /// Formats the purchase order as an ERP-compatible JSON document.
    pub fn to_erp_json(&self) -> String {
        let mut out = String::from("{\n");
        out.push_str(&format!("  \"distributor\": \"{}\",\n", self.distributor.code()));
        out.push_str(&format!("  \"po_reference\": \"{}\",\n", self.po_reference_number));
        out.push_str(&format!("  \"subtotal_usd\": {:.2},\n", self.subtotal_usd));
        out.push_str(&format!("  \"total_usd\": {:.2},\n", self.total_cost_usd));
        out.push_str("  \"items\": [\n");

        for (idx, item) in self.line_items.iter().enumerate() {
            out.push_str("    {\n");
            out.push_str(&format!("      \"line\": {},\n", item.line_number));
            out.push_str(&format!("      \"mpn\": \"{}\",\n", item.manufacturer_part_number));
            out.push_str(&format!("      \"sku\": \"{}\",\n", item.distributor_sku));
            out.push_str(&format!("      \"qty\": {},\n", item.ordered_quantity));
            out.push_str(&format!("      \"unit_price\": {:.4},\n", item.unit_price_usd));
            out.push_str(&format!("      \"extended\": {:.2}\n", item.extended_cost_usd));
            if idx + 1 < self.line_items.len() {
                out.push_str("    },\n");
            } else {
                out.push_str("    }\n");
            }
        }

        out.push_str("  ]\n}\n");
        out
    }

    /// Formats the purchase order as an industrial XML document.
    pub fn to_erp_xml(&self) -> String {
        let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<PurchaseOrder>\n");
        out.push_str(&format!("  <Distributor>{}</Distributor>\n", self.distributor.code()));
        out.push_str(&format!("  <Reference>{}</Reference>\n", self.po_reference_number));
        out.push_str(&format!("  <TotalCost currency=\"USD\">{:.2}</TotalCost>\n", self.total_cost_usd));
        out.push_str("  <LineItems>\n");

        for item in &self.line_items {
            out.push_str(&format!(
                "    <Item line=\"{}\" sku=\"{}\" qty=\"{}\" unit=\"{:.4}\" ext=\"{:.2}\"/>\n",
                item.line_number, item.distributor_sku, item.ordered_quantity, item.unit_price_usd, item.extended_cost_usd
            ));
        }

        out.push_str("  </LineItems>\n</PurchaseOrder>\n");
        out
    }
}

/// Consolidated multi-distributor procurement manager.
#[derive(Debug, Clone, Default)]
pub struct ConsolidatedProcurementManager {
    pub digikey_po: Option<SupplierPurchaseOrder>,
    pub mouser_po: Option<SupplierPurchaseOrder>,
    pub lcsc_po: Option<SupplierPurchaseOrder>,
}

impl ConsolidatedProcurementManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn total_procurement_cost(&self) -> f64 {
        let d = self.digikey_po.as_ref().map(|p| p.total_cost_usd).unwrap_or(0.0);
        let m = self.mouser_po.as_ref().map(|p| p.total_cost_usd).unwrap_or(0.0);
        let l = self.lcsc_po.as_ref().map(|p| p.total_cost_usd).unwrap_or(0.0);
        d + m + l
    }
}
