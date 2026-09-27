//! Spatially distributed heterogeneous semiconductor materials and device allocation database.
//!
//! Provides:
//! 1. `HeteroMaterialType`: Specialized semiconductor chemistries and physical mechanisms
//!    (Silicon GAA, Strained Ge pMOS, InGaAs nMOS, GaN, 4H-SiC, BEOL IGZO, CNT bundles, Topological Insulators).
//! 2. `HeteroMaterialProperties`: Comprehensive database of thermal, mechanical, electronic,
//!    and high-field transport constants.
//! 3. `ProcessorBlockType`: Canonical processor macro-structure functional blocks.
//! 4. `BlockAllocationMap`: Mapping linking functional blocks to specialized semiconductor chemistry.

/// Specialized semiconductor material types and interconnect technologies for heterogeneous CPU synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeteroMaterialType {
    /// Baseline 3nm Silicon Gate-All-Around (GAA) CMOS
    SiliconGaa,
    /// Strained Germanium (s-Ge) pMOS with high hole mobility and high injection velocity
    StrainedGePmos,
    /// Indium Gallium Arsenide (In0.7Ga0.3As) nMOS with high electron injection velocity
    InGaAsNmos,
    /// Wide-bandgap Gallium Nitride (GaN) high-electron-mobility transistor for power delivery
    WideBandgapGan,
    /// Silicon Carbide (4H-SiC) for high-voltage endurance and high thermal conductivity
    SiliconCarbide,
    /// Indium Gallium Zinc Oxide (IGZO) BEOL thin-film transistor for low-leakage caches
    IgzoBeol,
    /// Metallic Carbon Nanotube (CNT) bundles for low-RC clock distribution and buses
    CntBundleInterconnect,
    /// Bismuth Selenide (Bi2Se3) Topological Insulator with spin-momentum locked Dirac states
    TopologicalInsulator,
}

