#![deny(unsafe_code)]

//! Hierarchical Bill of Materials (BOM) Decomposition & Bottom-Up Cost Rollup Engine.
//!
//! Supports modular subcircuits (.phnc packages) with interactive split-view pricing:
//! users can maintain an estimated lump-sum parent cost or click "Split" to drill down
//! into child components one-by-one with automated bottom-up rollup.

use super::cost_registry::CentralCostRegistry;

/// Supported currencies for international production costing and quotations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    JPY,
}

impl Currency {
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::USD => "$",
            Self::EUR => "EUR ",
            Self::GBP => "GBP ",
            Self::JPY => "JPY ",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::USD => "USD",
            Self::EUR => "EUR",
            Self::GBP => "GBP",
            Self::JPY => "JPY",
        }
    }

    pub fn rate_from_usd(&self) -> f64 {
        match self {
            Self::USD => 1.0,
            Self::EUR => 0.92,
            Self::GBP => 0.79,
            Self::JPY => 152.0,
        }
    }

    pub fn convert_from_usd(&self, amount_usd: f64) -> f64 {
        amount_usd * self.rate_from_usd()
    }

    pub fn format_amount(&self, amount_usd: f64) -> String {
        let converted = self.convert_from_usd(amount_usd);
        match self {
            Self::JPY => format!("JPY {:.0}", converted),
            _ => format!("{}{:.4}", self.symbol(), converted),
        }
    }

    pub fn format_total(&self, amount_usd: f64) -> String {
        let converted = self.convert_from_usd(amount_usd);
        match self {
            Self::JPY => format!("JPY {:.0}", converted),
            _ => format!("{}{:.2}", self.symbol(), converted),
        }
    }
}

/// An individual line item in the hierarchical Bill of Materials.
#[derive(Debug, Clone, PartialEq)]
pub struct BomLineItem {
    /// Unique item tracking identifier.
    pub id: String,
    /// Human-readable designators (e.g. "R1, R2, R4").
    pub designators: String,
    /// Quantity per board / parent assembly.
    pub quantity: usize,
    /// Lookup key referencing CentralCostRegistry.
    pub part_key: String,
    /// Component functional description.
    pub description: String,
    /// Whether this item represents a modular subcircuit package.
    pub is_subcircuit: bool,
    /// Whether the user has split this subcircuit into internal children.
    pub is_split: bool,
    /// User-specified or estimated lump-sum cost when un-split.
    pub lump_sum_cost: f64,
    /// Internal child components if this item is a subcircuit.
    pub children: Vec<BomLineItem>,
}

impl BomLineItem {
    /// Creates a standard discrete component line item.
    pub fn new_component(
        id: &str,
        designators: &str,
        quantity: usize,
        part_key: &str,
        description: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            designators: designators.to_string(),
            quantity: quantity.max(1),
            part_key: part_key.to_string(),
            description: description.to_string(),
            is_subcircuit: false,
            is_split: false,
            lump_sum_cost: 0.0,
            children: Vec::new(),
        }
    }

    /// Creates a modular subcircuit line item with children.
    pub fn new_subcircuit(
        id: &str,
        designator: &str,
        quantity: usize,
        part_key: &str,
        description: &str,
        lump_sum_cost: f64,
        children: Vec<BomLineItem>,
    ) -> Self {
        Self {
            id: id.to_string(),
            designators: designator.to_string(),
            quantity: quantity.max(1),
            part_key: part_key.to_string(),
            description: description.to_string(),
            is_subcircuit: true,
            is_split: false,
            lump_sum_cost: lump_sum_cost.max(0.0001),
            children,
        }
    }

    /// Toggles the split state of this subcircuit.
    pub fn toggle_split(&mut self) {
        if self.is_subcircuit {
            self.is_split = !self.is_split;
        }
    }

    /// Calculates the effective unit cost for this item.
    /// If it is a split subcircuit, recursively sums children costs bottom-up.
    pub fn effective_unit_cost(&self, registry: &CentralCostRegistry) -> f64 {
        if self.is_subcircuit {
            if self.is_split && !self.children.is_empty() {
                // Bottom-up rollup of child component costs
                self.children
                    .iter()
                    .map(|child| child.effective_unit_cost(registry) * (child.quantity as f64))
                    .sum()
            } else {
                self.lump_sum_cost
            }
        } else {
            registry.get_cost(&self.part_key)
        }
    }

    /// Extended cost = effective_unit_cost * quantity.
    pub fn extended_cost(&self, registry: &CentralCostRegistry) -> f64 {
        self.effective_unit_cost(registry) * (self.quantity as f64)
    }

    /// Recursively searches for an item or sub-child by ID.
    pub fn find_mut(&mut self, target_id: &str) -> Option<&mut BomLineItem> {
        if self.id == target_id {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(found) = child.find_mut(target_id) {
                return Some(found);
            }
        }
        None
    }
}

