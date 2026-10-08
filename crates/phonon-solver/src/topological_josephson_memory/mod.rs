#![deny(unsafe_code)]

//! Topological Josephson Memory & Quantum Phase-Slip Crossbar Module.
//!
//! Master orchestrator integrating anomalous phi_0-junction supercurrents,
//! sub-150 ps chiral spin-orbit torque write dynamics, coherent quantum phase-slip
//! Bloch oscillations, and 8x8 cryogenic crossbar arrays with dispersive cavity readout.

pub mod chiral_sot;
pub mod josephson_cpr;
pub mod quantum_phase_slip;
pub mod superconducting_crossbar;

pub use chiral_sot::{ChiralSotMetrics, ChiralSotParams, ChiralSotSolver, SotTrajectoryPoint};
pub use josephson_cpr::{
    CprCurvePoint, JosephsonCprMetrics, TopologicalJosephsonParams, TopologicalJosephsonSolver,
};
pub use quantum_phase_slip::{
    QpsIvCurvePoint, QpsRabiPoint, QuantumPhaseSlipMetrics, QuantumPhaseSlipParams,
    QuantumPhaseSlipSolver,
};
pub use superconducting_crossbar::{
    CrossbarCellState, CrossbarReadoutSpectrumPoint, SuperconductingCrossbarMetrics,
    SuperconductingCrossbarParams, SuperconductingCrossbarSolver, CROSSBAR_DIMENSION,
    TOTAL_MEMORY_CELLS,
};

/// 10-point rigorous physics audit report for topological Josephson memory.
#[derive(Debug, Clone)]
pub struct TopologicalJosephsonAuditReport {
    /// 1. Anomalous ground phase phi_0 offset and double-well energy barrier Delta U >= 40.0 ueV.
    pub anomalous_phase_barrier_pass: bool,
    /// 2. Fractional 4pi-periodic Majorana supercurrent ratio I_c,4pi / I_c1 >= 0.25.
    pub fractional_4pi_current_pass: bool,
    /// 3. Non-volatile retention lifetime tau_ret >= 1.0 us at base temperature.
    pub retention_lifetime_pass: bool,
    /// 4. Chiral SOT switching pulse latency tau_switch <= 150.0 ps.
    pub sot_switching_speed_pass: bool,
    /// 5. Ultra-low switching energy dissipation E_switch <= 1.0 aJ (attojoules).
    pub switching_energy_pass: bool,
    /// 6. Cryogenic SOT switching error rate P_err <= 1.0e-5.
    pub switching_error_rate_pass: bool,
    /// 7. Coherent quantum phase-slip tunneling amplitude E_QPS >= 1.0 GHz.
    pub qps_tunneling_pass: bool,
    /// 8. Bloch voltage oscillation duality & Rabi single-qubit gate fidelity F >= 0.995.
    pub bloch_rabi_duality_pass: bool,
    /// 9. Dispersive cavity readout SNR >= 20.0 dB.
    pub dispersive_readout_snr_pass: bool,
    /// 10. Half-select crosstalk isolation >= 35.0 dB with cryogenic dephasing T_2* >= 12.0 us.
    pub crossbar_crosstalk_coherence_pass: bool,
}

impl TopologicalJosephsonAuditReport {
    /// Returns true if all 10 physics audit criteria evaluated to PASS.
    pub fn all_passed(&self) -> bool {
        self.anomalous_phase_barrier_pass
            && self.fractional_4pi_current_pass
            && self.retention_lifetime_pass
            && self.sot_switching_speed_pass
            && self.switching_energy_pass
            && self.switching_error_rate_pass
            && self.qps_tunneling_pass
            && self.bloch_rabi_duality_pass
            && self.dispersive_readout_snr_pass
            && self.crossbar_crosstalk_coherence_pass
    }

