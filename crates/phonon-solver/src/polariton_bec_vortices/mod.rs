#![deny(unsafe_code)]

//! Phase 412: Phonon Studio Dissipative Polariton BEC Vortices, Non-Equilibrium Superfluidity
//! & Josephson Acoustic Interferometer.
//!
//! Provides driven-dissipative Gross-Pitaevskii solver, quantized vortex dynamics,
//! Landau superfluidity, and acoustic Josephson strain interferometry.

pub mod gross_pitaevskii;
pub mod quantized_vortices;
pub mod josephson_interferometer;

pub use gross_pitaevskii::{
    BecCondensationMetrics, CondensateSpatialPoint, GrossPitaevskiiSolver, PolaritonBecParams,
    BOLTZMANN_K_EV, ELECTRON_MASS_KG, HBAR, HBAR_MEV_PS,
};
pub use quantized_vortices::{
    QuantizedVortexSolver, VortexCharge, VortexGridPoint, VortexLatticeMetrics,
    VortexSuperfluidParams,
};
pub use josephson_interferometer::{
    FringePatternPoint, JosephsonInterferometerParams, JosephsonInterferometerSolver,
    JosephsonSensorMetrics, JosephsonTrajectoryPoint,
};

/// An individual criterion in the 10-point physics audit checklist.
#[derive(Debug, Clone, PartialEq)]
pub struct PolaritonBecAuditCriterion {
    pub name: String,
    pub description: String,
    pub expected: String,
    pub actual: String,
    pub passed: bool,
}

/// Comprehensive physics audit report for the Polariton BEC system.
#[derive(Debug, Clone, PartialEq)]
pub struct PolaritonBecAuditReport {
    pub criteria: Vec<PolaritonBecAuditCriterion>,
    pub passed_count: usize,
    pub total_count: usize,
    pub all_passed: bool,
}

/// Master orchestrator unifying Gross-Pitaevskii condensation, quantized vortices, and Josephson interferometry.
#[derive(Debug, Clone)]
pub struct PolaritonBecInterferometer {
    pub gpe_solver: GrossPitaevskiiSolver,
    pub vortex_solver: QuantizedVortexSolver,
    pub josephson_solver: JosephsonInterferometerSolver,
}

impl Default for PolaritonBecInterferometer {
    fn default() -> Self {
        let gpe_params = PolaritonBecParams::default();
        let gpe_solver = GrossPitaevskiiSolver::new(gpe_params.clone());
        let gpe_metrics = gpe_solver.evaluate_metrics();

        let vortex_params = VortexSuperfluidParams {
            healing_length_um: gpe_metrics.healing_length_um,
            peak_density_um2: gpe_metrics.peak_density_um2,
            effective_mass_kg: gpe_solver.effective_mass_kg(),
            ..Default::default()
        };

        let josephson_params = JosephsonInterferometerParams::default();

        Self {
            gpe_solver,
            vortex_solver: QuantizedVortexSolver::new(vortex_params),
            josephson_solver: JosephsonInterferometerSolver::new(josephson_params),
        }
    }
}

impl PolaritonBecInterferometer {
    /// Creates a new system with custom solvers.
    pub fn new(
        gpe_params: PolaritonBecParams,
        vortex_params: VortexSuperfluidParams,
        josephson_params: JosephsonInterferometerParams,
    ) -> Self {
        Self {
            gpe_solver: GrossPitaevskiiSolver::new(gpe_params),
            vortex_solver: QuantizedVortexSolver::new(vortex_params),
            josephson_solver: JosephsonInterferometerSolver::new(josephson_params),
        }
    }

