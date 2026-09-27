//! Macro-architectural floorplan and functional block layout for CPU synthesis.
//!
//! Provides geometric 2D placement, power density modeling, and floorplan assembly
//! for a 5-stage pipelined RISC-V processor with integrated L1 cache, FIVR, and clock spine.

use crate::hetero::material_allocation::{
    BlockAllocationMap, HeteroMaterialType, ProcessorBlockType,
};

/// A functional macro block placed in the processor die floorplan.
#[derive(Debug, Clone)]
pub struct CpuMacroBlock {
    /// Unique block index
    pub id: usize,
    /// Block name
    pub name: String,
    /// Functional type
    pub block_type: ProcessorBlockType,
    /// Bottom-left corner X coordinate [um]
    pub x_um: f64,
    /// Bottom-left corner Y coordinate [um]
    pub y_um: f64,
    /// Block width [um]
    pub width_um: f64,
    /// Block height [um]
    pub height_um: f64,
    /// Assigned semiconductor material
    pub material: HeteroMaterialType,
    /// Active transistor / device count
    pub transistor_count: usize,
    /// Aggregate gate capacitance [fF]
    pub gate_capacitance_ff: f64,
    /// Switching activity factor (0.0 to 1.0)
    pub activity_factor: f64,
}

impl CpuMacroBlock {
    /// Silicon area in square micrometers [um^2].
    pub fn area_um2(&self) -> f64 {
        self.width_um * self.height_um
    }

    /// Dynamic switching power dissipation [mW] at given supply voltage and frequency.
    pub fn dynamic_power_mw(&self, v_dd: f64, freq_ghz: f64) -> f64 {
        // P_dyn = 0.5 * alpha * C_total * Vdd^2 * f
        // C_total in fF (10^-15 F), f in GHz (10^9 Hz)
        // 0.5 * 10^-15 * 10^9 = 0.5 * 10^-6 W = 0.5 * 10^-3 mW
        let c_total_ff = self.gate_capacitance_ff;
        0.5 * self.activity_factor * c_total_ff * (v_dd * v_dd) * freq_ghz * 1e-3
    }

    /// Static subthreshold leakage power dissipation [mW] at given supply voltage.
    pub fn static_power_mw(&self, v_dd: f64) -> f64 {
        let props = self.material.properties();
        // Total gate width: assume ~100 nm gate width per transistor
        let total_width_um = (self.transistor_count as f64) * 0.10;
        let leakage_current_a = total_width_um * props.subthreshold_leakage_a_per_um;
        // P_stat = I_leak * Vdd [W] -> * 1e3 for mW
        leakage_current_a * v_dd * 1.0e3
    }

    /// Total power dissipation (dynamic + static) [mW].
    pub fn total_power_mw(&self, v_dd: f64, freq_ghz: f64) -> f64 {
        self.dynamic_power_mw(v_dd, freq_ghz) + self.static_power_mw(v_dd)
    }

    /// Local power density in Watts per square centimeter [W / cm^2].
    pub fn power_density_w_per_cm2(&self, v_dd: f64, freq_ghz: f64) -> f64 {
        let power_w = self.total_power_mw(v_dd, freq_ghz) * 1e-3;
        let area_cm2 = self.area_um2() * 1.0e-8; // 1 um^2 = 10^-8 cm^2
        power_w / area_cm2.max(1e-12)
    }
}

/// Floorplan representation of a whole processor core.
#[derive(Debug, Clone)]
pub struct ProcessorFloorplan {
    /// Total die width [um]
    pub die_width_um: f64,
    /// Total die height [um]
    pub die_height_um: f64,
    /// List of placed functional macro blocks
    pub blocks: Vec<CpuMacroBlock>,
}

impl ProcessorFloorplan {
    /// Total die footprint area [um^2].
    pub fn total_area_um2(&self) -> f64 {
        self.die_width_um * self.die_height_um
    }

    /// Total die footprint area in square millimeters [mm^2].
    pub fn total_area_mm2(&self) -> f64 {
        self.total_area_um2() * 1.0e-6
    }

