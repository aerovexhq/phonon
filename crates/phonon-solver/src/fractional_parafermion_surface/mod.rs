#![deny(unsafe_code)]

//! Phase 445: Quantum Metamaterial Non-Abelian Fractional Parafermion Surface-Code Lattice & Anyonic Braid Repeater.
//!
//! Master module integrating fractional parafermion zero-mode lattices, generalized
//! Z_p qudit surface-code error correction, anyonic braid repeaters, and non-Clifford
//! magic-state distillation factories with cryogenic dispersive readout.

pub mod parafermion_lattice;
pub mod surface_code_repeater;
pub mod qudit_distillation_factory;

pub use parafermion_lattice::{
    FractionalParafermionBraidTrajectoryPoint, FractionalParafermionLatticeMetrics,
    FractionalParafermionLatticeParams, FractionalParafermionLatticeSolver,
    FractionalParafermionOrder, FractionalParafermionWavepacketPoint,
};
pub use surface_code_repeater::{
    ParafermionStabilizerKind, SurfaceCodeRepeaterMetrics, SurfaceCodeRepeaterParams,
    SurfaceCodeRepeaterSolver, SurfaceLatticeNode, ThresholdScalingPoint,
};
pub use qudit_distillation_factory::{
    DistillationRoundPoint, QuditCavitySpectrumPoint, QuditDistillationMetrics,
    QuditDistillationParams, QuditDistillationSolver,
};

/// 10-Point rigorous physical audit report for the fractional parafermion system.
#[derive(Debug, Clone)]
pub struct FractionalParafermionAuditReport {
    pub topological_gap_pass: bool,
    pub braid_fidelity_pass: bool,
    pub diabatic_leakage_pass: bool,
    pub poisoning_lifetime_pass: bool,
    pub threshold_error_pass: bool,
    pub logical_error_rate_pass: bool,
    pub repeater_fidelity_pass: bool,
    pub magic_distillation_fidelity_pass: bool,
    pub acceptance_probability_pass: bool,
    pub dispersive_readout_snr_pass: bool,

    pub total_score: usize,
    pub all_passed: bool,
}

/// Unified master orchestrator for Phase 445.
#[derive(Debug, Clone)]
pub struct FractionalParafermionProcessor {
    pub lattice_solver: FractionalParafermionLatticeSolver,
    pub repeater_solver: SurfaceCodeRepeaterSolver,
    pub distillation_solver: QuditDistillationSolver,
}

impl Default for FractionalParafermionProcessor {
    fn default() -> Self {
        Self {
            lattice_solver: FractionalParafermionLatticeSolver::new(
                FractionalParafermionLatticeParams::default(),
            ),
            repeater_solver: SurfaceCodeRepeaterSolver::new(SurfaceCodeRepeaterParams::default()),
            distillation_solver: QuditDistillationSolver::new(QuditDistillationParams::default()),
        }
    }
}

impl FractionalParafermionProcessor {
    pub fn new(
        lattice_params: FractionalParafermionLatticeParams,
        repeater_params: SurfaceCodeRepeaterParams,
        distillation_params: QuditDistillationParams,
    ) -> Self {
        Self {
            lattice_solver: FractionalParafermionLatticeSolver::new(lattice_params),
            repeater_solver: SurfaceCodeRepeaterSolver::new(repeater_params),
            distillation_solver: QuditDistillationSolver::new(distillation_params),
        }
    }

    /// Executes the 10-point physics audit checklist.
    pub fn audit_system(&self) -> FractionalParafermionAuditReport {
        let lm = self.lattice_solver.evaluate_metrics();
        let rm = self.repeater_solver.evaluate_metrics();
        let dm = self.distillation_solver.evaluate_metrics();

        let topological_gap_pass = lm.topological_gap_mhz >= 1.8;
        let braid_fidelity_pass = lm.braid_fidelity >= 0.999;
        let diabatic_leakage_pass = lm.diabatic_leakage_error < 1.0e-4;
        let poisoning_lifetime_pass = lm.poisoning_lifetime_us >= 80.0;

        let threshold_error_pass = rm.threshold_error_percent >= 1.5;
        let logical_error_rate_pass = rm.logical_error_rate < 1.0e-4;
        let repeater_fidelity_pass = rm.repeater_fidelity >= 0.992;

        let magic_distillation_fidelity_pass = dm.distilled_magic_fidelity >= 0.999;
        let acceptance_probability_pass = dm.acceptance_probability_percent >= 18.0;
        let dispersive_readout_snr_pass = dm.dispersive_readout_snr_db >= 18.5;

        let items = [
            topological_gap_pass,
            braid_fidelity_pass,
            diabatic_leakage_pass,
            poisoning_lifetime_pass,
            threshold_error_pass,
            logical_error_rate_pass,
            repeater_fidelity_pass,
            magic_distillation_fidelity_pass,
            acceptance_probability_pass,
            dispersive_readout_snr_pass,
        ];

        let total_score = items.iter().filter(|&&p| p).count();
        let all_passed = total_score == 10;

        FractionalParafermionAuditReport {
            topological_gap_pass,
            braid_fidelity_pass,
            diabatic_leakage_pass,
            poisoning_lifetime_pass,
            threshold_error_pass,
            logical_error_rate_pass,
            repeater_fidelity_pass,
            magic_distillation_fidelity_pass,
            acceptance_probability_pass,
            dispersive_readout_snr_pass,
            total_score,
            all_passed,
        }
    }
}