    /// Evaluates the comprehensive 10-point physics audit checklist.
    pub fn audit_polariton_bec(&self) -> PolaritonBecAuditReport {
        let mut criteria = Vec::with_capacity(10);

        let gpe_metrics = self.gpe_solver.evaluate_metrics();
        let vortex_metrics = self.vortex_solver.evaluate_metrics();
        let sensor_metrics = self.josephson_solver.evaluate_sensor_metrics();

        // 1. Condensation Threshold & Spontaneous Symmetry Breaking
        let is_condensed = self.gpe_solver.is_condensed();
        let p_ratio = self.gpe_solver.params.pump_power_ratio;
        let c1_pass = is_condensed && gpe_metrics.peak_density_um2 > 0.0;
        criteria.push(PolaritonBecAuditCriterion {
            name: "Condensation Threshold".to_string(),
            description: "Spontaneous condensation above threshold P > P_th".to_string(),
            expected: "P / P_th > 1.0, n_0 > 0".to_string(),
            actual: format!("P / P_th = {:.2}, n_0 = {:.1} um^-2", p_ratio, gpe_metrics.peak_density_um2),
            passed: c1_pass,
        });

        // 2. Long-Range Spatial Phase Coherence
        let lc = gpe_metrics.coherence_length_um;
        let c2_pass = lc >= 10.0;
        criteria.push(PolaritonBecAuditCriterion {
            name: "Long-Range Coherence".to_string(),
            description: "First-order coherence length l_c >= 10.0 um".to_string(),
            expected: ">= 10.0 um".to_string(),
            actual: format!("{:.2} um", lc),
            passed: c2_pass,
        });

        // 3. Healing Length Confinement
        let xi = gpe_metrics.healing_length_um;
        let c3_pass = xi >= 0.5 && xi <= 5.0;
        criteria.push(PolaritonBecAuditCriterion {
            name: "Healing Length Confinement".to_string(),
            description: "Microcavity healing length xi in [0.5, 5.0] um".to_string(),
            expected: "0.5 to 5.0 um".to_string(),
            actual: format!("{:.2} um", xi),
            passed: c3_pass,
        });

        // 4. Quantized Vortex Circulation
        let circ = vortex_metrics.quantized_circulation_units;
        let c4_pass = (circ.abs() - 1.0).abs() < 1e-6 || circ.abs() == 2.0;
        criteria.push(PolaritonBecAuditCriterion {
            name: "Quantized Circulation".to_string(),
            description: "Integer quantized vortex circulation Gamma = ell * (h / m*)".to_string(),
            expected: "ell = +/-1 or +/-2".to_string(),
            actual: format!("ell = {:.0}", circ),
            passed: c4_pass,
        });

        // 5. Vortex Core Density Vanishing
        let core_pt = self.vortex_solver.evaluate_grid_point(0.0, 0.0);
        let c5_pass = core_pt.normalized_density < 0.01;
        criteria.push(PolaritonBecAuditCriterion {
            name: "Vortex Core Depletion".to_string(),
            description: "Density vanishing at vortex core center r = 0".to_string(),
            expected: "n(0) / n_0 < 0.01".to_string(),
            actual: format!("{:.4}", core_pt.normalized_density),
            passed: c5_pass,
        });

        // 6. Landau Superfluidity & Drag Suppression
        let vc = vortex_metrics.landau_critical_velocity_ms;
        let supp_db = vortex_metrics.drag_suppression_db;
        let c6_pass = vc > 1.0e5 && supp_db >= 30.0;
        criteria.push(PolaritonBecAuditCriterion {
            name: "Landau Superfluidity".to_string(),
            description: "Critical velocity v_c > 1e5 m/s and drag suppression >= 30 dB".to_string(),
            expected: "v_c > 1e5 m/s, Drag supp >= 30.0 dB".to_string(),
            actual: format!("v_c = {:.1e} m/s, Supp = {:.1} dB", vc, supp_db),
            passed: c6_pass,
        });

        // 7. Josephson Plasma Oscillation Frequency
        let fj = sensor_metrics.plasma_frequency_ghz;
        let c7_pass = fj >= 10.0 && fj <= 150.0;
        criteria.push(PolaritonBecAuditCriterion {
            name: "Josephson Plasma Frequency".to_string(),
            description: "Plasma oscillation frequency f_J in [10, 150] GHz".to_string(),
            expected: "10.0 to 150.0 GHz".to_string(),
            actual: format!("{:.1} GHz", fj),
            passed: c7_pass,
        });

        // 8. Macroscopic Quantum Self-Trapping Transition
        let zc = sensor_metrics.mqst_critical_imbalance;
        let c8_pass = zc > 0.0 && zc <= 1.0;
        criteria.push(PolaritonBecAuditCriterion {
            name: "MQST Critical Boundary".to_string(),
            description: "Critical population imbalance z_c in (0, 1]".to_string(),
            expected: "0.0 < z_c <= 1.0".to_string(),
            actual: format!("z_c = {:.3}", zc),
            passed: c8_pass,
        });

        // 9. Sub-Nanostrain Sensitivity
        let eps_min = sensor_metrics.minimum_detectable_strain;
        let c9_pass = eps_min < 1.0e-9;
        criteria.push(PolaritonBecAuditCriterion {
            name: "Sub-Nanostrain Sensitivity".to_string(),
            description: "Minimum detectable acoustic strain epsilon_min < 1e-9 / sqrt(Hz)".to_string(),
            expected: "< 1.0e-9 / sqrt(Hz)".to_string(),
            actual: format!("{:.2e} / sqrt(Hz)", eps_min),
            passed: c9_pass,
        });

        // 10. Fringe Contrast & Visibility
        let vis = sensor_metrics.fringe_visibility;
        let c10_pass = vis >= 0.90;
        criteria.push(PolaritonBecAuditCriterion {
            name: "Fringe Visibility Contrast".to_string(),
            description: "Interference fringe visibility V >= 90.0%".to_string(),
            expected: ">= 90.0%".to_string(),
            actual: format!("{:.1}%", vis * 100.0),
            passed: c10_pass,
        });

        let passed_count = criteria.iter().filter(|c| c.passed).count();
        let total_count = criteria.len();
        let all_passed = passed_count == total_count;

        PolaritonBecAuditReport {
            criteria,
            passed_count,
            total_count,
            all_passed,
        }
    }
}
