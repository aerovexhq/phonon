#![deny(unsafe_code)]

//! Dissipative Topological Polariton Skin Laser & Non-Hermitian Chiral Acoustic Gyroscope Array Module (Phase 452).
//!
//! Master orchestrator integrating:
//! 1. Non-Hermitian skin-effect (NHSE) modal selection with asymmetric directional coupling (J_R / J_L >= 3.0).
//! 2. Boundary-localized topological polariton single-mode lasing with side-mode suppression ratio (SMSR >= 32.0 dB).
//! 3. Complex-energy Riemann surface point-gap topological winding W = +1 and Generalized Brillouin Zone deformation.
//! 4. Non-Hermitian Sagnac gyroscope rotation sensing with exceptional point enhancement (eta >= 45x).
//! 5. Coherent polariton gain medium kinetics with low threshold (P_th <= 1.80 mW) and Schawlow-Townes linewidth narrowing (Delta nu <= 12.0 kHz).

pub mod chiral_sagnac_gyroscope;
pub mod polariton_gain_medium;
pub mod riemann_energy_winding;
pub mod skin_effect_lasing;

pub use chiral_sagnac_gyroscope::{
    ChiralSagnacGyroscopeSolver, GyroscopePerformancePoint, GyroscopeSagnacMetrics,
    GyroscopeSagnacParams,
};
pub use polariton_gain_medium::{
    PolaritonGainMediumSolver, PolaritonGainMetrics, PolaritonGainParams,
    PolaritonPowerCurvePoint,
};
pub use riemann_energy_winding::{
    RiemannEnergyPoint, RiemannEnergyWindingSolver, RiemannWindingMetrics,
    RiemannWindingParams,
};
pub use skin_effect_lasing::{
    SkinEffectLasingMetrics, SkinEffectLasingParams, SkinEffectLasingSolver,
    SkinModeSpatialPoint,
};

/// 10-point rigorous physics audit report for the Dissipative Topological Polariton Skin Laser & Gyroscope.
#[derive(Debug, Clone)]
pub struct SkinPolaritonLaserAuditReport {
    /// 1. Asymmetric hopping contrast J_R / J_L >= 3.0.
    pub asymmetric_hopping_contrast_pass: bool,
    /// 2. Non-Hermitian skin mode penetration depth xi_skin <= 3.5 cells.
    pub skin_depth_pass: bool,
    /// 3. Boundary modal energy localization ratio >= 88.0%.
    pub boundary_localization_pass: bool,
    /// 4. Side-mode suppression ratio (SMSR) >= 32.0 dB.
    pub side_mode_suppression_pass: bool,
    /// 5. Complex eigenenergy point-gap topological winding number W == +1.
    pub point_gap_winding_pass: bool,
    /// 6. Generalized Brillouin Zone (GBZ) deformation magnitude |r_GBZ - 1.0| >= 0.30.
    pub gbz_deformation_pass: bool,
    /// 7. Non-Hermitian Sagnac rotation sensitivity enhancement factor eta >= 45.0x.
    pub gyro_enhancement_pass: bool,
    /// 8. Gyroscope Angle Random Walk (ARW) <= 0.008 deg / sqrt(h).
    pub angle_random_walk_pass: bool,
    /// 9. Coherent polariton lasing threshold power P_th <= 1.80 mW.
    pub laser_threshold_pass: bool,
    /// 10. Schawlow-Townes emission linewidth narrowing Delta nu <= 12.0 kHz.
    pub linewidth_narrowing_pass: bool,
}

impl SkinPolaritonLaserAuditReport {
    /// Returns true if all 10 physics audit criteria evaluated to PASS.
    pub fn all_passed(&self) -> bool {
        self.asymmetric_hopping_contrast_pass
            && self.skin_depth_pass
            && self.boundary_localization_pass
            && self.side_mode_suppression_pass
            && self.point_gap_winding_pass
            && self.gbz_deformation_pass
            && self.gyro_enhancement_pass
            && self.angle_random_walk_pass
            && self.laser_threshold_pass
            && self.linewidth_narrowing_pass
    }

    /// Returns the audit score as (passed_count, total_count).
    pub fn score(&self) -> (usize, usize) {
        let items = [
            self.asymmetric_hopping_contrast_pass,
            self.skin_depth_pass,
            self.boundary_localization_pass,
            self.side_mode_suppression_pass,
            self.point_gap_winding_pass,
            self.gbz_deformation_pass,
            self.gyro_enhancement_pass,
            self.angle_random_walk_pass,
            self.laser_threshold_pass,
            self.linewidth_narrowing_pass,
        ];
        let passed = items.iter().filter(|&&p| p).count();
        (passed, items.len())
    }

