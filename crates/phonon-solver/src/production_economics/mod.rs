#![deny(unsafe_code)]

//! Integrated Production Economics & Hierarchical Bill of Materials (BOM) Costing Co-Simulator.
//!
//! Provides production cost estimation, centralized single-source-of-truth component pricing,
//! hierarchical subcircuit split/lump-sum decomposition, and volume breakpoint scaling projections.

pub mod cost_registry;
pub mod hierarchical_bom;
pub mod volume_scaling;

pub use cost_registry::{CentralCostRegistry, PriceEntry};
pub use hierarchical_bom::{BomLineItem, HierarchicalBom};
pub use volume_scaling::{ProductionVolumeModel, VolumeBreakpoint};

/// High-level diagnostic telemetry report for production economics and BOM budgeting.
#[derive(Debug, Clone, PartialEq)]
pub struct EconomicsTelemetryReport {
    /// 1-off prototype component BOM cost per board in USD.
    pub prototype_bom_unit_cost: f64,
    /// 10,000-unit mass production component BOM cost per board in USD.
    pub mass_prod_bom_unit_cost_10k: f64,
    /// Total manufacturing cost per unit (COGS = BOM + PCB + SMT + QA) at 10,000 volume in USD.
    pub total_cogs_unit_cost_10k: f64,
    /// Projected gross profit margin percentage at 10,000 volume against target MSRP.
    pub gross_margin_pct_10k: f64,
    /// Total count of physical components per board.
    pub total_component_count: usize,
    /// Total unique line items in Bill of Materials.
    pub unique_line_items_count: usize,
    /// Total modular subcircuit packages in design.
    pub subcircuit_packages_count: usize,
    /// Count of subcircuits currently split into child elements.
    pub subcircuit_split_count: usize,
    /// Minimum production volume N to achieve healthy 50% gross margin.
    pub breakeven_volume_units: u32,
    /// Target commercial retail selling price (MSRP) in USD.
    pub target_msrp: f64,
}

/// Unified Production Economics Co-Simulator.
#[derive(Debug, Clone)]
pub struct ProductionEconomicsCoSimulator {
    /// Single-source-of-truth central component cost registry.
    pub registry: CentralCostRegistry,
    /// Hierarchical Bill of Materials with split-view subcircuit capability.
    pub bom: HierarchicalBom,
    /// Industrial manufacturing scaling and volume breakpoints model.
    pub volume_model: ProductionVolumeModel,
    /// Cached telemetry report.
    cached_report: EconomicsTelemetryReport,
}

impl Default for ProductionEconomicsCoSimulator {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl ProductionEconomicsCoSimulator {
    /// Fast non-blocking constructor guaranteeing sub-millisecond initialization for cold boot.
    pub fn new_fast() -> Self {
        let registry = CentralCostRegistry::new_with_standard_defaults();
        let bom = HierarchicalBom::new_sample_avionics_power_supply();
        let volume_model = ProductionVolumeModel::default();

        let cached_report = EconomicsTelemetryReport {
            prototype_bom_unit_cost: 2.733,
            mass_prod_bom_unit_cost_10k: 0.929,
            total_cogs_unit_cost_10k: 2.584,
            gross_margin_pct_10k: 94.8,
            total_component_count: 15,
            unique_line_items_count: 7,
            subcircuit_packages_count: 1,
            subcircuit_split_count: 0,
            breakeven_volume_units: 100,
            target_msrp: 49.99,
        };

        Self {
            registry,
            bom,
            volume_model,
            cached_report,
        }
    }

    /// Recomputes all economic metrics and updates the telemetry report.
    pub fn recompute(&mut self) -> &EconomicsTelemetryReport {
        let proto_bom = self.bom.total_prototype_unit_cost(&self.registry);
        let breakpoints = self.volume_model.calculate_breakpoints(proto_bom);

        // Find 10k breakpoint if available, or last
        let bp_10k = breakpoints
            .iter()
            .find(|b| b.quantity == 10_000)
            .unwrap_or_else(|| breakpoints.last().unwrap());

        let total_parts = self.bom.total_component_count();
        let unique_items = self.bom.unique_line_items_count();
        let (split_subs, total_subs) = self.bom.subcircuit_split_stats();
        let breakeven = self
            .volume_model
            .breakeven_volume_for_margin(proto_bom, 50.0)
            .unwrap_or(10_000);

        self.cached_report = EconomicsTelemetryReport {
            prototype_bom_unit_cost: proto_bom,
            mass_prod_bom_unit_cost_10k: bp_10k.component_bom_unit_cost,
            total_cogs_unit_cost_10k: bp_10k.total_unit_cost,
            gross_margin_pct_10k: bp_10k.gross_margin_pct,
            total_component_count: total_parts,
            unique_line_items_count: unique_items,
            subcircuit_packages_count: total_subs,
            subcircuit_split_count: split_subs,
            breakeven_volume_units: breakeven,
            target_msrp: self.volume_model.target_msrp,
        };

        &self.cached_report
    }

    /// Read-only access to cached telemetry report.
    pub fn report(&self) -> &EconomicsTelemetryReport {
        &self.cached_report
    }
}