/// Complete hierarchical Bill of Materials for a schematic project.
#[derive(Debug, Clone, PartialEq)]
pub struct HierarchicalBom {
    /// Top-level BOM line items.
    pub items: Vec<BomLineItem>,
}

impl Default for HierarchicalBom {
    fn default() -> Self {
        Self::new_sample_avionics_power_supply()
    }
}

impl HierarchicalBom {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Constructs a representative aerospace / avionics sensor power supply BOM.
    pub fn new_sample_avionics_power_supply() -> Self {
        let mut items = Vec::new();

        items.push(BomLineItem::new_component(
            "ITEM_R_10k",
            "R1, R2, R5, R6",
            4,
            "R_10k",
            "10k Ohm 0805 1% SMD Resistor",
        ));

        items.push(BomLineItem::new_component(
            "ITEM_R_1k",
            "R3, R4",
            2,
            "R_1k",
            "1k Ohm 0805 1% SMD Resistor",
        ));

        items.push(BomLineItem::new_component(
            "ITEM_C_100nF",
            "C1, C2, C3, C4",
            4,
            "C_100nF",
            "100nF 50V X7R 0805 Ceramic Capacitor",
        ));

        items.push(BomLineItem::new_component(
            "ITEM_Q_BC547B",
            "Q1, Q2",
            2,
            "Q_BC547B",
            "NPN Transistor SOT-23",
        ));

        items.push(BomLineItem::new_component(
            "ITEM_D_1N4148",
            "D1, D2",
            2,
            "D_1N4148",
            "High-speed switching diode",
        ));

        items.push(BomLineItem::new_component(
            "ITEM_IC_LM358",
            "U1",
            1,
            "IC_LM358",
            "Dual Operational Amplifier SOIC-8",
        ));

        // Modular Subcircuit Package: Synchronous Buck Regulator (.phnc)
        let subcircuit_children = vec![
            BomLineItem::new_component("SUB_L1", "L101", 1, "L_INDUCTOR_GENERIC", "Power Inductor 4.7uH"),
            BomLineItem::new_component("SUB_C1", "C101", 2, "C_10uF", "10uF 25V Output MLCC"),
            BomLineItem::new_component("SUB_Q1", "M101", 1, "Q_2N7002", "High-side Switching FET"),
            BomLineItem::new_component("SUB_Q2", "M102", 1, "Q_2N7002", "Low-side Sync Rectifier FET"),
            BomLineItem::new_component("SUB_R1", "R101, R102", 2, "R_10k", "Feedback Divider Network"),
            BomLineItem::new_component("SUB_IC", "U101", 1, "IC_NE555P", "PWM Controller Core"),
        ];

        items.push(BomLineItem::new_subcircuit(
            "ITEM_SUB_REG",
            "SUB1",
            1,
            "SUB_POWER_REGULATOR",
            "Synchronous DC-DC Buck Module",
            1.85,
            subcircuit_children,
        ));

        Self { items }
    }

    /// Finds a line item by ID across top-level items and nested subcircuits.
    pub fn find_item_mut(&mut self, id: &str) -> Option<&mut BomLineItem> {
        for item in &mut self.items {
            if let Some(found) = item.find_mut(id) {
                return Some(found);
            }
        }
        None
    }

    /// Computes the total prototype unit cost (sum of extended costs of top-level items).
    pub fn total_prototype_unit_cost(&self, registry: &CentralCostRegistry) -> f64 {
        let total: f64 = self.items.iter().map(|item| item.extended_cost(registry)).sum();
        (total * 1000.0).round() / 1000.0
    }

    /// Total component parts count in the assembly.
    pub fn total_component_count(&self) -> usize {
        let mut count = 0;
        for item in &self.items {
            if item.is_subcircuit && item.is_split {
                let sub_sum: usize = item.children.iter().map(|c| c.quantity).sum();
                count += sub_sum * item.quantity;
            } else {
                count += item.quantity;
            }
        }
        count
    }

    /// Count of unique line items.
    pub fn unique_line_items_count(&self) -> usize {
        let mut count = 0;
        for item in &self.items {
            if item.is_subcircuit && item.is_split {
                count += item.children.len();
            } else {
                count += 1;
            }
        }
        count
    }

