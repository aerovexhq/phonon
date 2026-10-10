#![deny(unsafe_code)]

//! Multi-Supplier Turnkey and Consigned PCBA Assembly Cost Estimator & Automated Panelizer.
//!
//! Models bare PCB fabrication, automated panelization arrays, SMT component placement fees,
//! stencils, and quality inspection across commercial PCB manufacturing tiers.

/// Surface finish option for printed circuit board manufacturing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SurfaceFinish {
    HaslLeadFree,
    EnigElectrolessNickelImmersionGold,
    OspOrganicSolderabilityPreservative,
    ImmersionSilver,
}

impl SurfaceFinish {
    pub fn name(&self) -> &'static str {
        match self {
            Self::HaslLeadFree => "HASL (Lead-Free)",
            Self::EnigElectrolessNickelImmersionGold => "ENIG (Gold)",
            Self::OspOrganicSolderabilityPreservative => "OSP",
            Self::ImmersionSilver => "Immersion Silver",
        }
    }

    pub fn cost_multiplier(&self) -> f64 {
        match self {
            Self::HaslLeadFree => 1.0,
            Self::EnigElectrolessNickelImmersionGold => 1.25,
            Self::OspOrganicSolderabilityPreservative => 0.95,
            Self::ImmersionSilver => 1.15,
        }
    }
}

/// SMT assembly sourcing mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AssemblySourcingMode {
    TurnkeyFull,       // Manufacturer sources all components
    ConsignedCustomer, // Customer ships all components to manufacturer
    HybridCombo,       // Manufacturer sources standard passives, customer supplies custom ICs
}

impl AssemblySourcingMode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::TurnkeyFull => "Full Turnkey (Manufacturer Procures All)",
            Self::ConsignedCustomer => "Consigned (Customer Supplies Components)",
            Self::HybridCombo => "Hybrid Combo (Shared Sourcing)",
        }
    }
}

/// Automated PCB panelization arrangement for mass SMT pick-and-place lines.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PanelizationSpec {
    pub single_board_width_mm: f64,
    pub single_board_length_mm: f64,
    pub panel_rows: usize,
    pub panel_cols: usize,
    pub edge_rail_margin_mm: f64,
    pub spacing_between_boards_mm: f64,
    pub panel_width_mm: f64,
    pub panel_length_mm: f64,
    pub boards_per_panel: usize,
    /// Percentage of total panel surface utilized by functional boards.
    pub panel_utilization_efficiency: f64,
}

impl PanelizationSpec {
    /// Computes optimal panelization layout for given board dimensions.
    pub fn compute(board_w_mm: f64, board_l_mm: f64, target_max_dim_mm: f64) -> Self {
        let rail = 5.0; // 5mm edge rails for conveyor handling
        let spacing = 2.0; // 2mm router/V-score clearance

        let usable_w = target_max_dim_mm - 2.0 * rail;
        let usable_l = target_max_dim_mm - 2.0 * rail;

        let cols = ((usable_w + spacing) / (board_w_mm + spacing)).floor().max(1.0) as usize;
        let rows = ((usable_l + spacing) / (board_l_mm + spacing)).floor().max(1.0) as usize;

        let total_w = 2.0 * rail + (cols as f64) * board_w_mm + ((cols - 1) as f64) * spacing;
        let total_l = 2.0 * rail + (rows as f64) * board_l_mm + ((rows - 1) as f64) * spacing;

        let boards = rows * cols;
        let board_area = (boards as f64) * board_w_mm * board_l_mm;
        let panel_area = total_w * total_l;
        let eff = if panel_area > 0.0 { board_area / panel_area } else { 0.0 };

        Self {
            single_board_width_mm: board_w_mm,
            single_board_length_mm: board_l_mm,
            panel_rows: rows,
            panel_cols: cols,
            edge_rail_margin_mm: rail,
            spacing_between_boards_mm: spacing,
            panel_width_mm: total_w,
            panel_length_mm: total_l,
            boards_per_panel: boards,
            panel_utilization_efficiency: eff,
        }
    }
}

/// Comprehensive industrial PCBA quotation breakdown at a given production volume.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PcbaVolumeQuote {
    pub batch_volume_units: usize,
    pub bare_pcb_unit_cost: f64,
    pub bare_pcb_total_cost: f64,
    pub smt_setup_total_cost: f64,
    pub smt_placement_unit_cost: f64,
    pub laser_stencil_total_cost: f64,
    pub aoi_inspection_unit_cost: f64,
    pub component_bom_unit_cost: f64,
    pub total_pcba_unit_cost: f64,
    pub total_batch_cost: f64,
}

/// Master PCBA Quoting and Assembly Engine.
#[derive(Debug, Clone)]
pub struct PcbaQuotingEngine {
    pub board_width_mm: f64,
    pub board_length_mm: f64,
    pub layer_count: usize,
    pub surface_finish: SurfaceFinish,
    pub smt_joints_count: usize,
    pub through_hole_pins_count: usize,
    pub bga_qfn_chip_count: usize,
    pub sourcing_mode: AssemblySourcingMode,
    pub panelization: PanelizationSpec,
}

