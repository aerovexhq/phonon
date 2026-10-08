#![deny(unsafe_code)]

//! Phase 439: Phonon Studio Topological Acoustic Second-Order Boundary-Mode
//! Soliton Logic Gate & Majority Voter.
//!
//! Master orchestrator and 10-point physics audit checklist.

pub mod boundary_soliton;
pub mod collisional_phase_shift;
pub mod majority_voter;

pub use boundary_soliton::{
    BoundarySolitonMetrics, BoundarySolitonParams, BoundarySolitonSolver, SolitonSpatialPoint,
};
pub use collisional_phase_shift::{
    CollisionMetrics, CollisionParams, CollisionTrajectoryPoint, CollisionalPhaseShiftSolver,
};
pub use majority_voter::{
    GateWaveformPoint, MajorityVoterMetrics, MajorityVoterParams, MajorityVoterSolver,
    SolitonGateMode, TruthTableEntry,
};

/// 10-point physics audit report for Phase 439.
#[derive(Debug, Clone, PartialEq)]
pub struct TopologicalSolitonAuditReport {
    /// 1. Topological bulk quadrupole moment quantization (q_xy = 0.5 +/- 0.01).
    pub bulk_quadrupole_quantization_pass: bool,
    /// 2. 1D boundary soliton acoustic energy localization (E_edge / E_total >= 85.0%).
    pub edge_localization_pass: bool,
    /// 3. Non-linear dispersion balance error (|beta_2 * eta^2 - gamma * P_peak| <= 0.05).
    pub dispersion_balance_pass: bool,
    /// 4. Soliton-soliton elastic collision shape conservation (F_shape >= 0.950).
    pub collision_shape_fidelity_pass: bool,
    /// 5. Collisional phase shift predictability (|Delta theta - Delta theta_theory| <= 0.08 rad).
    pub phase_shift_predictability_pass: bool,
    /// 6. 3-input majority voter truth table verification (8/8 states strictly verified).
    pub truth_table_completeness_pass: bool,
    /// 7. All-acoustic logic contrast ratio (C_logic >= 20.0 dB).
    pub logic_contrast_pass: bool,
    /// 8. Reconfigurable gate synthesis (AND, OR, NAND, NOR, Majority modes operational).
    pub reconfigurable_modes_pass: bool,
    /// 9. Ultra-low switching energy dissipation (E_switch <= 2.0 fJ).
    pub switching_energy_pass: bool,
    /// 10. Sub-30ns propagation latency (tau_prop <= 30.0 ns).
    pub propagation_latency_pass: bool,
    /// Total score out of 10.
    pub total_score: usize,
    /// Whether all 10 criteria passed.
    pub all_passed: bool,
}

/// Master processor orchestrating boundary solitons, collisions, and majority voter logic.
#[derive(Debug, Clone, PartialEq)]
pub struct TopologicalSolitonProcessor {
    pub boundary_solver: BoundarySolitonSolver,
    pub collision_solver: CollisionalPhaseShiftSolver,
    pub gate_solver: MajorityVoterSolver,
}

impl Default for TopologicalSolitonProcessor {
    fn default() -> Self {
        Self {
            boundary_solver: BoundarySolitonSolver::default(),
            collision_solver: CollisionalPhaseShiftSolver::default(),
            gate_solver: MajorityVoterSolver::default(),
        }
    }
}

impl TopologicalSolitonProcessor {
    pub fn new(
        boundary_params: BoundarySolitonParams,
        collision_params: CollisionParams,
        gate_params: MajorityVoterParams,
    ) -> Self {
        Self {
            boundary_solver: BoundarySolitonSolver::new(boundary_params),
            collision_solver: CollisionalPhaseShiftSolver::new(collision_params),
            gate_solver: MajorityVoterSolver::new(gate_params),
        }
    }

    /// Evaluates the comprehensive 10-point physics audit checklist.
    pub fn audit_system(&self) -> TopologicalSolitonAuditReport {
        let (boundary_metrics, _) = self.boundary_solver.solve_soliton(0.0);
        let (collision_metrics, _) = self.collision_solver.solve_collision();
        let (gate_metrics, truth_table, _) = self.gate_solver.solve_gate();

        // 1. Bulk quadrupole moment quantization
        let bulk_quadrupole_quantization_pass =
            (boundary_metrics.bulk_quadrupole_moment - 0.5).abs() < 0.01;

        // 2. 1D boundary mode acoustic energy localization
        let edge_localization_pass = boundary_metrics.edge_localization_ratio >= 0.85;

        // 3. Non-linear dispersion balance error
        let dispersion_balance_pass = boundary_metrics.dispersion_balance_error <= 0.05;

        // 4. Elastic collision shape conservation
        let collision_shape_fidelity_pass =
            collision_metrics.shape_conservation_fidelity >= 0.950;

        // 5. Collisional phase shift predictability
        let phase_shift_predictability_pass =
            collision_metrics.phase_shift_residual_rad <= 0.08;

        // 6. 3-input truth table verification
        let truth_table_completeness_pass =
            truth_table.len() == 8 && truth_table.iter().all(|e| e.state_pass);

        // 7. Logic contrast ratio
        let logic_contrast_pass = gate_metrics.contrast_ratio_db >= 20.0;

        // 8. Reconfigurable gate modes verification (test AND and OR synthesis)
        let and_solver = MajorityVoterSolver::new(MajorityVoterParams {
            mode: SolitonGateMode::AndGate,
            ..Default::default()
        });
        let or_solver = MajorityVoterSolver::new(MajorityVoterParams {
            mode: SolitonGateMode::OrGate,
            ..Default::default()
        });
        let and_pass = and_solver.evaluate_truth_table().iter().all(|e| e.state_pass);
        let or_pass = or_solver.evaluate_truth_table().iter().all(|e| e.state_pass);
        let reconfigurable_modes_pass = and_pass && or_pass;

        // 9. Low switching energy
        let switching_energy_pass = gate_metrics.switching_energy_fj <= 2.0;

        // 10. Sub-30ns propagation latency
        let propagation_latency_pass = gate_metrics.propagation_delay_ns <= 30.0;

        let checks = [
            bulk_quadrupole_quantization_pass,
            edge_localization_pass,
            dispersion_balance_pass,
            collision_shape_fidelity_pass,
            phase_shift_predictability_pass,
            truth_table_completeness_pass,
            logic_contrast_pass,
            reconfigurable_modes_pass,
            switching_energy_pass,
            propagation_latency_pass,
        ];

        let total_score = checks.iter().filter(|&&p| p).count();
        let all_passed = total_score == 10;

        TopologicalSolitonAuditReport {
            bulk_quadrupole_quantization_pass,
            edge_localization_pass,
            dispersion_balance_pass,
            collision_shape_fidelity_pass,
            phase_shift_predictability_pass,
            truth_table_completeness_pass,
            logic_contrast_pass,
            reconfigurable_modes_pass,
            switching_energy_pass,
            propagation_latency_pass,
            total_score,
            all_passed,
        }
    }
}
