#![deny(unsafe_code)]

//! Phase 459: Phonon Studio Chiral Topological Metamaterial Photonic-Phononic Qubit Transducer & Quantum Network Repeater Node.
//!
//! Master coordinator and physics invariant audit engine integrating:
//! 1. Piezo-optomechanical bidirectional quantum frequency conversion between microwave qubits (3.5 - 5.0 GHz)
//!    and telecom optical photons (193.4 THz / 1550 nm) with efficiency >= 15.0%, bandwidth >= 2.0 MHz,
//!    single-photon coupling g_0 >= 750 kHz, and added noise <= 0.10 quanta at 20 mK.
//! 2. 4-terminal non-reciprocal chiral circulator routing optical and acoustic qubits with forward loss <= 0.40 dB,
//!    backward isolation >= 38.0 dB, return loss >= 22.0 dB, and corner defect retention >= 95.0%.
//! 3. Entanglement swapping quantum network repeater node achieving swapped Bell fidelity >= 92.0%,
//!    concurrence >= 0.88, pair rate >= 1.0e3 pairs/s, and repeater rate gain G_rep >= 2.0x over direct fiber.

pub mod entanglement_swapping_repeater;
pub mod non_reciprocal_chiral_router;
pub mod piezo_optomechanical_transducer;

pub use entanglement_swapping_repeater::{
    EntanglementRepeaterMetrics, EntanglementRepeaterParams, EntanglementRepeaterSolver,
    RepeaterDistanceSweepPoint,
};
pub use non_reciprocal_chiral_router::{
    ChiralRouterMetrics, ChiralRouterParams, ChiralRouterSolver, ChiralRouterSpectrumPoint,
};
pub use piezo_optomechanical_transducer::{
    PiezoOptomechanicalMetrics, PiezoOptomechanicalParams, PiezoOptomechanicalSolver,
    TransductionPowerSweepPoint,
};

/// 10-point rigorous physics audit report for Phase 459.
#[derive(Debug, Clone)]
pub struct ChiralTransducerRepeaterAuditReport {
    /// 1. Bidirectional microwave-to-optical conversion efficiency eta >= 15.0%.
    pub bidirectional_transduction_efficiency: bool,
    /// 2. 3-dB transduction bandwidth Delta_f >= 2.0 MHz.
    pub transduction_bandwidth: bool,
    /// 3. Input-referred added thermal noise n_add <= 0.10 quanta.
    pub added_thermal_noise: bool,
    /// 4. Single-photon optomechanical coupling rate g_0 >= 750.0 kHz.
    pub optomechanical_coupling_rate: bool,
    /// 5. Chiral router forward transmission insertion loss IL <= 0.40 dB.
    pub chiral_router_forward_loss: bool,
    /// 6. Chiral router backward non-reciprocal isolation ISO >= 38.0 dB.
    pub chiral_router_isolation: bool,
    /// 7. Input port return loss RL >= 22.0 dB.
    pub port_return_loss: bool,
    /// 8. Swapped entangled Bell state fidelity F_swap >= 92.0%.
    pub swapped_state_fidelity: bool,
    /// 9. Swapped bipartite state concurrence C >= 0.88.
    pub swapped_state_concurrence: bool,
    /// 10. Quantum repeater rate advantage gain G_rep >= 2.0x over direct fiber.
    pub quantum_repeater_rate_gain: bool,
}

impl ChiralTransducerRepeaterAuditReport {
    /// Returns the (passed_count, total_count) score.
    pub fn score(&self) -> (usize, usize) {
        let items = [
            self.bidirectional_transduction_efficiency,
            self.transduction_bandwidth,
            self.added_thermal_noise,
            self.optomechanical_coupling_rate,
            self.chiral_router_forward_loss,
            self.chiral_router_isolation,
            self.port_return_loss,
            self.swapped_state_fidelity,
            self.swapped_state_concurrence,
            self.quantum_repeater_rate_gain,
        ];
        let passed = items.iter().filter(|&&v| v).count();
        (passed, items.len())
    }

    /// Returns true if all 10 physics audit criteria scored PASS.
    pub fn is_pass(&self) -> bool {
        let (passed, total) = self.score();
        passed == total
    }
}

/// Master coordinator processor for Chiral Transducer & Quantum Repeater Node (Phase 459).
#[derive(Debug, Clone)]
pub struct ChiralTransducerRepeaterProcessor {
    pub transducer_params: PiezoOptomechanicalParams,
    pub router_params: ChiralRouterParams,
    pub repeater_params: EntanglementRepeaterParams,
}

impl ChiralTransducerRepeaterProcessor {
    /// Creates a new master processor instance.
    pub fn new(
        transducer_params: PiezoOptomechanicalParams,
        router_params: ChiralRouterParams,
        repeater_params: EntanglementRepeaterParams,
    ) -> Self {
        Self {
            transducer_params,
            router_params,
            repeater_params,
        }
    }

    /// Runs a comprehensive 10-point physics audit across all subsystems.
    pub fn audit_system(&self) -> ChiralTransducerRepeaterAuditReport {
        let trans_solver = PiezoOptomechanicalSolver::new(self.transducer_params.clone());
        let trans_m = trans_solver.evaluate_metrics();

        let router_solver = ChiralRouterSolver::new(self.router_params.clone());
        let router_m = router_solver.evaluate_metrics();

        let rep_solver = EntanglementRepeaterSolver::new(self.repeater_params.clone());
        let rep_m = rep_solver.evaluate_metrics();

        ChiralTransducerRepeaterAuditReport {
            bidirectional_transduction_efficiency: trans_m.bidirectional_efficiency_percent >= 15.0,
            transduction_bandwidth: trans_m.transduction_bandwidth_mhz >= 2.0,
            added_thermal_noise: trans_m.added_noise_quanta <= 0.10,
            optomechanical_coupling_rate: self.transducer_params.optomech_coupling_g0_khz >= 750.0,
            chiral_router_forward_loss: router_m.forward_insertion_loss_db <= 0.40,
            chiral_router_isolation: router_m.backward_isolation_db >= 38.0,
            port_return_loss: router_m.port_return_loss_db >= 22.0,
            swapped_state_fidelity: rep_m.swapped_state_fidelity_percent >= 92.0,
            swapped_state_concurrence: rep_m.swapped_concurrence >= 0.88,
            quantum_repeater_rate_gain: rep_m.repeater_rate_gain >= 2.0,
        }
    }
}