impl Default for PcbaQuotingEngine {
    fn default() -> Self {
        Self::new(50.0, 50.0, 2, 80, 4)
    }
}

impl PcbaQuotingEngine {
    pub fn new(
        board_w_mm: f64,
        board_l_mm: f64,
        layer_count: usize,
        smt_joints: usize,
        th_pins: usize,
    ) -> Self {
        let panelization = PanelizationSpec::compute(board_w_mm, board_l_mm, 200.0);
        Self {
            board_width_mm: board_w_mm,
            board_length_mm: board_l_mm,
            layer_count,
            surface_finish: SurfaceFinish::EnigElectrolessNickelImmersionGold,
            smt_joints_count: smt_joints,
            through_hole_pins_count: th_pins,
            bga_qfn_chip_count: 1,
            sourcing_mode: AssemblySourcingMode::TurnkeyFull,
            panelization,
        }
    }

    /// Evaluates manufacturing and assembly quotation at a specified volume N.
    pub fn quote_at_volume(&self, volume: usize, raw_bom_unit_cost: f64) -> PcbaVolumeQuote {
        let vol = volume.max(1);

        // 1. Bare PCB Fabrication Cost
        // Base tooling setup amortized: ~$30 for 2-layer, $60 for 4-layer
        let pcb_tooling = match self.layer_count {
            1 | 2 => 30.0,
            3 | 4 => 60.0,
            5 | 6 => 120.0,
            _ => 200.0,
        };
        let pcb_area_sqm = (self.board_width_mm * self.board_length_mm) / 1_000_000.0;
        let base_rate_sqm = match self.layer_count {
            1 | 2 => 70.0,
            3 | 4 => 130.0,
            5 | 6 => 220.0,
            _ => 350.0,
        };
        let pcb_base_cost = (pcb_area_sqm * base_rate_sqm).max(0.30);
        let pcb_material_unit = pcb_base_cost * self.surface_finish.cost_multiplier();
        let bare_pcb_unit = (pcb_tooling / (vol as f64)) + pcb_material_unit;
        let bare_pcb_total = bare_pcb_unit * (vol as f64);

        // 2. SMT Assembly Fees
        // Stencil cost: $15 standard frameless, $35 framed
        let stencil_cost = if vol > 50 { 35.0 } else { 15.0 };
        // SMT Line Setup: ~$40
        let smt_setup = if vol > 500 { 80.0 } else { 40.0 };
        // SMT placement cost per joint: $0.005 down to $0.0015 at volume
        let joint_rate = if vol >= 10_000 {
            0.0015
        } else if vol >= 1_000 {
            0.0025
        } else if vol >= 100 {
            0.0040
        } else {
            0.0065
        };
        let smt_placement_unit = (self.smt_joints_count as f64) * joint_rate
            + (self.through_hole_pins_count as f64) * 0.035;

        // 3. Automated Inspection (AOI & X-Ray for BGA/QFN)
        let aoi_unit = if self.bga_qfn_chip_count > 0 {
            0.25 + (self.bga_qfn_chip_count as f64) * 0.15
        } else {
            0.15
        };

        // 4. Component BOM cost accounting for sourcing mode
        let effective_bom_cost = match self.sourcing_mode {
            AssemblySourcingMode::TurnkeyFull => raw_bom_unit_cost,
            AssemblySourcingMode::ConsignedCustomer => 0.0,
            AssemblySourcingMode::HybridCombo => raw_bom_unit_cost * 0.40,
        };

        let fixed_amortized = (stencil_cost + smt_setup) / (vol as f64);
        let total_unit = bare_pcb_unit + smt_placement_unit + aoi_unit + fixed_amortized + effective_bom_cost;

        PcbaVolumeQuote {
            batch_volume_units: vol,
            bare_pcb_unit_cost: bare_pcb_unit,
            bare_pcb_total_cost: bare_pcb_total,
            smt_setup_total_cost: smt_setup,
            smt_placement_unit_cost: smt_placement_unit,
            laser_stencil_total_cost: stencil_cost,
            aoi_inspection_unit_cost: aoi_unit,
            component_bom_unit_cost: effective_bom_cost,
            total_pcba_unit_cost: total_unit,
            total_batch_cost: total_unit * (vol as f64),
        }
    }

    /// Evaluates multi-tier volume trajectory (1, 10, 50, 100, 500, 1000, 5000, 10000).
    pub fn volume_curve(&self, raw_bom_unit_cost: f64) -> Vec<PcbaVolumeQuote> {
        let volumes = [1, 10, 50, 100, 500, 1_000, 5_000, 10_000];
        volumes.iter().map(|&v| self.quote_at_volume(v, raw_bom_unit_cost)).collect()
    }
}