impl HeteroMaterialType {
    /// Human-readable identifier.
    pub fn name(&self) -> &'static str {
        match self {
            HeteroMaterialType::SiliconGaa => "Silicon GAA (3nm)",
            HeteroMaterialType::StrainedGePmos => "Strained Germanium pMOS",
            HeteroMaterialType::InGaAsNmos => "In0.7Ga0.3As nMOS",
            HeteroMaterialType::WideBandgapGan => "Wide-Bandgap GaN",
            HeteroMaterialType::SiliconCarbide => "4H-Silicon Carbide (SiC)",
            HeteroMaterialType::IgzoBeol => "BEOL IGZO Oxide TFT",
            HeteroMaterialType::CntBundleInterconnect => "Carbon Nanotube (CNT) Bundle",
            HeteroMaterialType::TopologicalInsulator => "Bi2Se3 Topological Insulator",
        }
    }

    /// Queries the physical, thermal, mechanical, and transport properties of this material.
    pub fn properties(&self) -> HeteroMaterialProperties {
        match self {
            HeteroMaterialType::SiliconGaa => HeteroMaterialProperties {
                thermal_conductivity_w_mk: 140.0,
                cte_ppm_per_k: 2.6,
                youngs_modulus_gpa: 160.0,
                poisson_ratio: 0.28,
                bandgap_ev: 1.12,
                critical_breakdown_field_mv_cm: 0.30,
                injection_velocity_cm_s: 1.2e7,
                low_field_mobility_cm2_vs: 450.0,
                subthreshold_leakage_a_per_um: 1.0e-10, // 0.1 nA/um
                interconnect_resistivity_ohm_m: 2.2e-8, // Standard Cu
                max_current_density_a_cm2: 2.0e6,       // Cu EM limit
                black_activation_energy_ev: 0.90,
            },
            HeteroMaterialType::StrainedGePmos => HeteroMaterialProperties {
                thermal_conductivity_w_mk: 60.0,
                cte_ppm_per_k: 5.8,
                youngs_modulus_gpa: 130.0,
                poisson_ratio: 0.26,
                bandgap_ev: 0.66,
                critical_breakdown_field_mv_cm: 0.20,
                injection_velocity_cm_s: 2.4e7, // 2x silicon for hole transport
                low_field_mobility_cm2_vs: 900.0,
                subthreshold_leakage_a_per_um: 5.0e-10,
                interconnect_resistivity_ohm_m: 2.2e-8,
                max_current_density_a_cm2: 2.0e6,
                black_activation_energy_ev: 0.90,
            },
            HeteroMaterialType::InGaAsNmos => HeteroMaterialProperties {
                thermal_conductivity_w_mk: 15.0,
                cte_ppm_per_k: 5.2,
                youngs_modulus_gpa: 80.0,
                poisson_ratio: 0.35,
                bandgap_ev: 0.75,
                critical_breakdown_field_mv_cm: 0.25,
                injection_velocity_cm_s: 3.2e7, // ~2.7x silicon electron injection
                low_field_mobility_cm2_vs: 8000.0,
                subthreshold_leakage_a_per_um: 8.0e-10,
                interconnect_resistivity_ohm_m: 2.2e-8,
                max_current_density_a_cm2: 2.0e6,
                black_activation_energy_ev: 0.90,
            },
            HeteroMaterialType::WideBandgapGan => HeteroMaterialProperties {
                thermal_conductivity_w_mk: 130.0,
                cte_ppm_per_k: 5.6,
                youngs_modulus_gpa: 290.0,
                poisson_ratio: 0.23,
                bandgap_ev: 3.40,
                critical_breakdown_field_mv_cm: 3.30, // >10x silicon
                injection_velocity_cm_s: 2.5e7,
                low_field_mobility_cm2_vs: 1500.0,
                subthreshold_leakage_a_per_um: 1.0e-11,
                interconnect_resistivity_ohm_m: 2.2e-8,
                max_current_density_a_cm2: 5.0e6,
                black_activation_energy_ev: 1.20,
            },
            HeteroMaterialType::SiliconCarbide => HeteroMaterialProperties {
                thermal_conductivity_w_mk: 370.0, // High thermal conductivity
                cte_ppm_per_k: 4.0,
                youngs_modulus_gpa: 450.0,
                poisson_ratio: 0.21,
                bandgap_ev: 3.26,
                critical_breakdown_field_mv_cm: 3.00,
                injection_velocity_cm_s: 2.0e7,
                low_field_mobility_cm2_vs: 900.0,
                subthreshold_leakage_a_per_um: 1.0e-12,
                interconnect_resistivity_ohm_m: 2.2e-8,
                max_current_density_a_cm2: 8.0e6,
                black_activation_energy_ev: 1.40,
            },
            HeteroMaterialType::IgzoBeol => HeteroMaterialProperties {
                thermal_conductivity_w_mk: 1.5,
                cte_ppm_per_k: 4.5,
                youngs_modulus_gpa: 120.0,
                poisson_ratio: 0.25,
                bandgap_ev: 3.10,
                critical_breakdown_field_mv_cm: 1.50,
                injection_velocity_cm_s: 0.6e7,
                low_field_mobility_cm2_vs: 15.0,
                subthreshold_leakage_a_per_um: 1.0e-19, // Ultra-low off-state leakage (< 10^-18 A/um)
                interconnect_resistivity_ohm_m: 2.2e-8,
                max_current_density_a_cm2: 1.0e6,
                black_activation_energy_ev: 0.85,
            },
            HeteroMaterialType::CntBundleInterconnect => HeteroMaterialProperties {
                thermal_conductivity_w_mk: 3000.0, // Ballistic along tube axis
                cte_ppm_per_k: -1.5,
                youngs_modulus_gpa: 1000.0,
                poisson_ratio: 0.16,
                bandgap_ev: 0.0, // Metallic
                critical_breakdown_field_mv_cm: 10.0,
                injection_velocity_cm_s: 8.0e7,
                low_field_mobility_cm2_vs: 100_000.0,
                subthreshold_leakage_a_per_um: 0.0,
                interconnect_resistivity_ohm_m: 1.0e-8, // Half copper resistivity
                max_current_density_a_cm2: 1.0e9,       // > 1000x copper EM tolerance
                black_activation_energy_ev: 2.80,       // Negligible electromigration
            },
            HeteroMaterialType::TopologicalInsulator => HeteroMaterialProperties {
                thermal_conductivity_w_mk: 2.0,
                cte_ppm_per_k: 8.0,
                youngs_modulus_gpa: 60.0,
                poisson_ratio: 0.30,
                bandgap_ev: 0.30, // Bulk bandgap with gapless surface states
                critical_breakdown_field_mv_cm: 0.50,
                injection_velocity_cm_s: 5.0e7,
                low_field_mobility_cm2_vs: 10_000.0,
                subthreshold_leakage_a_per_um: 1.0e-8,
                interconnect_resistivity_ohm_m: 1.5e-8,
                max_current_density_a_cm2: 5.0e8,
                black_activation_energy_ev: 2.50,
            },
        }
    }
}

