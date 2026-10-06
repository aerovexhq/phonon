#![deny(unsafe_code)]

//! Phase 405: Topological Higher-Order Acoustic Quadrupole Corner-Pumped Polariton Laser
//! & Parity-Time (PT) Symmetric Metamaterial Engine.
//!
//! Provides:
//! - 2D BBH quadrupole lattice with pi-flux and balanced gain/loss PT-symmetry.
//! - Semi-classical rate equations for corner polariton lasing, slope efficiency, and SMSR.
//! - Temporal coherence g^(1)(tau), Schawlow-Townes linewidth, and g^(2)(0) coherence.
//! - Comprehensive 10-point physics audit suite with strict compliance verification.

mod coherence_emission;
mod corner_laser_benchmark;
mod corner_laser_solver;
mod phonon_lasing_dynamics;
mod pt_quadrupole_lattice;

pub use coherence_emission::{
    CoherenceEmissionEngine, CoherenceMetrics, CoherenceParams, TemporalCoherencePoint,
};
pub use corner_laser_benchmark::{CornerLaserBenchmarkResult, CornerLaserBenchmarkRunner};
pub use corner_laser_solver::TopologicalCornerLaserSolver;
pub use phonon_lasing_dynamics::{
    CornerLasingSolver, LasingParams, LasingSolution, LiPoint,
};
pub use pt_quadrupole_lattice::{
    PtComplex, PtCornerId, PtCornerMode, PtQuadrupoleLattice, PtQuadrupoleParams,
};

/// Individual verification item in the 10-point physics audit.
#[derive(Debug, Clone, PartialEq)]
pub struct CornerLaserAuditItem {
    pub name: &'static str,
    pub passed: bool,
    pub measured_value: f64,
    pub threshold_specification: &'static str,
    pub details: String,
}

/// Comprehensive outcome of the 10-point physics audit.
#[derive(Debug, Clone, PartialEq)]
pub struct CornerLaserAuditReport {
    pub items: Vec<CornerLaserAuditItem>,
    pub pass_count: usize,
    pub total_tests: usize,
    pub all_passed: bool,
}

/// Master orchestrator for the Topological Higher-Order Corner Polariton Laser engine.
#[derive(Debug, Clone, PartialEq)]
pub struct TopologicalCornerLaserProcessor {
    pub lattice: PtQuadrupoleLattice,
    pub lasing_solver: CornerLasingSolver,
    pub coherence_engine: CoherenceEmissionEngine,
}

impl Default for TopologicalCornerLaserProcessor {
    fn default() -> Self {
        Self::new(
            PtQuadrupoleParams::default(),
            LasingParams::default(),
            CoherenceParams::default(),
        )
    }
}

impl TopologicalCornerLaserProcessor {
    /// Creates a new master processor with specified parameters.
    pub fn new(
        lattice_params: PtQuadrupoleParams,
        lasing_params: LasingParams,
        coherence_params: CoherenceParams,
    ) -> Self {
        Self {
            lattice: PtQuadrupoleLattice::new(lattice_params),
            lasing_solver: CornerLasingSolver::new(lasing_params),
            coherence_engine: CoherenceEmissionEngine::new(coherence_params),
        }
    }

    /// Fast-boot constructor optimized for sub-2ms cold startup.
    pub fn new_fast() -> Self {
        Self::default()
    }

