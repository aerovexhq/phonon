#![deny(unsafe_code)]

//! Genus-2 Parafermion Surface Code & Universal Fault-Tolerant Acoustic Processor Module.
//!
//! Master orchestrator integrating compact genus-2 Riemann surfaces (double torus),
//! non-Abelian Z_3 parafermion 9-fold degenerate ground state code spaces, transversal
//! fault-tolerant logic and adiabatic Dehn twists, homological surface code stabilizers
//! with hyperbolic Minimum-Weight Matching (MWHM), and 12x12 cryogenic routing crossbar arrays
//! with dispersive cavity readout.

pub mod cryogenic_quantum_crossbar;
pub mod fault_tolerant_logic;
pub mod genus2_stabilizer;
pub mod genus2_surface;

pub use cryogenic_quantum_crossbar::{
    CavityTransmissionPoint, CrossbarCellState, CrossbarReadoutMetrics,
    CryogenicCrossbarSolver, Genus2CrossbarCellState, QuantumCrossbarParams, CROSSBAR_DIMENSION,
    GENUS2_CROSSBAR_DIMENSION, TOTAL_CROSSBAR_CELLS, TOTAL_GENUS2_CROSSBAR_CELLS,
};
pub use fault_tolerant_logic::{
    DehnTwistTrajectoryPoint, EntangledQuditState, FaultTolerantLogicParams, Genus2LogicSolver,
    LogicalGateKind, LogicalGateMetrics,
};
pub use genus2_stabilizer::{
    AnyonicDefect, CorrectionSegment, DefectKind, Genus2StabilizerParams, Genus2StabilizerSolver,
    Genus2ThresholdCurvePoint, StabilizerSyndromeResult, ThresholdCurvePoint,
};
pub use genus2_surface::{
    Genus2DispersionPoint, Genus2SurfaceMetrics, Genus2SurfaceParams, Genus2SurfaceSolver,
    PoincareDiskPoint,
};

/// 10-point rigorous physics audit report for the genus-2 parafermion surface processor.
#[derive(Debug, Clone)]
pub struct Genus2ParafermionAuditReport {
    /// 1. Homology cycle orthogonality & modular commutator phase error <= 1.0e-5.
    pub homology_commutator_pass: bool,
    /// 2. Degenerate ground state code space dimension D = M^g = 3^2 = 9.
    pub code_space_dimension_pass: bool,
    /// 3. Topological protection bulk energy gap Delta_topo >= 2.0 MHz (>= 10.0 ueV).
    pub topological_bulk_gap_pass: bool,
    /// 4. Cryogenic quasiparticle poisoning suppression lifetime tau_poisoning >= 10.0 us.
    pub poisoning_suppression_lifetime_pass: bool,
    /// 5. Adiabatic Dehn twist diabatic leakage probability P_leak <= 1.0e-5.
    pub dehn_twist_leakage_pass: bool,
    /// 6. Inter-handle 2-qudit entangling gate (CZ_L) fidelity F >= 0.999 and concurrence C >= 0.95.
    pub inter_handle_entangling_gate_pass: bool,
    /// 7. Distance d = 3 surface code fault-tolerance threshold P_th >= 1.0% (0.010).
    pub code_distance_threshold_pass: bool,
    /// 8. Minimum-Weight Homological Matching logical error suppression P_L <= 1.0e-6.
    pub homological_matching_suppression_pass: bool,
    /// 9. Cryogenic 12x12 crossbar half-select crosstalk isolation >= 38.0 dB.
    pub crossbar_crosstalk_isolation_pass: bool,
    /// 10. Dispersive cavity parity readout SNR >= 22.0 dB & retention lifetime >= 50.0 us.
    pub dispersive_readout_snr_retention_pass: bool,
}

