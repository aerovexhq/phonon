#![deny(unsafe_code)]

//! Phase 436: Phonon Studio Non-Hermitian Exceptional Surface Chiral Phonon Diode
//! & Unidirectional Quantum Repeater.
//!
//! Master orchestrator and 10-point physics audit checklist.

pub mod chiral_phonon_diode;
pub mod exceptional_surface;
pub mod quantum_repeater_node;

pub use chiral_phonon_diode::{
    ChiralDiodeMetrics, ChiralDiodeParams, ChiralDiodeSolver, DiodeSMatrixPoint,
    WaveguideModeSpatialPoint,
};
pub use exceptional_surface::{
    ChiralExceptionalSurfaceMetrics, ChiralExceptionalSurfaceParams,
    ChiralExceptionalSurfacePoint, ChiralExceptionalSurfaceSolver, Complex,
    ExceptionalSurfaceMetrics, ExceptionalSurfaceParams, ExceptionalSurfacePoint,
    ExceptionalSurfaceSolver, FermiArcSegment,
};
pub use quantum_repeater_node::{
    QuantumRepeaterMetrics, QuantumRepeaterParams, QuantumRepeaterSolver, RepeaterTimePoint,
};

/// 10-point physics audit report for Phase 436.
#[derive(Debug, Clone, PartialEq)]
pub struct ExceptionalSurfaceAuditReport {
    /// 1. Exceptional Surface coalescence (eigenvalue splitting coalesces at surface).
    pub exceptional_surface_coalescence_pass: bool,
    /// 2. Square-root branch cut scaling (|Delta E| proportional to sqrt(delta_k)).
    pub square_root_branch_cut_pass: bool,
    /// 3. Anisotropic bulk Fermi arcs (open Fermi arc connectivity in complex Brillouin zone).
    pub anisotropic_fermi_arcs_pass: bool,
    /// 4. Group velocity non-reciprocal asymmetry (v_fwd / v_bwd >= 5.0).
    pub group_velocity_asymmetry_pass: bool,
    /// 5. Chiral diode forward insertion loss (IL <= 0.50 dB).
    pub chiral_diode_insertion_loss_pass: bool,
    /// 6. Chiral diode backward isolation (ISO >= 35.0 dB).
    pub chiral_diode_isolation_pass: bool,
    /// 7. Directional rectification contrast ratio (R >= 35.0 dB).
    pub rectification_contrast_pass: bool,
    /// 8. Port return loss and matching (RL >= 20.0 dB).
    pub return_loss_match_pass: bool,
    /// 9. Protected quantum repeater Bell pair fidelity (F_ent >= 0.980).
    pub quantum_repeater_fidelity_pass: bool,
    /// 10. Cryogenic backscatter noise shielding and memory coherence (T2* >= 50.0 us, ISO >= 35.0 dB).
    pub cryogenic_noise_suppression_pass: bool,
    /// Total score out of 10.
    pub total_score: usize,
    /// Whether all 10 criteria passed.
    pub all_passed: bool,
}

/// Master coordinator for Phase 436: Exceptional Surface Diode & Quantum Repeater.
#[derive(Debug, Clone, PartialEq)]
pub struct ExceptionalSurfaceDiodeProcessor {
    pub surface_solver: ExceptionalSurfaceSolver,
    pub diode_solver: ChiralDiodeSolver,
    pub repeater_solver: QuantumRepeaterSolver,
}

impl Default for ExceptionalSurfaceDiodeProcessor {
    fn default() -> Self {
        Self {
            surface_solver: ExceptionalSurfaceSolver::default(),
            diode_solver: ChiralDiodeSolver::default(),
            repeater_solver: QuantumRepeaterSolver::default(),
        }
    }
}