    /// Comprehensive 10-point physics audit evaluating all topological and laser physical criteria:
    ///
    /// 1. BBH Quantized Quadrupole Moment q_xy = 0.500
    /// 2. Parity-Time (PT) Symmetry & Exceptional Point Transition
    /// 3. 0D Corner Mode Spatial Confinement >= 80%
    /// 4. Selective Corner Mode Threshold Inversion (P_th,corner < P_th,bulk)
    /// 5. Laser Slope Efficiency eta_slope >= 35%
    /// 6. Single-Mode Side-Mode Suppression Ratio SMSR >= 30.0 dB
    /// 7. Coherence Time tau_coh >= 10.0 us
    /// 8. Schawlow-Townes Linewidth Delta_f <= 50.0 kHz
    /// 9. Second-Order Coherence g^(2)(0) in [0.95, 1.05] (Coherent State)
    /// 10. Topological Disorder Immunity against random hopping perturbations
    pub fn audit_laser(&self) -> CornerLaserAuditReport {
        let mut items = Vec::with_capacity(10);

        // 1. BBH Quantized Quadrupole Moment
        let q_xy = self.lattice.quadrupole_moment();
        let q_passed = (q_xy - 0.500).abs() < 1e-6;
        items.push(CornerLaserAuditItem {
            name: "BBH Quantized Quadrupole Moment",
            passed: q_passed,
            measured_value: q_xy,
            threshold_specification: "q_xy = 0.500 (fractional corner charge)",
            details: format!("Measured bulk quadrupole moment q_xy = {:.3}", q_xy),
        });

        // 2. Parity-Time (PT) Symmetry & Exceptional Point Transition
        let ep_threshold = self.lattice.exceptional_point_threshold_mhz();
        let pt_unbroken = self.lattice.is_pt_unbroken();
        let ep_passed = ep_threshold > 0.0 && pt_unbroken;
        items.push(CornerLaserAuditItem {
            name: "PT Symmetry & Exceptional Point Transition",
            passed: ep_passed,
            measured_value: ep_threshold,
            threshold_specification: "gamma_crit > 0 and unbroken PT (gamma < gamma_crit)",
            details: format!(
                "Exceptional Point at gamma_crit = {:.3} MHz; operates in unbroken PT state ({})",
                ep_threshold, pt_unbroken
            ),
        });

        // 3. 0D Corner Mode Spatial Confinement >= 80%
        let confinement = self.lattice.corner_confinement_ratio();
        let confinement_pct = confinement * 100.0;
        let conf_passed = confinement >= 0.80;
        items.push(CornerLaserAuditItem {
            name: "0D Corner Mode Spatial Confinement",
            passed: conf_passed,
            measured_value: confinement_pct,
            threshold_specification: "Confinement >= 80.0%",
            details: format!(
                "Energy confinement fraction in 4 outer corner cells = {:.1}%",
                confinement_pct
            ),
        });

        // 4. Selective Corner Mode Threshold Inversion (P_th,corner < P_th,bulk)
        let p_th_corner = self.lasing_solver.compute_threshold_power_mw();
        let p_th_bulk = self.lasing_solver.compute_bulk_threshold_power_mw();
        let inversion_ratio = p_th_bulk / p_th_corner.max(0.1);
        let inv_passed = p_th_corner < p_th_bulk;
        items.push(CornerLaserAuditItem {
            name: "Selective Corner Mode Threshold Inversion",
            passed: inv_passed,
            measured_value: inversion_ratio,
            threshold_specification: "P_th,corner < P_th,bulk (ratio > 1.0)",
            details: format!(
                "Corner P_th = {:.2} mW vs Bulk P_th = {:.2} mW (Inversion ratio = {:.2}x)",
                p_th_corner, p_th_bulk, inversion_ratio
            ),
        });

        // 5. Laser Slope Efficiency eta_slope >= 35%
        let slope = self.lasing_solver.compute_slope_efficiency();
        let slope_pct = slope * 100.0;
        let slope_passed = slope >= 0.35;
        items.push(CornerLaserAuditItem {
            name: "Laser Slope Efficiency",
            passed: slope_passed,
            measured_value: slope_pct,
            threshold_specification: "eta_slope >= 35.0%",
            details: format!("Differential slope efficiency = {:.1}%", slope_pct),
        });

        // 6. Single-Mode Side-Mode Suppression Ratio SMSR >= 30.0 dB
        let lasing_sol = self.lasing_solver.solve();
        let smsr = lasing_sol.smsr_db;
        let smsr_passed = smsr >= 30.0;
        items.push(CornerLaserAuditItem {
            name: "Single-Mode Side-Mode Suppression Ratio",
            passed: smsr_passed,
            measured_value: smsr,
            threshold_specification: "SMSR >= 30.0 dB",
            details: format!("Measured SMSR = {:.1} dB over competing modes", smsr),
        });

        // Coherence calculation using solved threshold
        let coh_metrics = self.coherence_engine.evaluate_metrics(
            self.lasing_solver.params.pump_rate_mw,
            lasing_sol.threshold_pump_power_mw,
        );

        // 7. Coherence Time tau_coh >= 10.0 us
        let tau_coh = coh_metrics.coherence_time_us;
        let tau_passed = tau_coh >= 10.0;
        items.push(CornerLaserAuditItem {
            name: "Coherence Time",
            passed: tau_passed,
            measured_value: tau_coh,
            threshold_specification: "tau_coh >= 10.0 us",
            details: format!("First-order coherence time tau_coh = {:.2} us", tau_coh),
        });

        // 8. Schawlow-Townes Linewidth Delta_f <= 50.0 kHz
        let linewidth = coh_metrics.schawlow_townes_linewidth_khz;
        let lw_passed = linewidth <= 50.0;
        items.push(CornerLaserAuditItem {
            name: "Schawlow-Townes Linewidth Narrowing",
            passed: lw_passed,
            measured_value: linewidth,
            threshold_specification: "Delta_f <= 50.0 kHz",
            details: format!("Laser emission linewidth Delta_f = {:.2} kHz", linewidth),
        });

        // 9. Second-Order Coherence g^(2)(0) in [0.95, 1.05] (Coherent State)
        let g2_0 = coh_metrics.zero_delay_second_order_coherence;
        let g2_passed = g2_0 >= 0.95 && g2_0 <= 1.05;
        items.push(CornerLaserAuditItem {
            name: "Second-Order Photon Coherence g^(2)(0)",
            passed: g2_passed,
            measured_value: g2_0,
            threshold_specification: "g^(2)(0) in [0.95, 1.05] (Poissonian coherent state)",
            details: format!("Measured g^(2)(0) = {:.3}", g2_0),
        });

        // 10. Topological Disorder Immunity against random hopping perturbations
        let immunity = self.lattice.disorder_immunity_test(0.10, 50);
        let imm_pct = immunity * 100.0;
        let imm_passed = immunity >= 0.90;
        items.push(CornerLaserAuditItem {
            name: "Topological Disorder Immunity",
            passed: imm_passed,
            measured_value: imm_pct,
            threshold_specification: "Immunity retention >= 90.0% under 10% disorder",
            details: format!("Corner state survival rate = {:.1}%", imm_pct),
        });

        let pass_count = items.iter().filter(|i| i.passed).count();
        let total_tests = items.len();
        let all_passed = pass_count == total_tests;

        CornerLaserAuditReport {
            items,
            pass_count,
            total_tests,
            all_passed,
        }
    }
}