    /// Total processor dynamic power dissipation [mW].
    pub fn total_dynamic_power_mw(&self, v_dd: f64, freq_ghz: f64) -> f64 {
        self.blocks
            .iter()
            .map(|b| b.dynamic_power_mw(v_dd, freq_ghz))
            .sum()
    }

    /// Total processor static leakage power dissipation [mW].
    pub fn total_static_power_mw(&self, v_dd: f64) -> f64 {
        self.blocks.iter().map(|b| b.static_power_mw(v_dd)).sum()
    }

    /// Total processor power dissipation [mW].
    pub fn total_power_mw(&self, v_dd: f64, freq_ghz: f64) -> f64 {
        self.total_dynamic_power_mw(v_dd, freq_ghz) + self.total_static_power_mw(v_dd)
    }

    /// Finds the block with the highest power density (thermal hotspot origin).
    pub fn find_peak_hotspot_block(&self, v_dd: f64, freq_ghz: f64) -> &CpuMacroBlock {
        self.blocks
            .iter()
            .max_by(|a, b| {
                a.power_density_w_per_cm2(v_dd, freq_ghz)
                    .partial_cmp(&b.power_density_w_per_cm2(v_dd, freq_ghz))
                    .unwrap()
            })
            .expect("Floorplan must contain at least one block")
    }

    /// Finds a macro block by its functional type.
    pub fn get_block(&self, block_type: ProcessorBlockType) -> Option<&CpuMacroBlock> {
        self.blocks.iter().find(|b| b.block_type == block_type)
    }
}

/// Builder for assembling realistic pipelined RISC-V processor floorplans.
pub struct RiscVFloorplanBuilder;

