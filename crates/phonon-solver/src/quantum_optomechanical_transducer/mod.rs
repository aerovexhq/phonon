#![deny(unsafe_code)]

//! Cryogenic quantum optomechanical transducer and microwave-to-acoustic coherent interconnect.
//!
//! Master orchestrator and 10-point physics verification suite for micro-acoustic to
//! microwave/optical coherent quantum state conversion, ground-state sideband cooling,
//! and superconducting transmon qubit integration.

pub mod sideband_cooling;
pub mod transduction_engine;
pub mod transmon_interface;

pub use sideband_cooling::{SidebandCoolingEngine, SidebandCoolingParams};
pub use transduction_engine::{ScatteringMatrixPoint, TransductionEngine, TransductionParams};
pub use transmon_interface::{TransmonInterfaceEngine, TransmonInterfaceParams};

/// Verification item for a single physics compliance metric.
#[derive(Debug, Clone, PartialEq)]
pub struct TransducerAuditItem {
    /// Metric title.
    pub name: String,
    /// Measured physical value formatted with unit.
    pub measured: String,
    /// Roadmap required threshold.
    pub threshold: String,
    /// Verification pass/fail status.
    pub passed: bool,
    /// Physical rationale and description.
    pub details: String,
}

/// Comprehensive 10-point physics audit report.
#[derive(Debug, Clone, PartialEq)]
pub struct TransducerAuditReport {
    /// Individual evaluation items (10 criteria).
    pub items: Vec<TransducerAuditItem>,
    /// Number of passed criteria.
    pub passed_count: usize,
    /// Total criteria evaluated (10).
    pub total_count: usize,
    /// Overall physical compliance flag (10/10 PASS).
    pub is_fully_compliant: bool,
}

/// Master orchestrator for the cryogenic quantum optomechanical transducer system.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumOptomechanicalTransducer {
    pub transduction: TransductionEngine,
    pub cooling: SidebandCoolingEngine,
    pub transmon: TransmonInterfaceEngine,
}

impl Default for QuantumOptomechanicalTransducer {
    fn default() -> Self {
        Self::new(
            TransductionParams::default(),
            SidebandCoolingParams::default(),
            TransmonInterfaceParams::default(),
        )
    }
}

impl QuantumOptomechanicalTransducer {
    /// Creates a new transducer system orchestrator.
    pub fn new(
        transduction_params: TransductionParams,
        cooling_params: SidebandCoolingParams,
        transmon_params: TransmonInterfaceParams,
    ) -> Self {
        Self {
            transduction: TransductionEngine::new(transduction_params),
            cooling: SidebandCoolingEngine::new(cooling_params),
            transmon: TransmonInterfaceEngine::new(transmon_params),
        }
    }

