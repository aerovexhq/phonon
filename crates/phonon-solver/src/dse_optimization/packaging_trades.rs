#![deny(unsafe_code)]

//! Packaging Architecture Trade Studies & Cost/Performance Engine.
//!
//! Models 4 primary advanced packaging schemes:
//! - Monolithic SoC (single large die, zero D2D latency, high wafer defect cost)
//! - Organic MCM (low-cost multi-chip module, high D2D latency, moderate density)
//! - 2.5D Silicon Interposer (TSMC CoWoS-S/L, fine-pitch RDL, high throughput, low latency)
//! - 3D Direct Hybrid Bonding (TSMC SoIC, direct Cu-Cu micro-pitch, ultra-low energy, thermal density penalty)

/// Industrial packaging architecture selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackagingTechnology {
    MonolithicSoc,
    OrganicMcm,
    SiliconInterposerCoWoS,
    HybridBonding3D,
}

impl PackagingTechnology {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::MonolithicSoc => "Monolithic SoC",
            Self::OrganicMcm => "Organic Substrate MCM",
            Self::SiliconInterposerCoWoS => "2.5D CoWoS Silicon Interposer",
            Self::HybridBonding3D => "3D Direct Hybrid Bonding (SoIC)",
        }
    }

    /// Die-to-Die (D2D) interconnect latency across chiplets in nanoseconds.
    pub fn d2d_latency_ns(&self) -> f64 {
        match self {
            Self::MonolithicSoc => 0.15,
            Self::OrganicMcm => 6.80,
            Self::SiliconInterposerCoWoS => 1.85,
            Self::HybridBonding3D => 0.35,
        }
    }

    /// D2D PHY interconnect energy consumption in picojoules per bit (pJ/bit).
    pub fn d2d_energy_pj_per_bit(&self) -> f64 {
        match self {
            Self::MonolithicSoc => 0.04,
            Self::OrganicMcm => 1.45,
            Self::SiliconInterposerCoWoS => 0.48,
            Self::HybridBonding3D => 0.08,
        }
    }

    /// Packaging substrate/interposer cost in USD per square centimeter.
    pub fn substrate_cost_per_cm2(&self) -> f64 {
        match self {
            Self::MonolithicSoc => 0.0,
            Self::OrganicMcm => 14.50,
            Self::SiliconInterposerCoWoS => 88.00,
            Self::HybridBonding3D => 145.00,
        }
    }

    /// Assembly bonding yield factor (fraction of packages without packaging interconnect defect).
    pub fn assembly_yield_factor(&self) -> f64 {
        match self {
            Self::MonolithicSoc => 0.985,
            Self::OrganicMcm => 0.945,
            Self::SiliconInterposerCoWoS => 0.915,
            Self::HybridBonding3D => 0.875,
        }
    }

    /// Thermal junction-to-ambient resistance penalty in degC/W due to thermal stacking.
    pub fn thermal_resistance_penalty_c_per_w(&self) -> f64 {
        match self {
            Self::MonolithicSoc => 0.0,
            Self::OrganicMcm => 0.04,
            Self::SiliconInterposerCoWoS => 0.11,
            Self::HybridBonding3D => 0.38,
        }
    }
}

/// Evaluated Power, Performance, Area, and Cost (PPA-C) metrics for a packaging candidate.
#[derive(Debug, Clone)]
pub struct PackagingPpacResult {
    pub technology: PackagingTechnology,
    pub total_power_w: f64,
    pub dynamic_power_w: f64,
    pub leakage_power_w: f64,
    pub d2d_interconnect_power_w: f64,
    pub clock_freq_ghz: f64,
    pub effective_throughput_ipc: f64,
    pub total_silicon_area_mm2: f64,
    pub total_package_area_mm2: f64,
    pub unit_manufacturing_cost_usd: f64,
    pub silicon_cost_usd: f64,
    pub packaging_cost_usd: f64,
    pub thermal_headroom_c: f64,
}

