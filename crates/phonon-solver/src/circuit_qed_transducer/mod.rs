#![deny(unsafe_code)]

//! Phase 435: Phonon Studio Topological Higher-Order Acoustic Superconducting
//! Circuit QED Quantum Transducer & Multi-Qubit Crossbar.
//!
//! Master orchestrator and 10-point physics audit checklist.

pub mod acoustic_corner_mode;
pub mod multi_qubit_crossbar;
pub mod transmon_circuit_qed;

pub use acoustic_corner_mode::{
    AcousticCornerParams, AcousticCornerSolver, CornerModeMetrics, CornerSpatialPoint,
};
pub use multi_qubit_crossbar::{
    CrossbarMetrics, CrossbarParams, MultiQubitCrossbarSolver, RoutingMatrixElement,
    TwoQubitGateDynamics,
};
pub use transmon_circuit_qed::{
    CircuitQedMetrics, RabiOscillationPoint, TransmonCircuitQedSolver, TransmonParams,
};

pub type CircuitQedCrossbarParams = CrossbarParams;
pub type CircuitQedCrossbarMetrics = CrossbarMetrics;

/// 10-point physics audit report for Phase 435.
#[derive(Debug, Clone, PartialEq)]
pub struct CircuitQedAuditReport {
    /// 1. Topological corner state localization (confinement ratio >= 85.0%).
    pub corner_localization_pass: bool,
    /// 2. Piezoelectric electromechanical transduction coupling rate (g_trans/2pi >= 10.0 MHz).
    pub piezoelectric_transduction_pass: bool,
    /// 3. Transmon negative anharmonicity (|alpha| >= 200.0 MHz, alpha < 0).
    pub transmon_anharmonicity_pass: bool,
    /// 4. On-resonance Jaynes-Cummings vacuum Rabi splitting (2g/2pi >= 20.0 MHz).
    pub vacuum_rabi_splitting_pass: bool,
    /// 5. Dispersive frequency shift in detuned regime (chi/2pi >= 1.0 MHz).
    pub dispersive_shift_pass: bool,
    /// 6. Coherent transmon-to-phonon quantum state transfer fidelity (F_transfer >= 0.990).
    pub state_transfer_fidelity_pass: bool,
    /// 7. Multi-qubit acoustic crossbar crosstalk isolation (>= 35.0 dB).
    pub crossbar_crosstalk_isolation_pass: bool,
    /// 8. Virtual-phonon-mediated two-qubit entangling gate process fidelity (F_gate >= 0.985).
    pub two_qubit_gate_fidelity_pass: bool,
    /// 9. Dispersive QND quantum readout Signal-to-Noise Ratio (SNR >= 15.0 dB).
    pub dispersive_qnd_readout_pass: bool,
    /// 10. Cryogenic coherence preservation (T2* / tau_swap >= 1000.0).
    pub cryogenic_coherence_pass: bool,
    /// Total score out of 10.
    pub total_score: usize,
    /// Whether all 10 criteria passed.
    pub all_passed: bool,
}

/// Master orchestrator for Phase 435: Topological Circuit QED Transducer & Multi-Qubit Crossbar.
#[derive(Debug, Clone, PartialEq)]
pub struct CircuitQedTransducerProcessor {
    pub corner_solver: AcousticCornerSolver,
    pub transmon_solver: TransmonCircuitQedSolver,
    pub crossbar_solver: MultiQubitCrossbarSolver,
}

impl Default for CircuitQedTransducerProcessor {
    fn default() -> Self {
        Self {
            corner_solver: AcousticCornerSolver::default(),
            transmon_solver: TransmonCircuitQedSolver::default(),
            crossbar_solver: MultiQubitCrossbarSolver::default(),
        }
    }
}