    /// Returns the audit score as (passed_count, total_count).
    pub fn score(&self) -> (usize, usize) {
        let items = [
            self.anomalous_phase_barrier_pass,
            self.fractional_4pi_current_pass,
            self.retention_lifetime_pass,
            self.sot_switching_speed_pass,
            self.switching_energy_pass,
            self.switching_error_rate_pass,
            self.qps_tunneling_pass,
            self.bloch_rabi_duality_pass,
            self.dispersive_readout_snr_pass,
            self.crossbar_crosstalk_coherence_pass,
        ];
        let passed = items.iter().filter(|&&p| p).count();
        (passed, items.len())
    }

    /// Formats a human-readable text summary of the audit checklist.
    pub fn summary(&self) -> String {
        let (passed, total) = self.score();
        format!(
            "Topological Josephson Memory Audit: {}/{} PASS\n\
             1. Anomalous Phase Shift & Barrier (Delta U >= 40 ueV): {}\n\
             2. Fractional 4pi Supercurrent (Ratio >= 0.25): {}\n\
             3. Non-Volatile Retention Lifetime (tau >= 1.0 us): {}\n\
             4. Chiral SOT Switching Speed (tau <= 150 ps): {}\n\
             5. Ultra-Low Dissipation (E <= 1.0 aJ): {}\n\
             6. Switching Error Rate (P_err <= 1.0e-5): {}\n\
             7. Coherent QPS Tunneling (E_QPS >= 1.0 GHz): {}\n\
             8. Bloch Voltage & Rabi Gate Fidelity (F >= 0.995): {}\n\
             9. Dispersive Readout SNR (SNR >= 20.0 dB): {}\n\
             10. Crossbar Half-Select Crosstalk & Coherence (Iso >= 35 dB, T2* >= 12 us): {}",
            passed,
            total,
            if self.anomalous_phase_barrier_pass { "PASS" } else { "FAIL" },
            if self.fractional_4pi_current_pass { "PASS" } else { "FAIL" },
            if self.retention_lifetime_pass { "PASS" } else { "FAIL" },
            if self.sot_switching_speed_pass { "PASS" } else { "FAIL" },
            if self.switching_energy_pass { "PASS" } else { "FAIL" },
            if self.switching_error_rate_pass { "PASS" } else { "FAIL" },
            if self.qps_tunneling_pass { "PASS" } else { "FAIL" },
            if self.bloch_rabi_duality_pass { "PASS" } else { "FAIL" },
            if self.dispersive_readout_snr_pass { "PASS" } else { "FAIL" },
            if self.crossbar_crosstalk_coherence_pass { "PASS" } else { "FAIL" }
        )
    }
}

/// Master coordinator for topological Josephson memory operations and audit.
#[derive(Debug, Clone)]
pub struct TopologicalJosephsonMemoryProcessor {
    cpr_solver: TopologicalJosephsonSolver,
    sot_solver: ChiralSotSolver,
    qps_solver: QuantumPhaseSlipSolver,
    crossbar_solver: SuperconductingCrossbarSolver,
}

impl Default for TopologicalJosephsonMemoryProcessor {
    fn default() -> Self {
        Self {
            cpr_solver: TopologicalJosephsonSolver::new(TopologicalJosephsonParams::default()),
            sot_solver: ChiralSotSolver::new(ChiralSotParams::default()),
            qps_solver: QuantumPhaseSlipSolver::new(QuantumPhaseSlipParams::default()),
            crossbar_solver: SuperconductingCrossbarSolver::new(SuperconductingCrossbarParams::default()),
        }
    }
}

impl TopologicalJosephsonMemoryProcessor {
    /// Constructs a new processor with configured solvers.
    pub fn new(
        cpr_params: TopologicalJosephsonParams,
        sot_params: ChiralSotParams,
        qps_params: QuantumPhaseSlipParams,
        crossbar_params: SuperconductingCrossbarParams,
    ) -> Self {
        Self {
            cpr_solver: TopologicalJosephsonSolver::new(cpr_params),
            sot_solver: ChiralSotSolver::new(sot_params),
            qps_solver: QuantumPhaseSlipSolver::new(qps_params),
            crossbar_solver: SuperconductingCrossbarSolver::new(crossbar_params),
        }
    }

    /// Accessor for the CPR solver.
    pub fn cpr_solver(&self) -> &TopologicalJosephsonSolver {
        &self.cpr_solver
    }

