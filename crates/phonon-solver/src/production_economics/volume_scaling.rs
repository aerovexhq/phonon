#![deny(unsafe_code)]

//! Automated Production Scaling & Volume Breakpoints Modeling Engine.
//!
//! Evaluates economies of scale across batch volume breakpoints (N = 1 to 100,000 units),
//! incorporating component reel discounts, PCB panelization amortizations, SMT stencil setup,
//! and automated optical inspection (AOI) to project commercial viability.

/// Complete cost and margin projection for a discrete production volume tier.
#[derive(Debug, Clone, PartialEq)]
pub struct VolumeBreakpoint {
    /// Batch production volume (units).
    pub quantity: u32,
    /// Industry production tier designation.
    pub tier_name: String,
    /// Component BOM cost per unit in USD.
    pub component_bom_unit_cost: f64,
    /// Bare PCB fabrication cost per unit in USD.
    pub pcb_fab_unit_cost: f64,
    /// SMT pick-and-place assembly cost per unit in USD.
    pub smt_assembly_unit_cost: f64,
    /// Testing, flashing & QA cost per unit in USD.
    pub testing_qa_unit_cost: f64,
    /// Total manufacturing cost per unit (COGS) in USD.
    pub total_unit_cost: f64,
    /// Total upfront batch manufacturing capital in USD.
    pub total_batch_cost: f64,
    /// Gross profit margin percentage at specified target MSRP.
    pub gross_margin_pct: f64,
}

/// Manufacturing and fabrication process parameter model.
#[derive(Debug, Clone, PartialEq)]
pub struct ProductionVolumeModel {
    /// Fixed PCB tooling and photolithography setup fee (USD).
    pub pcb_setup_fee: f64,
    /// Fixed SMT solder stencil, feeder setup, and calibration fee (USD).
    pub smt_setup_fee: f64,
    /// Physical printed circuit board surface area in square centimeters.
    pub board_area_cm2: f64,
    /// Number of conductive layers (e.g. 2, 4, 6 layers).
    pub layer_count: usize,
    /// Total SMT component solder joints / pads on board.
    pub solder_joints_count: usize,
    /// Assembly cost per solder joint placement in volume (USD).
    pub placement_cost_per_joint: f64,
    /// Automated testing, firmware flash, and functional QA cost per unit (USD).
    pub testing_cost_per_unit: f64,
    /// Target retail / commercial selling price (MSRP) in USD.
    pub target_msrp: f64,
}

impl Default for ProductionVolumeModel {
    fn default() -> Self {
        Self {
            pcb_setup_fee: 50.0,
            smt_setup_fee: 120.0,
            board_area_cm2: 28.0,
            layer_count: 4,
            solder_joints_count: 72,
            placement_cost_per_joint: 0.0075,
            testing_cost_per_unit: 0.85,
            target_msrp: 49.99,
        }
    }
}

impl ProductionVolumeModel {
    /// Standard production run quantities analyzed.
    pub const STANDARD_BREAKPOINTS: [u32; 6] = [1, 10, 100, 1_000, 10_000, 100_000];