impl Genus2ParafermionAuditReport {
    /// Returns true if all 10 physics audit criteria evaluated to PASS.
    pub fn all_passed(&self) -> bool {
        self.homology_commutator_pass
            && self.code_space_dimension_pass
            && self.topological_bulk_gap_pass
            && self.poisoning_suppression_lifetime_pass
            && self.dehn_twist_leakage_pass
            && self.inter_handle_entangling_gate_pass
            && self.code_distance_threshold_pass
            && self.homological_matching_suppression_pass
            && self.crossbar_crosstalk_isolation_pass
            && self.dispersive_readout_snr_retention_pass
    }

    /// Returns the audit score as (passed_count, total_count).
    pub fn score(&self) -> (usize, usize) {
        let items = [
            self.homology_commutator_pass,
            self.code_space_dimension_pass,
            self.topological_bulk_gap_pass,
            self.poisoning_suppression_lifetime_pass,
            self.dehn_twist_leakage_pass,
            self.inter_handle_entangling_gate_pass,
            self.code_distance_threshold_pass,
            self.homological_matching_suppression_pass,
            self.crossbar_crosstalk_isolation_pass,
            self.dispersive_readout_snr_retention_pass,
        ];
        let passed = items.iter().filter(|&&p| p).count();
        (passed, items.len())
    }

    /// Formats a human-readable text summary of the audit checklist.
    pub fn summary(&self) -> String {
        let (passed, total) = self.score();
        format!(
            "Genus-2 Parafermion Surface Processor Audit: {}/{} PASS\n\
             1. Homology Commutator Phase Error (<= 1.0e-5): {}\n\
             2. Degenerate Ground Code Dimension (D = 9): {}\n\
             3. Bulk Topological Gap (Delta >= 2.0 MHz): {}\n\
             4. Poisoning Suppression Lifetime (tau >= 10.0 us): {}\n\
             5. Dehn Twist Diabatic Leakage (P_leak <= 1.0e-5): {}\n\
             6. Inter-Handle Gate CZ_L (F >= 0.999, C >= 0.95): {}\n\
             7. Fault-Tolerance Threshold (P_th >= 1.0%): {}\n\
             8. Homological Matching Error Suppression (P_L <= 1.0e-6): {}\n\
             9. 12x12 Crossbar Crosstalk Isolation (>= 38.0 dB): {}\n\
             10. Dispersive Readout SNR & Retention (SNR >= 22 dB, tau >= 50 us): {}",
            passed,
            total,
            self.homology_commutator_pass,
            self.code_space_dimension_pass,
            self.topological_bulk_gap_pass,
            self.poisoning_suppression_lifetime_pass,
            self.dehn_twist_leakage_pass,
            self.inter_handle_entangling_gate_pass,
            self.code_distance_threshold_pass,
            self.homological_matching_suppression_pass,
            self.crossbar_crosstalk_isolation_pass,
            self.dispersive_readout_snr_retention_pass,
        )
    }
}

/// Master orchestrator for the genus-2 non-Abelian parafermion acoustic processor.
#[derive(Debug, Clone)]
pub struct Genus2ParafermionProcessor {
    surface_solver: Genus2SurfaceSolver,
    logic_solver: Genus2LogicSolver,
    stabilizer_solver: Genus2StabilizerSolver,
    crossbar_solver: CryogenicCrossbarSolver,
}

impl Default for Genus2ParafermionProcessor {
    fn default() -> Self {
        Self {
            surface_solver: Genus2SurfaceSolver::new(Genus2SurfaceParams::default()),
            logic_solver: Genus2LogicSolver::new(FaultTolerantLogicParams::default()),
            stabilizer_solver: Genus2StabilizerSolver::new(Genus2StabilizerParams::default()),
            crossbar_solver: CryogenicCrossbarSolver::new(QuantumCrossbarParams::default()),
        }
    }
}

