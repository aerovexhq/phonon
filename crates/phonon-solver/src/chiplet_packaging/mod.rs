#![deny(unsafe_code)]

//! Industrial Heterogeneous 2.5D/3D Multi-Die & Chiplet Packaging Co-Simulator.
//!
//! Integrates:
//! 1. Universal Chiplet Interconnect Express (UCIe) & BoW PHY channel verification and eye diagram synthesis.
//! 2. Through-Silicon Via (TSV) & fine-pitch Redistribution Layer (RDL) electrodynamic RLGC extraction.
//! 3. Multi-material Coefficient of Thermal Expansion (CTE) mismatch, warpage bow, and corner micro-bump shear fatigue.
//! 4. Advanced industrial packaging architectures: CoWoS-S/L, Intel EMIB, Standard Organic 2.5D, and 3D SoIC direct hybrid bonding.

pub mod tsv_rdl_parasitics;
pub mod thermo_mechanical_warpage;
pub mod ucie_phy;

pub use tsv_rdl_parasitics::{
    compute_s_parameters, extract_rdl_rlgc, extract_tsv_rlgc, RdlGeometry, RdlRlgc,
    SParameterPoint, TsvGeometry, TsvRlgc,
};
pub use thermo_mechanical_warpage::{
    evaluate_thermo_mechanics, LayerMaterial, PackageStackGeometry, ThermalCycleParams,
    WarpageStressReport,
};
pub use ucie_phy::{
    evaluate_ucie_phy, EyeDiagramSample, UcieDataRateGbps, UcieEyeMetrics, UciePackageType,
    UciePhyParams,
};

/// High-level industrial packaging architecture templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackagingArchitecture {
    /// TSMC CoWoS-S (Chip-on-Wafer-on-Substrate with Full Silicon Interposer & TSVs).
    CoWoS_S,
    /// Intel EMIB (Embedded Multi-die Interconnect Bridge in Organic Cavity).
    Intel_EMIB,
    /// Standard High-Density Organic 2.5D Packaging.
    Organic_2_5D,
    /// TSMC SoIC (3D Direct Hybrid Cu-Cu Die Stacking, pitch < 10 um).
    SoIC_3D,
}

impl PackagingArchitecture {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::CoWoS_S => "TSMC CoWoS-S (Full Si Interposer)",
            Self::Intel_EMIB => "Intel EMIB (Embedded Silicon Bridge)",
            Self::Organic_2_5D => "High-Density Organic 2.5D Substrate",
            Self::SoIC_3D => "TSMC SoIC (3D Direct Hybrid Bonding)",
        }
    }
}

/// Unified telemetry and diagnostic summary report for chiplet packaging.
#[derive(Debug, Clone, PartialEq)]
pub struct PackagingTelemetryReport {
    pub architecture: PackagingArchitecture,
    /// D2D PHY eye height opening in millivolts.
    pub eye_height_mv: f64,
    /// D2D PHY eye width opening in Unit Intervals.
    pub eye_width_ui: f64,
    /// D2D channel insertion loss at Nyquist frequency in dB.
    pub insertion_loss_nyquist_db: f64,
    /// D2D energy efficiency in picojoules per bit (pJ/bit).
    pub energy_efficiency_pj_bit: f64,
    /// TSV loop inductance in pico-Henries.
    pub tsv_inductance_ph: f64,
    /// TSV total parasitic capacitance in femto-Farads.
    pub tsv_capacitance_ff: f64,
    /// RDL characteristic impedance in Ohms.
    pub rdl_z0_ohm: f64,
    /// Maximum out-of-plane package bow warpage in micrometers.
    pub package_bow_warpage_um: f64,
    /// Outer corner micro-bump shear stress in MPa.
    pub corner_bump_shear_stress_mpa: f64,
    /// Coffin-Manson low-cycle thermal fatigue cycles to failure.
    pub thermal_fatigue_cycles: u32,
    /// Overall packaging compliance status.
    pub is_fully_qualified: bool,
}