    /// Returns (split_subcircuits_count, total_subcircuits_count).
    pub fn subcircuit_split_stats(&self) -> (usize, usize) {
        let mut split = 0;
        let mut total = 0;
        for item in &self.items {
            if item.is_subcircuit {
                total += 1;
                if item.is_split {
                    split += 1;
                }
            }
        }
        (split, total)
    }

    /// Decomposes (splits) all subcircuits in the BOM.
    pub fn split_all(&mut self) {
        for item in &mut self.items {
            if item.is_subcircuit {
                item.is_split = true;
            }
        }
    }

    /// Collapses all subcircuits back to their lump-sum parent costs.
    pub fn unsplit_all(&mut self) {
        for item in &mut self.items {
            if item.is_subcircuit {
                item.is_split = false;
            }
        }
    }

    /// Exports the Bill of Materials as standard comma-separated values (CSV).
    pub fn export_csv(&self, registry: &CentralCostRegistry, batch_volume: u32) -> String {
        self.export_csv_with_currency(registry, batch_volume, Currency::USD)
    }

    /// Exports the Bill of Materials as standard CSV localized to target currency.
    pub fn export_csv_with_currency(
        &self,
        registry: &CentralCostRegistry,
        batch_volume: u32,
        currency: Currency,
    ) -> String {
        let mut csv = String::new();
        csv.push_str(&format!(
            "Item ID,Designators,Qty,Part Key,Description,Unit Cost ({}),Extended Cost ({})\n",
            currency.code(),
            currency.code()
        ));

        for item in &self.items {
            let unit_cost = currency.convert_from_usd(item.effective_unit_cost(registry));
            let ext_cost = currency.convert_from_usd(item.extended_cost(registry));
            csv.push_str(&format!(
                "\"{}\",\"{}\",{},\"{}\",\"{}\",{:.4},{:.4}\n",
                item.id, item.designators, item.quantity, item.part_key, item.description, unit_cost, ext_cost
            ));

            if item.is_subcircuit && item.is_split {
                for child in &item.children {
                    let c_unit = currency.convert_from_usd(child.effective_unit_cost(registry));
                    let c_ext = currency.convert_from_usd(child.extended_cost(registry));
                    csv.push_str(&format!(
                        "\"  -> {}\",\"{}\",{},\"{}\",\"{}\",{:.4},{:.4}\n",
                        child.id, child.designators, child.quantity, child.part_key, child.description, c_unit, c_ext
                    ));
                }
            }
        }

        let total_unit = currency.convert_from_usd(self.total_prototype_unit_cost(registry));
        let batch_total = total_unit * (batch_volume as f64);
        csv.push_str(&format!(
            "\n\"TOTAL PROTOTYPE UNIT COST ({})\",,,,,,{:.4}\n",
            currency.code(),
            total_unit
        ));
        csv.push_str(&format!(
            "\"TOTAL BATCH INVESTMENT (N={}, {})\",,,,,,{:.2}\n",
            batch_volume,
            currency.code(),
            batch_total
        ));

        csv
    }

    /// Exports the Bill of Materials as a clean Markdown table.
    pub fn export_markdown_table(&self, registry: &CentralCostRegistry, batch_volume: u32) -> String {
        self.export_markdown_table_with_currency(registry, batch_volume, Currency::USD)
    }

    /// Exports the Bill of Materials as a clean Markdown table localized to target currency.
    pub fn export_markdown_table_with_currency(
        &self,
        registry: &CentralCostRegistry,
        batch_volume: u32,
        currency: Currency,
    ) -> String {
        let mut md = String::new();
        md.push_str(&format!(
            "| Designator | Qty | Part Key | Description | Unit Price ({}) | Extended ({}) |\n",
            currency.code(),
            currency.code()
        ));
        md.push_str("|:---|:---:|:---|:---|:---:|:---:|\n");

        for item in &self.items {
            let unit_cost = currency.convert_from_usd(item.effective_unit_cost(registry));
            let ext_cost = currency.convert_from_usd(item.extended_cost(registry));
            let prefix = if item.is_subcircuit {
                if item.is_split { "[SPLIT SUB] " } else { "[SUB] " }
            } else {
                ""
            };

            md.push_str(&format!(
                "| {}{} | {} | `{}` | {} | {}{:.4} | {}{:.4} |\n",
                prefix, item.designators, item.quantity, item.part_key, item.description, currency.symbol(), unit_cost, currency.symbol(), ext_cost
            ));

            if item.is_subcircuit && item.is_split {
                for child in &item.children {
                    let c_unit = currency.convert_from_usd(child.effective_unit_cost(registry));
                    let c_ext = currency.convert_from_usd(child.extended_cost(registry));
                    md.push_str(&format!(
                        "| &nbsp;&nbsp;&rarr; `{}` | {} | `{}` | {} | {}{:.4} | {}{:.4} |\n",
                        child.designators, child.quantity, child.part_key, child.description, currency.symbol(), c_unit, currency.symbol(), c_ext
                    ));
                }
            }
        }

        let total_unit = currency.convert_from_usd(self.total_prototype_unit_cost(registry));
        let batch_total = total_unit * (batch_volume as f64);
        md.push_str(&format!(
            "\n**Total Prototype Unit Cost**: {}{:.4} | **Batch Total (N={})**: {}{:.2}\n",
            currency.symbol(), total_unit, batch_volume, currency.symbol(), batch_total
        ));

        md
    }