/// Evaluates PPA-C for a specific architecture configuration under a selected packaging technology.
pub fn evaluate_packaging_ppac(
    tech: PackagingTechnology,
    num_cores: usize,
    cache_l3_mb: f64,
    freq_ghz: f64,
    vdd_v: f64,
    wafer_fab_cost_usd: f64,
) -> PackagingPpacResult {
    let cores_f = num_cores as f64;

    // Silicon area breakdown
    // Standard 3nm/5nm logic: ~1.8 mm^2 per high-perf core, ~0.65 mm^2 per MB of L3 SRAM
    let core_area_mm2 = cores_f * 1.85;
    let cache_area_mm2 = cache_l3_mb * 0.68;
    let base_logic_area = core_area_mm2 + cache_area_mm2;

    // Multi-chiplet partitioning vs monolithic
    let (silicon_area_mm2, package_area_mm2, d2d_bandwidth_tb_s) = match tech {
        PackagingTechnology::MonolithicSoc => {
            // Monolithic: single die with 15% uncore/I/O
            let total_die = base_logic_area * 1.15;
            (total_die, total_die * 1.6, 0.0)
        }
        PackagingTechnology::OrganicMcm => {
            // Partitioned into 2 compute chiplets + 1 I/O die, with UCIe PHY beachhead area (+18%)
            let chiplet_area = base_logic_area * 1.18;
            (chiplet_area, chiplet_area * 2.4, 1.2)
        }
        PackagingTechnology::SiliconInterposerCoWoS => {
            // Fine-pitch D2D with minimal beachhead overhead (+8%)
            let chiplet_area = base_logic_area * 1.08;
            (chiplet_area, chiplet_area * 2.1, 3.8)
        }
        PackagingTechnology::HybridBonding3D => {
            // 3D face-to-face vertical stacking (cores stacked over cache), reducing footprint area by ~40%
            let stacked_footprint = (core_area_mm2.max(cache_area_mm2)) * 1.05;
            let total_silicon = base_logic_area * 1.05;
            (total_silicon, stacked_footprint * 1.8, 8.5)
        }
    };

    // Power calculations
    // Dynamic core power: P = alpha * C * V^2 * f
    let cap_per_core_pf = 380.0;
    let activity_factor = 0.65;
    let core_dyn_power_w = cores_f * activity_factor * (cap_per_core_pf * 1e-12) * (vdd_v * vdd_v) * (freq_ghz * 1e9);

    // Cache dynamic power
    let cache_dyn_power_w = cache_l3_mb * 0.045 * (vdd_v * vdd_v) * (freq_ghz * 0.6);

    // D2D interconnect power: P_d2d = Bandwidth * Energy_per_bit
    let d2d_energy_pj = tech.d2d_energy_pj_per_bit();
    let d2d_power_w = d2d_bandwidth_tb_s * 8.0 * 1e12 * (d2d_energy_pj * 1e-12);

    let dynamic_power = core_dyn_power_w + cache_dyn_power_w;

    // Static leakage power: exponential function of Vdd and temperature
    let subthreshold_leakage_w = cores_f * 0.42 * (vdd_v / 0.85).powi(3) + cache_l3_mb * 0.025;
    let total_power = dynamic_power + subthreshold_leakage_w + d2d_power_w;

    // Performance throughput: IPC degradation from D2D latency
    let base_ipc = 2.45;
    let latency_penalty_pct = match tech {
        PackagingTechnology::MonolithicSoc => 0.0,
        PackagingTechnology::OrganicMcm => 7.8,
        PackagingTechnology::SiliconInterposerCoWoS => 2.2,
        PackagingTechnology::HybridBonding3D => 0.4,
    };
    let effective_ipc = base_ipc * (1.0 - latency_penalty_pct / 100.0);

    // Cost modeling via Stapper negative binomial yield:
    // Die cost = Wafer_cost / (DPW * Yield)
    let wafer_diameter_mm = 300.0;
    let wafer_radius_mm = wafer_diameter_mm * 0.5 - 3.0;
    let wafer_area_mm2 = std::f64::consts::PI * wafer_radius_mm * wafer_radius_mm;

    let die_area_cm2 = (silicon_area_mm2 / (if tech == PackagingTechnology::MonolithicSoc { 1.0 } else { 3.0 })) * 0.01;
    let d0 = 0.12;
    let alpha = 2.0;
    let die_yield = (1.0 + d0 * die_area_cm2 * 0.75 / alpha).powf(-alpha);

    let dies_per_wafer = (wafer_area_mm2 / (die_area_cm2 * 100.0)).max(1.0);
    let good_dies_per_wafer = (dies_per_wafer * die_yield).max(1.0);

    let per_die_silicon_cost = wafer_fab_cost_usd / good_dies_per_wafer;
    let total_silicon_cost = if tech == PackagingTechnology::MonolithicSoc {
        per_die_silicon_cost
    } else {
        per_die_silicon_cost * 2.8 // Multiple smaller chiplets
    };

    // Substrate & Packaging assembly cost
    let package_area_cm2 = package_area_mm2 * 0.01;
    let substrate_cost = package_area_cm2 * tech.substrate_cost_per_cm2();
    let assembly_base_cost = 18.0;
    let packaging_total_cost = (substrate_cost + assembly_base_cost) / tech.assembly_yield_factor();

    let total_mfg_cost = total_silicon_cost + packaging_total_cost;

    // Thermal junction temperature estimation (assuming 0.35 C/W cooling solution)
    let total_thermal_r = 0.35 + tech.thermal_resistance_penalty_c_per_w();
    let ambient_temp_c = 25.0;
    let junction_temp_c = ambient_temp_c + total_power * total_thermal_r;
    let max_allowable_temp_c = 105.0;
    let thermal_headroom = (max_allowable_temp_c - junction_temp_c).max(0.0);

    PackagingPpacResult {
        technology: tech,
        total_power_w: total_power,
        dynamic_power_w: dynamic_power,
        leakage_power_w: subthreshold_leakage_w,
        d2d_interconnect_power_w: d2d_power_w,
        clock_freq_ghz: freq_ghz,
        effective_throughput_ipc: effective_ipc,
        total_silicon_area_mm2: silicon_area_mm2,
        total_package_area_mm2: package_area_mm2,
        unit_manufacturing_cost_usd: total_mfg_cost,
        silicon_cost_usd: total_silicon_cost,
        packaging_cost_usd: packaging_total_cost,
        thermal_headroom_c: thermal_headroom,
    }
}