/// Unified Chiplet Packaging Co-Simulator.
#[derive(Debug, Clone)]
pub struct ChipletPackagingCoSimulator {
    pub architecture: PackagingArchitecture,
    pub ucie_params: UciePhyParams,
    pub tsv_geom: TsvGeometry,
    pub rdl_geom: RdlGeometry,
    pub stack_geom: PackageStackGeometry,
    pub thermal_params: ThermalCycleParams,

    // Cached simulation results
    cached_eye_metrics: UcieEyeMetrics,
    cached_eye_contours: EyeDiagramSample,
    cached_tsv_rlgc: TsvRlgc,
    cached_rdl_rlgc: RdlRlgc,
    cached_s_parameters: Vec<SParameterPoint>,
    cached_warpage: WarpageStressReport,
    cached_report: PackagingTelemetryReport,
}

impl Default for ChipletPackagingCoSimulator {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl ChipletPackagingCoSimulator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fast non-blocking constructor guaranteeing sub-millisecond initialization for cold boot.
    pub fn new_fast() -> Self {
        let architecture = PackagingArchitecture::CoWoS_S;
        let ucie_params = UciePhyParams::default();
        let tsv_geom = TsvGeometry::default();
        let rdl_geom = RdlGeometry::default();
        let stack_geom = PackageStackGeometry::default();
        let thermal_params = ThermalCycleParams::default();

        let (cached_eye_metrics, cached_eye_contours) = evaluate_ucie_phy(&ucie_params);
        let cached_tsv_rlgc = extract_tsv_rlgc(&tsv_geom);
        let cached_rdl_rlgc = extract_rdl_rlgc(&rdl_geom);
        let cached_s_parameters = compute_s_parameters(&cached_tsv_rlgc, &cached_rdl_rlgc);
        let cached_warpage = evaluate_thermo_mechanics(&stack_geom, &thermal_params);

        let is_fully_qualified = cached_eye_metrics.is_compliant
            && cached_warpage.is_coplanar
            && cached_warpage.fatigue_cycles_to_failure >= 1_000;

        let cached_report = PackagingTelemetryReport {
            architecture,
            eye_height_mv: cached_eye_metrics.eye_height_mv,
            eye_width_ui: cached_eye_metrics.eye_width_ui,
            insertion_loss_nyquist_db: cached_eye_metrics.insertion_loss_db,
            energy_efficiency_pj_bit: cached_eye_metrics.energy_efficiency_pj_bit,
            tsv_inductance_ph: cached_tsv_rlgc.l_ph,
            tsv_capacitance_ff: cached_tsv_rlgc.c_total_ff,
            rdl_z0_ohm: cached_rdl_rlgc.z0_ohm,
            package_bow_warpage_um: cached_warpage.max_cycle_warpage_um,
            corner_bump_shear_stress_mpa: cached_warpage.corner_bump_shear_stress_mpa,
            thermal_fatigue_cycles: cached_warpage.fatigue_cycles_to_failure,
            is_fully_qualified,
        };

        Self {
            architecture,
            ucie_params,
            tsv_geom,
            rdl_geom,
            stack_geom,
            thermal_params,
            cached_eye_metrics,
            cached_eye_contours,
            cached_tsv_rlgc,
            cached_rdl_rlgc,
            cached_s_parameters,
            cached_warpage,
            cached_report,
        }
    }