impl RiscVFloorplanBuilder {
    /// Builds a canonical 5-stage RISC-V processor floorplan with specified material allocation map.
    pub fn build(allocation: &BlockAllocationMap) -> ProcessorFloorplan {
        let blocks = vec![
            // 1. Instruction Fetch (IF): Top-left
            CpuMacroBlock {
                id: 0,
                name: "Instruction Fetch (IF)".to_string(),
                block_type: ProcessorBlockType::InstructionFetch,
                x_um: 0.0,
                y_um: 120.0,
                width_um: 110.0,
                height_um: 80.0,
                material: allocation.get(ProcessorBlockType::InstructionFetch),
                transistor_count: 250_000,
                gate_capacitance_ff: 62.5,
                activity_factor: 0.40,
            },
            // 2. Instruction Decode (ID): Top-center
            CpuMacroBlock {
                id: 1,
                name: "Instruction Decode (ID)".to_string(),
                block_type: ProcessorBlockType::InstructionDecode,
                x_um: 110.0,
                y_um: 120.0,
                width_um: 90.0,
                height_um: 80.0,
                material: allocation.get(ProcessorBlockType::InstructionDecode),
                transistor_count: 180_000,
                gate_capacitance_ff: 45.0,
                activity_factor: 0.35,
            },
            // 3. Execution ALU (EX): Center-left (Critical frequency path & hotspot)
            CpuMacroBlock {
                id: 2,
                name: "Execution ALU (EX)".to_string(),
                block_type: ProcessorBlockType::ExecutionAlu,
                x_um: 0.0,
                y_um: 0.0,
                width_um: 120.0,
                height_um: 120.0,
                material: allocation.get(ProcessorBlockType::ExecutionAlu),
                transistor_count: 450_000,
                gate_capacitance_ff: 112.5,
                activity_factor: 0.70, // High activity in critical arithmetic
            },
            // 4. Memory Access (MEM): Center
            CpuMacroBlock {
                id: 3,
                name: "Memory Access (MEM)".to_string(),
                block_type: ProcessorBlockType::MemoryAccess,
                x_um: 120.0,
                y_um: 0.0,
                width_um: 80.0,
                height_um: 120.0,
                material: allocation.get(ProcessorBlockType::MemoryAccess),
                transistor_count: 160_000,
                gate_capacitance_ff: 40.0,
                activity_factor: 0.45,
            },
            // 5. Write-Back & Commit (WB): Top right of datapath
            CpuMacroBlock {
                id: 4,
                name: "Write-Back (WB)".to_string(),
                block_type: ProcessorBlockType::WriteBack,
                x_um: 200.0,
                y_um: 120.0,
                width_um: 70.0,
                height_um: 80.0,
                material: allocation.get(ProcessorBlockType::WriteBack),
                transistor_count: 110_000,
                gate_capacitance_ff: 27.5,
                activity_factor: 0.30,
            },
            // 6. L1 Data & Instruction Cache (L1-D/I): Right half of die (leakage sensitive)
            CpuMacroBlock {
                id: 5,
                name: "L1 Cache Array (L1-D/I)".to_string(),
                block_type: ProcessorBlockType::L1Cache,
                x_um: 270.0,
                y_um: 0.0,
                width_um: 230.0,
                height_um: 200.0,
                material: allocation.get(ProcessorBlockType::L1Cache),
                transistor_count: 2_800_000,
                gate_capacitance_ff: 700.0,
                activity_factor: 0.15, // Low switching activity, dominated by standby leakage
            },
            // 7. Power Delivery & On-Chip FIVR: Bottom strip across die
            CpuMacroBlock {
                id: 6,
                name: "Power Delivery / FIVR".to_string(),
                block_type: ProcessorBlockType::PowerDeliveryFivr,
                x_um: 0.0,
                y_um: 200.0,
                width_um: 500.0,
                height_um: 50.0,
                material: allocation.get(ProcessorBlockType::PowerDeliveryFivr),
                transistor_count: 120_000,
                gate_capacitance_ff: 300.0,
                activity_factor: 0.80, // High power switching
            },
            // 8. Clock Distribution Spine: Central vertical corridor
            CpuMacroBlock {
                id: 7,
                name: "Clock Distribution Spine".to_string(),
                block_type: ProcessorBlockType::ClockDistribution,
                x_um: 200.0,
                y_um: 0.0,
                width_um: 70.0,
                height_um: 120.0,
                material: allocation.get(ProcessorBlockType::ClockDistribution),
                transistor_count: 80_000,
                gate_capacitance_ff: 150.0,
                activity_factor: 1.0, // Continuous clock toggling
            },
        ];

        ProcessorFloorplan {
            die_width_um: 500.0,
            die_height_um: 250.0,
            blocks,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_riscv_floorplan_dimensions_and_power() {
        let alloc = BlockAllocationMap::uniform_silicon();
        let floorplan = RiscVFloorplanBuilder::build(&alloc);

        assert_eq!(floorplan.blocks.len(), 8);
        assert!((floorplan.die_width_um - 500.0).abs() < 1e-6);
        assert!((floorplan.die_height_um - 250.0).abs() < 1e-6);
        assert!((floorplan.total_area_mm2() - 0.125).abs() < 1e-6);

        let p_dyn = floorplan.total_dynamic_power_mw(0.8, 3.0);
        let p_stat = floorplan.total_static_power_mw(0.8);

        assert!(p_dyn > 0.0);
        assert!(p_stat > 0.0);

        let hotspot = floorplan.find_peak_hotspot_block(0.8, 3.0);
        assert!(
            hotspot.block_type == ProcessorBlockType::ClockDistribution
                || hotspot.block_type == ProcessorBlockType::PowerDeliveryFivr
                || hotspot.block_type == ProcessorBlockType::ExecutionAlu
        );
    }

    #[test]
    fn test_igzo_cache_leakage_elimination() {
        let alloc_si = BlockAllocationMap::uniform_silicon();
        let alloc_hetero = BlockAllocationMap::synthesized_heterogeneous();

        let fp_si = RiscVFloorplanBuilder::build(&alloc_si);
        let fp_hetero = RiscVFloorplanBuilder::build(&alloc_hetero);

        let cache_si = fp_si.get_block(ProcessorBlockType::L1Cache).unwrap();
        let cache_igzo = fp_hetero.get_block(ProcessorBlockType::L1Cache).unwrap();

        let leak_si = cache_si.static_power_mw(0.8);
        let leak_igzo = cache_igzo.static_power_mw(0.8);

        // IGZO cache static leakage must be > 10^6x lower than silicon
        assert!(leak_si / leak_igzo > 1.0e6);
    }
}