impl ExceptionalSurfaceDiodeProcessor {
    pub fn new(
        surface_params: ExceptionalSurfaceParams,
        diode_params: ChiralDiodeParams,
        repeater_params: QuantumRepeaterParams,
    ) -> Self {
        Self {
            surface_solver: ExceptionalSurfaceSolver::new(surface_params),
            diode_solver: ChiralDiodeSolver::new(diode_params),
            repeater_solver: QuantumRepeaterSolver::new(repeater_params),
        }
    }

    /// Evaluates the comprehensive 10-point physics audit checklist.
    pub fn audit_system(&self) -> ExceptionalSurfaceAuditReport {
        let (surface_metrics, _, _) = self.surface_solver.solve_spectrum();
        let (diode_metrics, _, _) = self.diode_solver.solve_diode();
        let (repeater_metrics, _) = self
            .repeater_solver
            .solve_repeater(diode_metrics.peak_isolation_db);

        // 1. ES Coalescence
        let es_coalescence_pass = surface_metrics.es_points_count > 0
            && surface_metrics.min_splitting_mhz <= 15.0;

        // 2. Square-root branch cut scaling
        let branch_samples = vec![0.001, 0.004, 0.009, 0.016];
        let branch_results = self.surface_solver.evaluate_branch_cut_scaling(&branch_samples);
        let branch_cut_pass = if branch_results.len() >= 2 {
            let split1 = branch_results[0].1;
            let split4 = branch_results[3].1;
            // delta_k increased by 16x -> sqrt(16) = 4x
            let ratio = split4 / split1.max(1e-6);
            ratio >= 3.0 && ratio <= 5.5
        } else {
            false
        };

        // 3. Anisotropic bulk Fermi arcs
        let fermi_arcs_pass = surface_metrics.fermi_arc_points_count > 0
            && surface_metrics.max_im_energy_mhz >= 10.0;

        // 4. Group velocity asymmetry
        let group_vel_pass = surface_metrics.velocity_asymmetry_ratio >= 5.0;

        // 5. Forward insertion loss (IL <= 0.50 dB)
        let il_pass = diode_metrics.peak_insertion_loss_db <= 0.50;

        // 6. Backward isolation (ISO >= 35.0 dB)
        let iso_pass = diode_metrics.peak_isolation_db >= 35.0;

        // 7. Rectification contrast ratio (R >= 35.0 dB)
        let rect_pass = diode_metrics.rectification_contrast_db >= 35.0;

        // 8. Port return loss (RL >= 20.0 dB)
        let rl_pass = diode_metrics.return_loss_db >= 20.0;

        // 9. Quantum repeater Bell pair fidelity (F_ent >= 0.980)
        let rep_fid_pass = repeater_metrics.bell_pair_fidelity >= 0.980;

        // 10. Cryogenic noise suppression & memory coherence (T2* >= 50.0 us, ISO >= 35.0 dB)
        let cryo_pass = repeater_metrics.quantum_memory_t2_us >= 50.0
            && repeater_metrics.backscatter_noise_suppression_db >= 35.0;

        let checks = [
            es_coalescence_pass,
            branch_cut_pass,
            fermi_arcs_pass,
            group_vel_pass,
            il_pass,
            iso_pass,
            rect_pass,
            rl_pass,
            rep_fid_pass,
            cryo_pass,
        ];

        let total_score = checks.iter().filter(|&&c| c).count();
        let all_passed = total_score == 10;

        ExceptionalSurfaceAuditReport {
            exceptional_surface_coalescence_pass: es_coalescence_pass,
            square_root_branch_cut_pass: branch_cut_pass,
            anisotropic_fermi_arcs_pass: fermi_arcs_pass,
            group_velocity_asymmetry_pass: group_vel_pass,
            chiral_diode_insertion_loss_pass: il_pass,
            chiral_diode_isolation_pass: iso_pass,
            rectification_contrast_pass: rect_pass,
            return_loss_match_pass: rl_pass,
            quantum_repeater_fidelity_pass: rep_fid_pass,
            cryogenic_noise_suppression_pass: cryo_pass,
            total_score,
            all_passed,
        }
    }
}