    /// Executes the comprehensive 10-point physics audit checklist.
    pub fn audit_transducer(&self) -> TransducerAuditReport {
        let mut items = Vec::with_capacity(10);

        // 1. Phononic cavity resonance
        let omega_m = self.transduction.params.omega_m_ghz;
        let gamma_m = self.transduction.params.gamma_m_khz;
        let q_m = (omega_m * 1.0e6) / gamma_m.max(1.0e-6);
        let pass_1 = (omega_m - 4.0).abs() < 0.1 && q_m >= 1.0e6;
        items.push(TransducerAuditItem {
            name: "Phononic Crystal Acoustic Resonance & High-Q".to_string(),
            measured: format!("{:.2} GHz (Q_m = {:.2e})", omega_m, q_m),
            threshold: "4.00 GHz, Q_m >= 1.0e6".to_string(),
            passed: pass_1,
            details: "Low-loss acoustic intermediate cavity in megahertz bandgap shield".to_string(),
        });

        // 2. Electromechanical coupling
        let g_e_khz = self.transduction.compute_linearized_g_e_khz();
        let pass_2 = g_e_khz >= 200.0;
        items.push(TransducerAuditItem {
            name: "Linearized Electromechanical Coupling G_e".to_string(),
            measured: format!("{:.1} kHz", g_e_khz),
            threshold: ">= 200.0 kHz".to_string(),
            passed: pass_2,
            details: "Multi-photon parametric drive matching microwave cavity".to_string(),
        });

        // 3. Optomechanical coupling
        let g_o_khz = self.transduction.compute_linearized_g_o_khz();
        let pass_3 = g_o_khz >= 200.0;
        items.push(TransducerAuditItem {
            name: "Linearized Optomechanical Coupling G_o".to_string(),
            measured: format!("{:.1} kHz", g_o_khz),
            threshold: ">= 200.0 kHz".to_string(),
            passed: pass_3,
            details: "Telecom 1550 nm pump drive matching optical cavity linewidth".to_string(),
        });

        // 4. Cooperativity matching balance
        let c_e = self.transduction.compute_electromechanical_cooperativity();
        let c_o = self.transduction.compute_optomechanical_cooperativity();
        let match_ratio = self.transduction.compute_cooperativity_matching_ratio();
        let pass_4 = c_e >= 150.0 && c_o >= 150.0 && match_ratio <= 0.15;
        items.push(TransducerAuditItem {
            name: "Cooperativity Matching Balance (C_e vs C_o)".to_string(),
            measured: format!("C_e = {:.1}, C_o = {:.1} (Imbalance: {:.2}%)", c_e, c_o, match_ratio * 100.0),
            threshold: "C_e, C_o >= 150.0, Imbalance <= 15.0%".to_string(),
            passed: pass_4,
            details: "Impedance-matched bidirectional quantum conversion condition".to_string(),
        });

        // 5. Peak transduction efficiency
        let eta_trans = self.transduction.compute_peak_transduction_efficiency();
        let pass_5 = eta_trans >= 0.45;
        items.push(TransducerAuditItem {
            name: "Peak Bidirectional Transduction Efficiency".to_string(),
            measured: format!("{:.2}%", eta_trans * 100.0),
            threshold: ">= 45.0% (target >= 70.0%)".to_string(),
            passed: pass_5,
            details: "End-to-end microwave-to-telecom photon conversion efficiency".to_string(),
        });

        // 6. Bidirectional scattering symmetry
        let pt0 = self.transduction.compute_scattering_point(0.0);
        let pass_6 = pt0.asymmetry <= 1.0e-6;
        items.push(TransducerAuditItem {
            name: "Langevin S-Matrix Bidirectional Symmetry".to_string(),
            measured: format!("|S_oe|^2 - |S_eo|^2 = {:.2e}", pt0.asymmetry),
            threshold: "<= 1.0e-6".to_string(),
            passed: pass_6,
            details: "Reciprocal quantum coherence between microwave and optical ports".to_string(),
        });

        // 7. Transduction 3-dB bandwidth
        let bw_mhz = self.transduction.compute_transduction_bandwidth_mhz();
        let pass_7 = bw_mhz >= 0.50;
        items.push(TransducerAuditItem {
            name: "Transduction 3-dB Bandwidth Delta_f".to_string(),
            measured: format!("{:.3} MHz", bw_mhz),
            threshold: ">= 0.500 MHz".to_string(),
            passed: pass_7,
            details: "Cooperative broadening across intermediate phononic crystal".to_string(),
        });

        // 8. Effective sideband cooling
        let n_eff = self.cooling.compute_effective_occupancy();
        let p_ground = self.cooling.compute_ground_state_purity();
        let pass_8 = n_eff < 0.05 && p_ground >= 0.95;
        items.push(TransducerAuditItem {
            name: "Dynamical Ground-State Sideband Cooling".to_string(),
            measured: format!("n_eff = {:.4} phonons (P_ground = {:.2}%)", n_eff, p_ground * 100.0),
            threshold: "n_eff < 0.05 phonons, P_ground >= 95.0%".to_string(),
            passed: pass_8,
            details: "Resolved-sideband cooling suppresses thermal noise below quantum limit".to_string(),
        });

        // 9. Input-referred added quantum noise
        let n_add = self.cooling.compute_added_quantum_noise(c_e);
        let pass_9 = n_add < 0.50;
        items.push(TransducerAuditItem {
            name: "Input-Referred Added Quantum Noise N_add".to_string(),
            measured: format!("{:.3} quanta", n_add),
            threshold: "< 0.500 quanta".to_string(),
            passed: pass_9,
            details: "Sub-quanta noise performance ensuring quantum state preservation".to_string(),
        });

        // 10. Transmon coherent iSWAP & state transfer
        let g_q = self.transmon.params.coupling_g_q_mhz;
        let tau_swap = self.transmon.compute_iswap_time_ns();
        let f_state = self.transmon.compute_state_transfer_fidelity(eta_trans);
        let concurrence = self.transmon.compute_bell_pair_concurrence(eta_trans);
        let pass_10 = g_q >= 35.0 && tau_swap < 25.0 && f_state >= 0.950 && concurrence >= 0.90;
        items.push(TransducerAuditItem {
            name: "Transmon Qubit Coherent Interconnect & Bell Concurrence".to_string(),
            measured: format!("g_q = {:.1} MHz, tau = {:.2} ns, F = {:.3}, C = {:.3}", g_q, tau_swap, f_state, concurrence),
            threshold: "g_q >= 35 MHz, tau < 25 ns, F >= 0.950, C >= 0.900".to_string(),
            passed: pass_10,
            details: "Fast resonant Jaynes-Cummings iSWAP gate and Bell state generation".to_string(),
        });

        let passed_count = items.iter().filter(|i| i.passed).count();
        let total_count = items.len();
        let is_fully_compliant = passed_count == total_count;

        TransducerAuditReport {
            items,
            passed_count,
            total_count,
            is_fully_compliant,
        }
    }
}