impl CircuitQedTransducerProcessor {
    /// Creates a new processor with custom configurations.
    pub fn new(
        corner_params: AcousticCornerParams,
        transmon_params: TransmonParams,
        crossbar_params: CrossbarParams,
    ) -> Self {
        Self {
            corner_solver: AcousticCornerSolver::new(corner_params),
            transmon_solver: TransmonCircuitQedSolver::new(transmon_params),
            crossbar_solver: MultiQubitCrossbarSolver::new(crossbar_params),
        }
    }

    /// Evaluates the 10-point physics audit checklist.
    pub fn audit_transducer(&self) -> CircuitQedAuditReport {
        let (corner_metrics, _) = self.corner_solver.solve_corner_mode();
        let (qed_metrics, _) = self
            .transmon_solver
            .solve_dynamics(corner_metrics.transduction_rate_mhz);
        let crossbar_metrics = self.crossbar_solver.solve_crossbar(
            corner_metrics.transduction_rate_mhz,
            self.transmon_solver.params.detuning_mhz,
            self.transmon_solver.params.t2_us,
        );

        // 1. Corner mode localization (>= 85.0%)
        let corner_localization_pass =
            corner_metrics.confinement_ratio >= 0.85 && corner_metrics.is_topological;

        // 2. Piezoelectric electromechanical transduction rate (>= 10.0 MHz)
        let piezoelectric_transduction_pass = corner_metrics.transduction_rate_mhz >= 10.0;

        // 3. Transmon negative anharmonicity (|alpha| >= 200.0 MHz, alpha < 0)
        let transmon_anharmonicity_pass =
            qed_metrics.anharmonicity_mhz.abs() >= 200.0 && qed_metrics.anharmonicity_mhz < 0.0;

        // 4. Vacuum Rabi splitting (>= 20.0 MHz)
        let vacuum_rabi_splitting_pass = qed_metrics.vacuum_rabi_mhz >= 20.0;

        // 5. Dispersive shift (>= 1.0 MHz)
        let dispersive_shift_pass = qed_metrics.dispersive_shift_mhz >= 1.0;

        // 6. Quantum state transfer fidelity (>= 0.990)
        let state_transfer_fidelity_pass = qed_metrics.state_transfer_fidelity >= 0.990;

        // 7. Crossbar crosstalk isolation (>= 35.0 dB)
        let crossbar_crosstalk_isolation_pass = crossbar_metrics.crosstalk_isolation_db >= 35.0;

        // 8. Two-qubit entangling gate process fidelity (>= 0.985)
        let two_qubit_gate_fidelity_pass =
            crossbar_metrics.two_qubit_dynamics.gate_fidelity >= 0.985;

        // 9. Dispersive QND readout SNR (>= 15.0 dB)
        let dispersive_qnd_readout_pass = qed_metrics.qnd_readout_snr_db >= 15.0;

        // 10. Cryogenic coherence preservation (T2* / tau_swap >= 1000.0)
        let cryogenic_coherence_pass = qed_metrics.coherence_ratio >= 1000.0;

        let checks = [
            corner_localization_pass,
            piezoelectric_transduction_pass,
            transmon_anharmonicity_pass,
            vacuum_rabi_splitting_pass,
            dispersive_shift_pass,
            state_transfer_fidelity_pass,
            crossbar_crosstalk_isolation_pass,
            two_qubit_gate_fidelity_pass,
            dispersive_qnd_readout_pass,
            cryogenic_coherence_pass,
        ];

        let total_score = checks.iter().filter(|&&c| c).count();
        let all_passed = total_score == 10;

        CircuitQedAuditReport {
            corner_localization_pass,
            piezoelectric_transduction_pass,
            transmon_anharmonicity_pass,
            vacuum_rabi_splitting_pass,
            dispersive_shift_pass,
            state_transfer_fidelity_pass,
            crossbar_crosstalk_isolation_pass,
            two_qubit_gate_fidelity_pass,
            dispersive_qnd_readout_pass,
            cryogenic_coherence_pass,
            total_score,
            all_passed,
        }
    }
}
