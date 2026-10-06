#![deny(unsafe_code)]

//! Chiplet Core Harvesting, Market SKU Binning Strategy & Wafer Economics.
//!
//! Models multi-core modular architectures with redundant unit harvest tiers:
//! - Uncore / System Fabric critical area (fatal defect scraps entire die)
//! - Core array defect allocation (0 defects -> 16-Core Flagship, 1-2 defects -> 12-Core, 3-4 -> 8-Core)
//! - Wafer manufacturing cost, test/packaging costs, gross revenue, and profit margins.

/// Market SKU classification tier for harvested multi-core dies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HarvestSkuTier {
    /// 16 fully functional cores, zero defects.
    Flagship16Core,
    /// 12-14 functional cores (1-2 defective cores disabled).
    Harvested12Core,
    /// 8-10 functional cores (3-4 defective cores disabled).
    Salvage8Core,
    /// Uncore failure or > 4 defective cores. Non-functional scrap.
    Scrap,
}

impl HarvestSkuTier {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Flagship16Core => "16-Core Flagship",
            Self::Harvested12Core => "12-Core Harvested",
            Self::Salvage8Core => "8-Core Salvage",
            Self::Scrap => "Scrap / Reject",
        }
    }

    pub fn is_marketable(&self) -> bool {
        !matches!(self, Self::Scrap)
    }
}

/// Architectural configuration of multi-core die for defect harvesting.
#[derive(Debug, Clone)]
pub struct DieArchitectureParams {
    /// Total number of compute cores on die (typically 16).
    pub total_cores: usize,
    /// Fraction of die area occupied by the uncore / I/O / power management (e.g. 0.25 = 25%).
    pub uncore_area_fraction: f64,
    /// Selling price of Tier 1 Flagship SKU in USD.
    pub price_tier1_flagship_usd: f64,
    /// Selling price of Tier 2 Harvested SKU in USD.
    pub price_tier2_harvested_usd: f64,
    /// Selling price of Tier 3 Salvage SKU in USD.
    pub price_tier3_salvage_usd: f64,
}

impl Default for DieArchitectureParams {
    fn default() -> Self {
        Self {
            total_cores: 16,
            uncore_area_fraction: 0.25,
            price_tier1_flagship_usd: 850.0,
            price_tier2_harvested_usd: 550.0,
            price_tier3_salvage_usd: 320.0,
        }
    }
}

/// Wafer fabrication and downstream packaging financial parameters.
#[derive(Debug, Clone)]
pub struct WaferEconomicsParams {
    /// Leading-edge 300mm wafer fabrication cost in USD (e.g. $16,500).
    pub wafer_fab_cost_usd: f64,
    /// Wafer probe sort/test cost per gross die in USD.
    pub probe_test_cost_per_die_usd: f64,
    /// Advanced packaging and final assembly test cost per marketable good die in USD.
    pub packaging_cost_per_good_die_usd: f64,
}

impl Default for WaferEconomicsParams {
    fn default() -> Self {
        Self {
            wafer_fab_cost_usd: 16500.0,
            probe_test_cost_per_die_usd: 12.0,
            packaging_cost_per_good_die_usd: 38.0,
        }
    }
}

/// Harvest evaluation for a single die based on defect count and spatial allocation.
#[derive(Debug, Clone)]
pub struct DieHarvestStatus {
    pub uncore_defects: u32,
    pub core_defects: u32,
    pub total_defects: u32,
    pub sku_tier: HarvestSkuTier,
    pub realized_value_usd: f64,
}

/// Evaluates SKU binning tier given defect allocation across uncore and core array.
pub fn classify_die_harvest(
    uncore_defects: u32,
    core_defects: u32,
    arch: &DieArchitectureParams,
) -> DieHarvestStatus {
    let total_defects = uncore_defects + core_defects;

    let (sku_tier, value) = if uncore_defects > 0 {
        // Critical uncore defect destroys clock/power/bus; cannot be salvaged
        (HarvestSkuTier::Scrap, 0.0)
    } else {
        match core_defects {
            0 => (HarvestSkuTier::Flagship16Core, arch.price_tier1_flagship_usd),
            1..=2 => (HarvestSkuTier::Harvested12Core, arch.price_tier2_harvested_usd),
            3..=4 => (HarvestSkuTier::Salvage8Core, arch.price_tier3_salvage_usd),
            _ => (HarvestSkuTier::Scrap, 0.0),
        }
    };

    DieHarvestStatus {
        uncore_defects,
        core_defects,
        total_defects,
        sku_tier,
        realized_value_usd: value,
    }
}