    /// Formats a human-readable text summary of the audit checklist.
    pub fn summary(&self) -> String {
        let (passed, total) = self.score();
        format!(
            "Skin Polariton Laser & Gyroscope Audit: {}/{} PASS\n\
             1. Asymmetric Hopping Contrast (J_R / J_L >= 3.0): {}\n\
             2. Skin Penetration Depth (xi <= 3.5 cells): {}\n\
             3. Boundary Modal Localization (>= 88.0%): {}\n\
             4. Side-Mode Suppression Ratio (SMSR >= 32.0 dB): {}\n\
             5. Point-Gap Winding Number (W == +1): {}\n\
             6. GBZ Deformation (|r_GBZ - 1.0| >= 0.30): {}\n\
             7. Sagnac Sensitivity Enhancement (eta >= 45.0x): {}\n\
             8. Gyroscope Angle Random Walk (ARW <= 0.008 deg/sqrt(h)): {}\n\
             9. Lasing Threshold Power (P_th <= 1.80 mW): {}\n\
             10. Schawlow-Townes Linewidth (Delta nu <= 12.0 kHz): {}",
            passed,
            total,
            if self.asymmetric_hopping_contrast_pass { "PASS" } else { "FAIL" },
            if self.skin_depth_pass { "PASS" } else { "FAIL" },
            if self.boundary_localization_pass { "PASS" } else { "FAIL" },
            if self.side_mode_suppression_pass { "PASS" } else { "FAIL" },
            if self.point_gap_winding_pass { "PASS" } else { "FAIL" },
            if self.gbz_deformation_pass { "PASS" } else { "FAIL" },
            if self.gyro_enhancement_pass { "PASS" } else { "FAIL" },
            if self.angle_random_walk_pass { "PASS" } else { "FAIL" },
            if self.laser_threshold_pass { "PASS" } else { "FAIL" },
            if self.linewidth_narrowing_pass { "PASS" } else { "FAIL" }
        )
    }
}

/// Master processor orchestrating the Dissipative Topological Polariton Skin Laser & Gyroscope Array.
#[derive(Debug, Clone)]
pub struct SkinPolaritonLaserProcessor {
    pub skin_params: SkinEffectLasingParams,
    pub gyro_params: GyroscopeSagnacParams,
    pub winding_params: RiemannWindingParams,
    pub gain_params: PolaritonGainParams,
}

impl Default for SkinPolaritonLaserProcessor {
    fn default() -> Self {
        Self {
            skin_params: SkinEffectLasingParams::default(),
            gyro_params: GyroscopeSagnacParams::default(),
            winding_params: RiemannWindingParams::default(),
            gain_params: PolaritonGainParams::default(),
        }
    }
}

impl SkinPolaritonLaserProcessor {
    /// Constructs a new master processor.
    pub fn new(
        skin_params: SkinEffectLasingParams,
        gyro_params: GyroscopeSagnacParams,
        winding_params: RiemannWindingParams,
        gain_params: PolaritonGainParams,
    ) -> Self {
        Self {
            skin_params,
            gyro_params,
            winding_params,
            gain_params,
        }
    }

    /// Evaluates the complete 10-point physics audit.
    pub fn evaluate_audit(&self) -> SkinPolaritonLaserAuditReport {
        let skin_solver = SkinEffectLasingSolver::new(self.skin_params.clone());
        let gyro_solver = ChiralSagnacGyroscopeSolver::new(self.gyro_params.clone());
        let winding_solver = RiemannEnergyWindingSolver::new(self.winding_params.clone());
        let gain_solver = PolaritonGainMediumSolver::new(self.gain_params.clone());

        let skin_m = skin_solver.evaluate_metrics();
        let gyro_m = gyro_solver.evaluate_metrics();
        let wind_m = winding_solver.evaluate_metrics();
        let gain_m = gain_solver.evaluate_metrics();

        SkinPolaritonLaserAuditReport {
            asymmetric_hopping_contrast_pass: skin_m.hopping_asymmetry_ratio >= 3.0,
            skin_depth_pass: skin_m.skin_depth_cells <= 3.5,
            boundary_localization_pass: skin_m.boundary_localization_ratio >= 0.88,
            side_mode_suppression_pass: skin_m.side_mode_suppression_ratio_db >= 32.0,
            point_gap_winding_pass: wind_m.point_gap_winding_number == 1,
            gbz_deformation_pass: wind_m.gbz_deformation_magnitude >= 0.30,
            gyro_enhancement_pass: gyro_m.sensitivity_enhancement_factor >= 45.0,
            angle_random_walk_pass: gyro_m.angle_random_walk_deg_sqrt_h <= 0.008,
            laser_threshold_pass: gain_m.threshold_power_mw <= 1.80,
            linewidth_narrowing_pass: gain_m.schawlow_townes_linewidth_khz <= 12.0,
        }
    }
}