/// Physical, transport, thermal, and mechanical properties of a semiconductor material.
#[derive(Debug, Clone, Copy)]
pub struct HeteroMaterialProperties {
    /// Thermal conductivity kappa [W / (m*K)]
    pub thermal_conductivity_w_mk: f64,
    /// Coefficient of Thermal Expansion (CTE) [ppm / K = 10^-6 / K]
    pub cte_ppm_per_k: f64,
    /// Young's Elastic Modulus [GPa]
    pub youngs_modulus_gpa: f64,
    /// Poisson's ratio nu
    pub poisson_ratio: f64,
    /// Electronic bandgap [eV]
    pub bandgap_ev: f64,
    /// Critical dielectric breakdown electric field [MV / cm]
    pub critical_breakdown_field_mv_cm: f64,
    /// Thermal injection carrier velocity [cm / s]
    pub injection_velocity_cm_s: f64,
    /// Low-field carrier mobility [cm^2 / (V*s)]
    pub low_field_mobility_cm2_vs: f64,
    /// Room-temperature off-state subthreshold leakage current [A / um]
    pub subthreshold_leakage_a_per_um: f64,
    /// Interconnect electrical resistivity [Ohm * m]
    pub interconnect_resistivity_ohm_m: f64,
    /// Maximum safe current density before electromigration failure [A / cm^2]
    pub max_current_density_a_cm2: f64,
    /// Electromigration activation energy Ea [eV] (Black's equation)
    pub black_activation_energy_ev: f64,
}

/// Functional processor block types in a pipelined CPU macro-structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProcessorBlockType {
    /// Instruction Fetch (IF): Program counter, branch predictor, instruction pre-decode
    InstructionFetch,
    /// Instruction Decode (ID): Micro-op decoder, register rename table, dependency matrix
    InstructionDecode,
    /// Execution ALU (EX): Critical integer arithmetic, fast multiplier, shifter
    ExecutionAlu,
    /// Memory Access (MEM): Load-store unit, data address generator, store buffer
    MemoryAccess,
    /// Write-Back (WB): Register commit and retirement logic
    WriteBack,
    /// On-chip L1 Cache: Data and instruction SRAM / BEOL arrays
    L1Cache,
    /// Power Delivery Network & FIVR: Integrated buck converter and low-dropout regulators
    PowerDeliveryFivr,
    /// Clock Distribution Spine: Global H-tree clock distribution network
    ClockDistribution,
}

impl ProcessorBlockType {
    /// Human-readable block name.
    pub fn name(&self) -> &'static str {
        match self {
            ProcessorBlockType::InstructionFetch => "Instruction Fetch (IF)",
            ProcessorBlockType::InstructionDecode => "Instruction Decode (ID)",
            ProcessorBlockType::ExecutionAlu => "Execution ALU (EX)",
            ProcessorBlockType::MemoryAccess => "Memory Access (MEM)",
            ProcessorBlockType::WriteBack => "Write-Back (WB)",
            ProcessorBlockType::L1Cache => "L1 Cache Array (L1-D/I)",
            ProcessorBlockType::PowerDeliveryFivr => "Power Delivery / FIVR",
            ProcessorBlockType::ClockDistribution => "Clock Distribution Spine",
        }
    }

    /// Primary sensitivity metric for this block.
    pub fn primary_sensitivity(&self) -> &'static str {
        match self {
            ProcessorBlockType::ExecutionAlu => "Critical Path Delay / Max Frequency",
            ProcessorBlockType::L1Cache => "Static Standby Leakage Power",
            ProcessorBlockType::PowerDeliveryFivr => "Breakdown Field & Power Efficiency",
            ProcessorBlockType::ClockDistribution => "RC Propagation Delay & Electromigration",
            _ => "Balanced Logic Throughput & Energy",
        }
    }
}

/// Spatially distributed allocation map assigning materials to functional processor blocks.
#[derive(Debug, Clone)]
pub struct BlockAllocationMap {
    allocations: std::collections::HashMap<ProcessorBlockType, HeteroMaterialType>,
}

impl BlockAllocationMap {
    /// Creates an empty allocation map.
    pub fn new() -> Self {
        Self {
            allocations: std::collections::HashMap::new(),
        }
    }

    /// Sets the material assigned to a specific processor block.
    pub fn assign(&mut self, block: ProcessorBlockType, material: HeteroMaterialType) {
        self.allocations.insert(block, material);
    }

    /// Gets the material assigned to a block (defaults to Silicon GAA if unassigned).
    pub fn get(&self, block: ProcessorBlockType) -> HeteroMaterialType {
        self.allocations
            .get(&block)
            .copied()
            .unwrap_or(HeteroMaterialType::SiliconGaa)
    }