    /// Recomputes all chiplet packaging sub-models and updates telemetry.
    pub fn recompute(&mut self) -> &PackagingTelemetryReport {
        let (eye_metrics, eye_contours) = evaluate_ucie_phy(&self.ucie_params);
        let tsv_rlgc = extract_tsv_rlgc(&self.tsv_geom);
        let rdl_rlgc = extract_rdl_rlgc(&self.rdl_geom);
        let s_params = compute_s_parameters(&tsv_rlgc, &rdl_rlgc);
        let warpage = evaluate_thermo_mechanics(&self.stack_geom, &self.thermal_params);

        let is_fully_qualified = eye_metrics.is_compliant
            && warpage.is_coplanar
            && warpage.fatigue_cycles_to_failure >= 1_000;

        self.cached_eye_metrics = eye_metrics;
        self.cached_eye_contours = eye_contours;
        self.cached_tsv_rlgc = tsv_rlgc;
        self.cached_rdl_rlgc = rdl_rlgc;
        self.cached_s_parameters = s_params;
        self.cached_warpage = warpage;

        self.cached_report = PackagingTelemetryReport {
            architecture: self.architecture,
            eye_height_mv: self.cached_eye_metrics.eye_height_mv,
            eye_width_ui: self.cached_eye_metrics.eye_width_ui,
            insertion_loss_nyquist_db: self.cached_eye_metrics.insertion_loss_db,
            energy_efficiency_pj_bit: self.cached_eye_metrics.energy_efficiency_pj_bit,
            tsv_inductance_ph: self.cached_tsv_rlgc.l_ph,
            tsv_capacitance_ff: self.cached_tsv_rlgc.c_total_ff,
            rdl_z0_ohm: self.cached_rdl_rlgc.z0_ohm,
            package_bow_warpage_um: self.cached_warpage.max_cycle_warpage_um,
            corner_bump_shear_stress_mpa: self.cached_warpage.corner_bump_shear_stress_mpa,
            thermal_fatigue_cycles: self.cached_warpage.fatigue_cycles_to_failure,
            is_fully_qualified,
        };

        &self.cached_report
    }

    /// Applies presets according to selected packaging architecture.
    pub fn apply_architecture_preset(&mut self, arch: PackagingArchitecture) {
        self.architecture = arch;
        match arch {
            PackagingArchitecture::CoWoS_S => {
                self.ucie_params.package_type = UciePackageType::AdvancedSiliconBridge;
                self.ucie_params.data_rate = UcieDataRateGbps::Rate16Gbps;
                self.stack_geom.has_silicon_interposer = true;
                self.stack_geom.bump_pitch_um = 35.0;
                self.tsv_geom.diameter_um = 5.0;
                self.tsv_geom.height_um = 50.0;
            }
            PackagingArchitecture::Intel_EMIB => {
                self.ucie_params.package_type = UciePackageType::AdvancedSiliconBridge;
                self.ucie_params.data_rate = UcieDataRateGbps::Rate16Gbps;
                self.stack_geom.has_silicon_interposer = false;
                self.stack_geom.bump_pitch_um = 45.0;
                self.tsv_geom.diameter_um = 8.0;
                self.tsv_geom.height_um = 75.0;
            }
            PackagingArchitecture::Organic_2_5D => {
                self.ucie_params.package_type = UciePackageType::StandardOrganic;
                self.ucie_params.data_rate = UcieDataRateGbps::Rate8Gbps;
                self.stack_geom.has_silicon_interposer = false;
                self.stack_geom.bump_pitch_um = 110.0;
                self.tsv_geom.diameter_um = 10.0;
                self.tsv_geom.height_um = 100.0;
            }
            PackagingArchitecture::SoIC_3D => {
                self.ucie_params.package_type = UciePackageType::AdvancedSiliconBridge;
                self.ucie_params.data_rate = UcieDataRateGbps::Rate32Gbps;
                self.stack_geom.has_silicon_interposer = true;
                self.stack_geom.bump_pitch_um = 9.0;
                self.tsv_geom.diameter_um = 3.0;
                self.tsv_geom.height_um = 20.0;
            }
        }
        self.recompute();
    }

    pub fn report(&self) -> &PackagingTelemetryReport {
        &self.cached_report
    }

    pub fn eye_metrics(&self) -> &UcieEyeMetrics {
        &self.cached_eye_metrics
    }

    pub fn eye_contours(&self) -> &EyeDiagramSample {
        &self.cached_eye_contours
    }

    pub fn tsv_rlgc(&self) -> &TsvRlgc {
        &self.cached_tsv_rlgc
    }

    pub fn rdl_rlgc(&self) -> &RdlRlgc {
        &self.cached_rdl_rlgc
    }

    pub fn s_parameters(&self) -> &[SParameterPoint] {
        &self.cached_s_parameters
    }

    pub fn warpage_report(&self) -> &WarpageStressReport {
        &self.cached_warpage
    }
}