    /// Returns human-readable industry tier designation.
    pub fn tier_name_for_qty(qty: u32) -> &'static str {
        match qty {
            1 => "1 (Prototype)",
            10 => "10 (Engineering Pilot)",
            100 => "100 (Small Batch)",
            1_000 => "1k (Mid-Volume)",
            10_000 => "10k (Mass Production)",
            _ => "100k (Wafer / Full Reel)",
        }
    }

    /// Evaluates volume discount multiplier for component BOM according to distributor reel breaks.
    pub fn component_discount_factor(qty: u32) -> f64 {
        if qty <= 1 {
            1.00
        } else if qty <= 10 {
            0.85
        } else if qty <= 100 {
            0.68
        } else if qty <= 1_000 {
            0.48
        } else if qty <= 10_000 {
            0.34
        } else {
            0.24
        }
    }

    /// Evaluates volume efficiency scaling factor for PCB laminate panelization.
    pub fn pcb_efficiency_factor(qty: u32) -> f64 {
        if qty <= 1 {
            1.00
        } else if qty <= 10 {
            0.75
        } else if qty <= 100 {
            0.55
        } else if qty <= 1_000 {
            0.38
        } else if qty <= 10_000 {
            0.26
        } else {
            0.18
        }
    }

    /// Evaluates SMT pick-and-place assembly machine speedup factor in large panels.
    pub fn smt_efficiency_factor(qty: u32) -> f64 {
        if qty <= 1 {
            1.00
        } else if qty <= 10 {
            0.80
        } else if qty <= 100 {
            0.60
        } else if qty <= 1_000 {
            0.42
        } else if qty <= 10_000 {
            0.28
        } else {
            0.19
        }
    }

    /// Generates full cost breakdown across all standard production volume breakpoints.
    pub fn calculate_breakpoints(&self, prototype_bom_cost: f64) -> Vec<VolumeBreakpoint> {
        let mut results = Vec::with_capacity(Self::STANDARD_BREAKPOINTS.len());

        let layer_multiplier = match self.layer_count {
            1 | 2 => 1.0,
            4 => 1.45,
            6 => 2.10,
            _ => 2.80,
        };

        let base_raw_pcb_cost = (self.board_area_cm2 * 0.045) * layer_multiplier;
        let base_smt_cost = (self.solder_joints_count as f64) * self.placement_cost_per_joint;

        for &qty in &Self::STANDARD_BREAKPOINTS {
            let q_f64 = qty as f64;

            // 1. Component BOM with volume discounting
            let bom_unit = prototype_bom_cost * Self::component_discount_factor(qty);

            // 2. PCB Fabrication unit cost
            let pcb_unit = (self.pcb_setup_fee / q_f64) + (base_raw_pcb_cost * Self::pcb_efficiency_factor(qty));

            // 3. SMT Assembly unit cost
            let smt_unit = (self.smt_setup_fee / q_f64) + (base_smt_cost * Self::smt_efficiency_factor(qty));

            // 4. Testing, firmware programming & QA
            let qa_unit = self.testing_cost_per_unit * if qty >= 1_000 { 0.45 } else { 1.0 };

            // Total COGS unit cost
            let total_unit = bom_unit + pcb_unit + smt_unit + qa_unit;
            let total_batch = total_unit * q_f64;

            // Gross Margin % = (1 - COGS / MSRP) * 100%
            let margin_pct = if self.target_msrp > 0.0 {
                ((1.0 - (total_unit / self.target_msrp)) * 100.0).clamp(-100.0, 99.0)
            } else {
                0.0
            };

            results.push(VolumeBreakpoint {
                quantity: qty,
                tier_name: Self::tier_name_for_qty(qty).to_string(),
                component_bom_unit_cost: (bom_unit * 1000.0).round() / 1000.0,
                pcb_fab_unit_cost: (pcb_unit * 1000.0).round() / 1000.0,
                smt_assembly_unit_cost: (smt_unit * 1000.0).round() / 1000.0,
                testing_qa_unit_cost: (qa_unit * 1000.0).round() / 1000.0,
                total_unit_cost: (total_unit * 1000.0).round() / 1000.0,
                total_batch_cost: (total_batch * 100.0).round() / 100.0,
                gross_margin_pct: (margin_pct * 10.0).round() / 10.0,
            });
        }

        results
    }

    /// Identifies the minimum production volume N required to achieve target gross margin %.
    pub fn breakeven_volume_for_margin(&self, prototype_bom_cost: f64, target_margin_pct: f64) -> Option<u32> {
        let breakpoints = self.calculate_breakpoints(prototype_bom_cost);
        for bp in breakpoints {
            if bp.gross_margin_pct >= target_margin_pct {
                return Some(bp.quantity);
            }
        }
        None
    }
}
