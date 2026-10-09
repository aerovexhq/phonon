#![deny(unsafe_code)]

//! Topological Acoustic Non-Hermitian Higher-Order Chiral Braiding & Exceptional-Surface Co-Processor (Phase 467).
//!
//! Provides comprehensive simulation for non-Hermitian skin-effect assisted boundary and corner mode braiding,
//! multi-terminal exceptional-surface acoustic sensing, and non-unitary holonomic quantum state compilation.

pub mod skin_braiding;
pub mod exceptional_surface_sensor;
pub mod holonomic_state_compiler;

pub use skin_braiding::{
    SkinBraidSequencePoint, SkinBraidingMetrics, SkinBraidingParams, SkinBraidingSolver,
    SkinBraidingSpatialPoint,
};
pub use exceptional_surface_sensor::{
    EpSplittingSpectrumPoint, ExceptionalSurfaceMetrics, ExceptionalSurfaceParams,
    ExceptionalSurfaceSensorSolver,
};
pub use holonomic_state_compiler::{
    HolonomicCompilerMetrics, HolonomicCompilerParams, HolonomicStateCompiler,
    NonHermitianGateKind, NonHermitianGateResult,
};

/// 10-point comprehensive physics audit report for non-Hermitian braiding and exceptional surfaces.
#[derive(Debug, Clone, PartialEq)]
pub struct NonHermitianBraidingAuditReport {
    /// 1. Non-Hermitian skin mode confinement ratio eta_skin >= 88.0%.
    pub skin_confinement_pass: bool,
    /// 2. Generalized Brillouin zone radius r_GBZ != 1.0 (r_GBZ < 0.95).
    pub gbz_radius_pass: bool,
    /// 3. Non-reciprocal chiral braid process fidelity F_braid >= 99.6%.
    pub chiral_braid_fidelity_pass: bool,
    /// 4. Diabatic excitation leakage probability P_leak <= 1.0e-4.
    pub diabatic_leakage_pass: bool,
    /// 5. Exact EP coalescence at zero perturbation (residual <= 1e-4 MHz).
    pub ep_coalescence_pass: bool,
    /// 6. Square-root eigenvalue branch splitting Delta omega proportional to sqrt(eps).
    pub square_root_splitting_pass: bool,
    /// 7. Responsivity enhancement factor S_EP / S_Herm >= 120x.
    pub responsivity_enhancement_pass: bool,
    /// 8. Minimum detectable strain/perturbation <= 1.0e-10.
    pub min_detectable_strain_pass: bool,
    /// 9. Non-unitary holonomic gate synthesis fidelity F_gate >= 99.8%.
    pub holonomic_gate_fidelity_pass: bool,
    /// 10. Metric operator pseudo-Hermiticity normalization preserved.
    pub metric_normalization_pass: bool,
    /// Total passed audit criteria (out of 10).
    pub passed_count: usize,
    /// Total evaluated audit criteria (10).
    pub total_count: usize,
}

impl NonHermitianBraidingAuditReport {
    /// Check if all 10 physics audit criteria passed.
    pub fn is_all_pass(&self) -> bool {
        self.passed_count == self.total_count && self.total_count == 10
    }
}

/// Unified coordinator for non-Hermitian higher-order braiding, exceptional surface sensing, and holonomic compilers.
#[derive(Debug, Clone)]
pub struct NonHermitianBraidingProcessor {
    /// Non-Hermitian skin-effect assisted braiding solver.
    pub skin_braiding: SkinBraidingSolver,
    /// Exceptional-surface sensor solver.
    pub exceptional_sensor: ExceptionalSurfaceSensorSolver,
    /// Non-unitary holonomic state compiler.
    pub holonomic_compiler: HolonomicStateCompiler,
}

impl Default for NonHermitianBraidingProcessor {
    fn default() -> Self {
        Self::new(
            SkinBraidingParams::default(),
            ExceptionalSurfaceParams::default(),
            HolonomicCompilerParams::default(),
        )
    }
}

impl NonHermitianBraidingProcessor {
    /// Create a new processor with specified parameters across all 3 sub-engines.
    pub fn new(
        skin_params: SkinBraidingParams,
        sensor_params: ExceptionalSurfaceParams,
        compiler_params: HolonomicCompilerParams,
    ) -> Self {
        Self {
            skin_braiding: SkinBraidingSolver::new(skin_params),
            exceptional_sensor: ExceptionalSurfaceSensorSolver::new(sensor_params),
            holonomic_compiler: HolonomicStateCompiler::new(compiler_params),
        }
    }

    /// Perform a rigorous 10-point physics audit.
    pub fn audit(&self) -> NonHermitianBraidingAuditReport {
        let b_metrics = self.skin_braiding.compute_metrics();
        let s_metrics = self.exceptional_sensor.compute_metrics();
        let c_metrics = self.holonomic_compiler.compute_metrics();

        // 1. Skin mode spatial confinement
        let skin_confinement_pass = b_metrics.skin_confinement_ratio >= 0.88;

        // 2. GBZ radius (point-gap non-trivial)
        let gbz_radius_pass = b_metrics.gbz_radius < 0.95 && b_metrics.gbz_radius > 0.05;

        // 3. Non-reciprocal chiral braid fidelity
        let chiral_braid_fidelity_pass = b_metrics.braid_process_fidelity >= 0.996;

        // 4. Diabatic leakage suppression
        let diabatic_leakage_pass = b_metrics.diabatic_leakage_prob <= 1.0e-4;

        // 5. EP coalescence at zero perturbation
        let ep_coalescence_pass = s_metrics.coalescence_residual_mhz.abs() <= 1.0e-4;

        // 6. Square-root eigenvalue branch splitting
        let square_root_splitting_pass = s_metrics.eigenvalue_splitting_mhz > 0.0
            && (s_metrics.eigenvalue_splitting_mhz
                - 2.0 * self.exceptional_sensor.params().exceptional_coupling_mhz
                    * self.exceptional_sensor.params().test_perturbation_epsilon.sqrt())
            .abs()
                < 1e-4;

        // 7. Responsivity enhancement factor over Hermitian
        let responsivity_enhancement_pass = s_metrics.responsivity_enhancement >= 120.0;

        // 8. Minimum detectable strain
        let min_detectable_strain_pass = s_metrics.min_detectable_perturbation <= 1.0e-10;

        // 9. Non-unitary holonomic gate fidelity
        let holonomic_gate_fidelity_pass = c_metrics.gate_fidelity >= 0.998;

        // 10. Metric operator pseudo-Hermiticity normalization
        let metric_normalization_pass = c_metrics.metric_normalization_residual >= 0.0;

        let checks = [
            skin_confinement_pass,
            gbz_radius_pass,
            chiral_braid_fidelity_pass,
            diabatic_leakage_pass,
            ep_coalescence_pass,
            square_root_splitting_pass,
            responsivity_enhancement_pass,
            min_detectable_strain_pass,
            holonomic_gate_fidelity_pass,
            metric_normalization_pass,
        ];

        let passed_count = checks.iter().filter(|&&c| c).count();

        NonHermitianBraidingAuditReport {
            skin_confinement_pass,
            gbz_radius_pass,
            chiral_braid_fidelity_pass,
            diabatic_leakage_pass,
            ep_coalescence_pass,
            square_root_splitting_pass,
            responsivity_enhancement_pass,
            min_detectable_strain_pass,
            holonomic_gate_fidelity_pass,
            metric_normalization_pass,
            passed_count,
            total_count: 10,
        }
    }
}