impl Genus2ParafermionProcessor {
    /// Constructs a processor with explicit solvers.
    pub fn new(
        surface_params: Genus2SurfaceParams,
        logic_params: FaultTolerantLogicParams,
        stabilizer_params: Genus2StabilizerParams,
        crossbar_params: QuantumCrossbarParams,
    ) -> Self {
        Self {
            surface_solver: Genus2SurfaceSolver::new(surface_params),
            logic_solver: Genus2LogicSolver::new(logic_params),
            stabilizer_solver: Genus2StabilizerSolver::new(stabilizer_params),
            crossbar_solver: CryogenicCrossbarSolver::new(crossbar_params),
        }
    }

    /// Access the underlying surface solver.
    pub fn surface(&self) -> &Genus2SurfaceSolver {
        &self.surface_solver
    }

    /// Access the underlying logic solver.
    pub fn logic(&self) -> &Genus2LogicSolver {
        &self.logic_solver
    }

    /// Access the underlying stabilizer solver.
    pub fn stabilizer(&self) -> &Genus2StabilizerSolver {
        &self.stabilizer_solver
    }

    /// Access the underlying crossbar solver.
    pub fn crossbar(&self) -> &CryogenicCrossbarSolver {
        &self.crossbar_solver
    }

    /// Evaluates the comprehensive 10-point physics audit checklist.
    pub fn evaluate_audit(&self) -> Genus2ParafermionAuditReport {
        let surface_metrics = self.surface_solver.evaluate_metrics();
        let cz_metrics = self.logic_solver.evaluate_gate(LogicalGateKind::ControlledZ);
        let twist_metrics = self.logic_solver.evaluate_gate(LogicalGateKind::DehnTwistAlpha1);
        let syndrome_result = self.stabilizer_solver.inject_and_decode_errors(4);
        let crossbar_metrics = self.crossbar_solver.evaluate_readout_metrics(0, 0);

        // 1. Commutator phase error <= 1.0e-5
        let pass1 = surface_metrics.commutator_phase_error <= 1.0e-5;

        // 2. Degenerate code space dimension == 9
        let pass2 = surface_metrics.code_space_dimension == 9;

        // 3. Bulk topological gap >= 2.0 MHz (>= 10.0 ueV)
        let pass3 = self.surface_solver.params().topological_gap_mhz >= 2.0
            && surface_metrics.protection_gap_uev >= 10.0;

        // 4. Poisoning suppression lifetime >= 10.0 us
        let pass4 = surface_metrics.poisoning_suppression_lifetime_us >= 10.0;

        // 5. Dehn twist leakage <= 1.0e-5
        let pass5 = twist_metrics.diabatic_leakage_prob <= 1.0e-5;

        // 6. Inter-handle CZ_L gate: F >= 0.999 and C >= 0.95
        let pass6 = cz_metrics.process_fidelity >= 0.999 && cz_metrics.concurrence >= 0.95;

        // 7. Distance d = 3 threshold >= 0.010 (1.0%)
        let pass7 = self.stabilizer_solver.params().threshold_error_rate >= 0.010;

        // 8. Homological matching error suppression P_L <= 1.0e-6
        let pass8 = syndrome_result.logical_error_rate <= 1.0e-6;

        // 9. Half-select crosstalk isolation >= 38.0 dB
        let pass9 = crossbar_metrics.crosstalk_isolation_db >= 38.0;

        // 10. Dispersive readout SNR >= 22.0 dB & retention >= 50.0 us
        let pass10 = crossbar_metrics.readout_snr_db >= 22.0
            && crossbar_metrics.retention_lifetime_us >= 50.0;

        Genus2ParafermionAuditReport {
            homology_commutator_pass: pass1,
            code_space_dimension_pass: pass2,
            topological_bulk_gap_pass: pass3,
            poisoning_suppression_lifetime_pass: pass4,
            dehn_twist_leakage_pass: pass5,
            inter_handle_entangling_gate_pass: pass6,
            code_distance_threshold_pass: pass7,
            homological_matching_suppression_pass: pass8,
            crossbar_crosstalk_isolation_pass: pass9,
            dispersive_readout_snr_retention_pass: pass10,
        }
    }
}