/// Comprehensive wafer financial and yield economics breakdown report.
#[derive(Debug, Clone)]
pub struct WaferEconomicsReport {
    pub gross_dpw: usize,
    pub count_tier1_flagship: usize,
    pub count_tier2_harvested: usize,
    pub count_tier3_salvage: usize,
    pub count_scrap: usize,
    pub total_good_dies: usize,
    pub functional_yield_pct: f64,
    pub unharvested_yield_pct: f64,
    pub gross_revenue_usd: f64,
    pub unharvested_revenue_usd: f64,
    pub harvesting_revenue_uplift_pct: f64,
    pub total_mfg_cost_usd: f64,
    pub gross_profit_usd: f64,
    pub gross_margin_pct: f64,
    pub cost_per_good_die_usd: f64,
}

/// Computes wafer-level economic aggregation from die harvest outcomes.
pub fn compute_wafer_economics(
    die_statuses: &[DieHarvestStatus],
    arch: &DieArchitectureParams,
    econ: &WaferEconomicsParams,
) -> WaferEconomicsReport {
    let gross_dpw = die_statuses.len();
    let mut count_tier1 = 0;
    let mut count_tier2 = 0;
    let mut count_tier3 = 0;
    let mut count_scrap = 0;
    let mut gross_revenue = 0.0;

    for die in die_statuses {
        match die.sku_tier {
            HarvestSkuTier::Flagship16Core => {
                count_tier1 += 1;
                gross_revenue += arch.price_tier1_flagship_usd;
            }
            HarvestSkuTier::Harvested12Core => {
                count_tier2 += 1;
                gross_revenue += arch.price_tier2_harvested_usd;
            }
            HarvestSkuTier::Salvage8Core => {
                count_tier3 += 1;
                gross_revenue += arch.price_tier3_salvage_usd;
            }
            HarvestSkuTier::Scrap => {
                count_scrap += 1;
            }
        }
    }

    let total_good_dies = count_tier1 + count_tier2 + count_tier3;
    let functional_yield_pct = if gross_dpw > 0 {
        (total_good_dies as f64 / gross_dpw as f64) * 100.0
    } else {
        0.0
    };

    let unharvested_yield_pct = if gross_dpw > 0 {
        (count_tier1 as f64 / gross_dpw as f64) * 100.0
    } else {
        0.0
    };

    let unharvested_revenue = count_tier1 as f64 * arch.price_tier1_flagship_usd;
    let harvesting_revenue_uplift_pct = if unharvested_revenue > 0.0 {
        ((gross_revenue - unharvested_revenue) / unharvested_revenue) * 100.0
    } else {
        0.0
    };

    let total_mfg_cost = econ.wafer_fab_cost_usd
        + (gross_dpw as f64 * econ.probe_test_cost_per_die_usd)
        + (total_good_dies as f64 * econ.packaging_cost_per_good_die_usd);

    let gross_profit = gross_revenue - total_mfg_cost;
    let gross_margin_pct = if gross_revenue > 0.0 {
        (gross_profit / gross_revenue) * 100.0
    } else {
        0.0
    };

    let cost_per_good_die_usd = if total_good_dies > 0 {
        total_mfg_cost / total_good_dies as f64
    } else {
        0.0
    };

    WaferEconomicsReport {
        gross_dpw,
        count_tier1_flagship: count_tier1,
        count_tier2_harvested: count_tier2,
        count_tier3_salvage: count_tier3,
        count_scrap,
        total_good_dies,
        functional_yield_pct,
        unharvested_yield_pct,
        gross_revenue_usd: gross_revenue,
        unharvested_revenue_usd: unharvested_revenue,
        harvesting_revenue_uplift_pct,
        total_mfg_cost_usd: total_mfg_cost,
        gross_profit_usd: gross_profit,
        gross_margin_pct,
        cost_per_good_die_usd,
    }
}