    /// Exports the Bill of Materials as structured JSON without external dependencies.
    pub fn export_json(
        &self,
        registry: &CentralCostRegistry,
        batch_volume: u32,
        currency: Currency,
    ) -> String {
        let proto_unit_usd = self.total_prototype_unit_cost(registry);
        let proto_unit_local = currency.convert_from_usd(proto_unit_usd);
        let batch_total_local = proto_unit_local * (batch_volume as f64);

        let mut lines = Vec::new();
        lines.push("{".to_string());
        lines.push(format!("  \"currency\": \"{}\",", currency.code()));
        lines.push(format!("  \"exchange_rate\": {:.4},", currency.rate_from_usd()));
        lines.push(format!("  \"batch_volume\": {},", batch_volume));
        lines.push(format!("  \"prototype_unit_cost_usd\": {:.6},", proto_unit_usd));
        lines.push(format!("  \"prototype_unit_cost_local\": {:.6},", proto_unit_local));
        lines.push(format!("  \"batch_total_cost_local\": {:.4},", batch_total_local));
        lines.push(format!("  \"total_component_count\": {},", self.total_component_count()));
        lines.push(format!("  \"unique_line_items\": {},", self.unique_line_items_count()));
        lines.push("  \"items\": [".to_string());

        for (idx, item) in self.items.iter().enumerate() {
            let u_usd = item.effective_unit_cost(registry);
            let u_loc = currency.convert_from_usd(u_usd);
            let ext_loc = currency.convert_from_usd(item.extended_cost(registry));
            let comma = if idx + 1 < self.items.len() { "," } else { "" };

            lines.push("    {".to_string());
            lines.push(format!("      \"id\": \"{}\",", item.id));
            lines.push(format!("      \"designators\": \"{}\",", item.designators));
            lines.push(format!("      \"quantity\": {},", item.quantity));
            lines.push(format!("      \"part_key\": \"{}\",", item.part_key));
            lines.push(format!("      \"description\": \"{}\",", item.description));
            lines.push(format!("      \"is_subcircuit\": {},", item.is_subcircuit));
            lines.push(format!("      \"is_split\": {},", item.is_split));
            lines.push(format!("      \"unit_cost_usd\": {:.6},", u_usd));
            lines.push(format!("      \"unit_cost_local\": {:.6},", u_loc));
            lines.push(format!("      \"extended_cost_local\": {:.6}", ext_loc));

            if item.is_subcircuit && !item.children.is_empty() {
                lines.push(",      \"children\": [".to_string());
                for (c_idx, child) in item.children.iter().enumerate() {
                    let cu_usd = child.effective_unit_cost(registry);
                    let cu_loc = currency.convert_from_usd(cu_usd);
                    let cext_loc = currency.convert_from_usd(child.extended_cost(registry));
                    let c_comma = if c_idx + 1 < item.children.len() { "," } else { "" };

                    lines.push("        {".to_string());
                    lines.push(format!("          \"id\": \"{}\",", child.id));
                    lines.push(format!("          \"designators\": \"{}\",", child.designators));
                    lines.push(format!("          \"quantity\": {},", child.quantity));
                    lines.push(format!("          \"part_key\": \"{}\",", child.part_key));
                    lines.push(format!("          \"description\": \"{}\",", child.description));
                    lines.push(format!("          \"unit_cost_local\": {:.6},", cu_loc));
                    lines.push(format!("          \"extended_cost_local\": {:.6}", cext_loc));
                    lines.push(format!("        }}{}", c_comma));
                }
                lines.push("      ]".to_string());
            }

            lines.push(format!("    }}{}", comma));
        }

        lines.push("  ]".to_string());
        lines.push("}".to_string());

        lines.join("\n")
    }
}