    /// Accessor for the SOT solver.
    pub fn sot_solver(&self) -> &ChiralSotSolver {
        &self.sot_solver
    }

    /// Accessor for the QPS solver.
    pub fn qps_solver(&self) -> &QuantumPhaseSlipSolver {
        &self.qps_solver
    }

    /// Accessor for the crossbar solver.
    pub fn crossbar_solver(&self) -> &SuperconductingCrossbarSolver {
        &self.crossbar_solver
    }

    /// Mutable accessor for the crossbar solver.
    pub fn crossbar_solver_mut(&mut self) -> &mut SuperconductingCrossbarSolver {
        &mut self.crossbar_solver
    }

    /// Executes a memory cycle: writes bit to cell, simulates SOT write pulse, and verifies readout.
    pub fn execute_memory_write(&mut self, row: usize, col: usize, bit_value: u8) -> bool {
        self.crossbar_solver.write_cell(row, col, bit_value)
    }

    /// Evaluates the 10-point rigorous physics audit checklist.
    pub fn run_physics_audit(&self) -> TopologicalJosephsonAuditReport {
        let cpr_metrics = self.cpr_solver.evaluate_metrics();
        let sot_metrics = self.sot_solver.evaluate_metrics();
        let qps_metrics = self.qps_solver.evaluate_metrics();
        let crossbar_metrics = self.crossbar_solver.evaluate_metrics();

        // 1. Anomalous Phase Shift & Barrier (Delta U >= 40 ueV)
        let phase_pass = cpr_metrics.anomalous_phase_phi0_rad > 0.3 * std::f64::consts::PI
            && cpr_metrics.memory_energy_barrier_uev >= 40.0;

        // 2. Fractional 4pi Supercurrent (Ratio >= 0.25)
        let frac_ratio = self.cpr_solver.params().fractional_current_ratio;
        let frac_pass = frac_ratio >= 0.25;

        // 3. Non-Volatile Retention Lifetime (tau >= 1.0 us)
        let ret_pass = cpr_metrics.retention_lifetime_us >= 1.0;

        // 4. Chiral SOT Switching Speed (tau <= 150 ps)
        let sot_speed_pass = sot_metrics.switching_time_ps <= 150.0;

        // 5. Ultra-Low Dissipation (E <= 1.0 aJ)
        let sot_energy_pass = sot_metrics.switching_energy_aj <= 1.0;

        // 6. Switching Error Rate (P_err <= 1.0e-5)
        let sot_err_pass = sot_metrics.switching_error_rate <= 1.0e-5;

        // 7. Coherent QPS Tunneling (E_QPS >= 1.0 GHz)
        let qps_pass = qps_metrics.qps_amplitude_ghz >= 1.0;

        // 8. Bloch Voltage & Rabi Gate Fidelity (F >= 0.995)
        let rabi_pass = qps_metrics.rabi_gate_fidelity >= 0.995 && qps_metrics.bloch_voltage_uv > 0.0;

        // 9. Dispersive Readout SNR (SNR >= 20.0 dB)
        let snr_pass = crossbar_metrics.readout_snr_db >= 20.0;

        // 10. Crossbar Half-Select Crosstalk & Coherence (Iso >= 35 dB, T2* >= 12 us)
        let crossbar_pass = crossbar_metrics.half_select_isolation_db >= 35.0
            && crossbar_metrics.dephasing_time_t2_star_us >= 12.0;

        TopologicalJosephsonAuditReport {
            anomalous_phase_barrier_pass: phase_pass,
            fractional_4pi_current_pass: frac_pass,
            retention_lifetime_pass: ret_pass,
            sot_switching_speed_pass: sot_speed_pass,
            switching_energy_pass: sot_energy_pass,
            switching_error_rate_pass: sot_err_pass,
            qps_tunneling_pass: qps_pass,
            bloch_rabi_duality_pass: rabi_pass,
            dispersive_readout_snr_pass: snr_pass,
            crossbar_crosstalk_coherence_pass: crossbar_pass,
        }
    }
}