    /// Creates the standard uniform baseline: 100% 3nm Silicon GAA everywhere.
    pub fn uniform_silicon() -> Self {
        let mut map = Self::new();
        let all_blocks = [
            ProcessorBlockType::InstructionFetch,
            ProcessorBlockType::InstructionDecode,
            ProcessorBlockType::ExecutionAlu,
            ProcessorBlockType::MemoryAccess,
            ProcessorBlockType::WriteBack,
            ProcessorBlockType::L1Cache,
            ProcessorBlockType::PowerDeliveryFivr,
            ProcessorBlockType::ClockDistribution,
        ];
        for block in all_blocks {
            map.assign(block, HeteroMaterialType::SiliconGaa);
        }
        map
    }

    /// Creates an optimized heterogeneous configuration:
    /// - ALU -> High-velocity InGaAs / Strained Ge
    /// - FIVR -> Wide-bandgap GaN
    /// - L1 Cache -> Ultra-low leakage BEOL IGZO
    /// - Clock Spine -> Low-RC CNT bundle
    /// - Control paths -> Silicon GAA
    pub fn synthesized_heterogeneous() -> Self {
        let mut map = Self::new();
        map.assign(
            ProcessorBlockType::InstructionFetch,
            HeteroMaterialType::SiliconGaa,
        );
        map.assign(
            ProcessorBlockType::InstructionDecode,
            HeteroMaterialType::SiliconGaa,
        );
        map.assign(
            ProcessorBlockType::ExecutionAlu,
            HeteroMaterialType::InGaAsNmos,
        );
        map.assign(
            ProcessorBlockType::MemoryAccess,
            HeteroMaterialType::SiliconGaa,
        );
        map.assign(
            ProcessorBlockType::WriteBack,
            HeteroMaterialType::SiliconGaa,
        );
        map.assign(ProcessorBlockType::L1Cache, HeteroMaterialType::IgzoBeol);
        map.assign(
            ProcessorBlockType::PowerDeliveryFivr,
            HeteroMaterialType::WideBandgapGan,
        );
        map.assign(
            ProcessorBlockType::ClockDistribution,
            HeteroMaterialType::CntBundleInterconnect,
        );
        map
    }
}

impl Default for BlockAllocationMap {
    fn default() -> Self {
        Self::synthesized_heterogeneous()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_property_scaling() {
        let props_si = HeteroMaterialType::SiliconGaa.properties();
        let props_ingaas = HeteroMaterialType::InGaAsNmos.properties();
        let props_gan = HeteroMaterialType::WideBandgapGan.properties();
        let props_igzo = HeteroMaterialType::IgzoBeol.properties();
        let props_cnt = HeteroMaterialType::CntBundleInterconnect.properties();

        // InGaAs electron injection velocity must exceed Silicon by > 2x
        assert!(props_ingaas.injection_velocity_cm_s > 2.0 * props_si.injection_velocity_cm_s);

        // GaN breakdown field must exceed Silicon by > 10x
        assert!(
            props_gan.critical_breakdown_field_mv_cm
                > 10.0 * props_si.critical_breakdown_field_mv_cm
        );

        // IGZO leakage must be zeptofarad-level (< 10^-18 A/um) and > 10^7x lower than Silicon
        assert!(props_igzo.subthreshold_leakage_a_per_um < 1.0e-18);
        assert!(
            props_si.subthreshold_leakage_a_per_um / props_igzo.subthreshold_leakage_a_per_um
                > 1.0e7
        );

        // CNT bundle current capacity must exceed Copper by > 100x
        assert!(props_cnt.max_current_density_a_cm2 > 100.0 * props_si.max_current_density_a_cm2);
    }

    #[test]
    fn test_allocation_map_retrieval() {
        let map = BlockAllocationMap::synthesized_heterogeneous();
        assert_eq!(
            map.get(ProcessorBlockType::ExecutionAlu),
            HeteroMaterialType::InGaAsNmos
        );
        assert_eq!(
            map.get(ProcessorBlockType::L1Cache),
            HeteroMaterialType::IgzoBeol
        );
        assert_eq!(
            map.get(ProcessorBlockType::PowerDeliveryFivr),
            HeteroMaterialType::WideBandgapGan
        );
        assert_eq!(
            map.get(ProcessorBlockType::ClockDistribution),
            HeteroMaterialType::CntBundleInterconnect
        );
        assert_eq!(
            map.get(ProcessorBlockType::InstructionFetch),
            HeteroMaterialType::SiliconGaa
        );
    }
}
