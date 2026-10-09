#![deny(unsafe_code)]

//! Topological Acoustic Floquet Chiral Magnon-Phonon Crossbar Transceiver
//! & Entanglement Router Super-Array (Phase 464).
//!
//! Provides Floquet-engineered chiral acoustic-magnonic hybrid transceivers, synthetic
//! gauge field directional circulators, distributed continuous-variable cluster state
//! entanglement routing, and cryo-CMOS microwave-to-phonon interfaces.

pub mod cluster_entanglement_router;
pub mod floquet_chiral_transceiver;
pub mod synthetic_circulator_array;

pub use cluster_entanglement_router::{
    ClusterEntanglementRouterMetrics, ClusterEntanglementRouterParams,
    ClusterEntanglementRouterSolver, ClusterNodePoint, QuadratureVariancePoint,
};
pub use floquet_chiral_transceiver::{
    FloquetChiralTransceiverMetrics, FloquetChiralTransceiverParams,
    FloquetChiralTransceiverSolver, TransceiverDispersionPoint,
};
pub use synthetic_circulator_array::{
    CirculatorSMatrixElement, SyntheticCirculatorArrayMetrics, SyntheticCirculatorArrayParams,
    SyntheticCirculatorArraySolver,
};

/// 10-point rigorous physics audit report for Phase 464.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FloquetMagnonCrossbarAuditReport {
    /// 1. Floquet dynamic time-reversal symmetry breaking (|k_+ - (-k_-)| >= 0.05 rad/um).
    pub floquet_time_reversal_symmetry_breaking: bool,
    /// 2. High-efficiency magnetoelastic transduction coupling (G_me >= 35.0 MHz).
    pub magnetoelastic_transduction_coupling: bool,
    /// 3. Non-reciprocal chiral isolation across transceiver (ISO >= 40.0 dB).
    pub non_reciprocal_chiral_isolation: bool,
    /// 4. Forward acoustic-magnonic polariton insertion loss (IL <= 0.30 dB).
    pub forward_insertion_loss: bool,
    /// 5. Transceiver operational non-reciprocal bandwidth (Delta_f >= 120.0 MHz).
    pub transceiver_operational_bandwidth: bool,
    /// 6. Multi-terminal synthetic circulator directivity (D >= 38.0 dB).
    pub multi_terminal_circulator_directivity: bool,
    /// 7. Circulator port impedance return loss matching (RL >= 22.0 dB).
    pub port_return_loss_matching: bool,
    /// 8. Continuous-variable quadrature squeezing depth below shot noise (S_sqz >= 6.0 dB).
    pub cv_squeezing_below_shot_noise: bool,
    /// 9. Duan-Simon EPR inseparability nullifier (< 0.50 certifying entanglement).
    pub duan_simon_epr_inseparability: bool,
    /// 10. Cryo-CMOS quantum-limited added noise (n_add <= 0.08 quanta and n_th <= 1.0e-3 at 15 mK).
    pub cryo_cmos_quantum_limited_noise: bool,
}

impl FloquetMagnonCrossbarAuditReport {
    /// Evaluates the 10-point physics audit score (passing_count, total_count).
    pub fn score(&self) -> (usize, usize) {
        let tests = [
            self.floquet_time_reversal_symmetry_breaking,
            self.magnetoelastic_transduction_coupling,
            self.non_reciprocal_chiral_isolation,
            self.forward_insertion_loss,
            self.transceiver_operational_bandwidth,
            self.multi_terminal_circulator_directivity,
            self.port_return_loss_matching,
            self.cv_squeezing_below_shot_noise,
            self.duan_simon_epr_inseparability,
            self.cryo_cmos_quantum_limited_noise,
        ];
        let pass_count = tests.iter().filter(|&&b| b).count();
        (pass_count, tests.len())
    }

    /// Verifies whether all 10 physics audit criteria are satisfied (10/10 PASS).
    pub fn is_pass(&self) -> bool {
        let (passed, total) = self.score();
        passed == total
    }
}

/// Master coordinator for Floquet chiral magnon-phonon crossbar and entanglement router.
#[derive(Debug, Clone)]
pub struct FloquetMagnonCrossbarProcessor {
    pub transceiver_params: FloquetChiralTransceiverParams,
    pub circulator_params: SyntheticCirculatorArrayParams,
    pub router_params: ClusterEntanglementRouterParams,
}

impl Default for FloquetMagnonCrossbarProcessor {
    fn default() -> Self {
        Self {
            transceiver_params: FloquetChiralTransceiverParams::default(),
            circulator_params: SyntheticCirculatorArrayParams::default(),
            router_params: ClusterEntanglementRouterParams::default(),
        }
    }
}

impl FloquetMagnonCrossbarProcessor {
    /// Creates a new processor coordinator with custom parameters.
    pub fn new(
        transceiver_params: FloquetChiralTransceiverParams,
        circulator_params: SyntheticCirculatorArrayParams,
        router_params: ClusterEntanglementRouterParams,
    ) -> Self {
        Self {
            transceiver_params,
            circulator_params,
            router_params,
        }
    }

    /// Solves all three physical sub-engines simultaneously.
    pub fn solve_all(
        &self,
    ) -> (
        FloquetChiralTransceiverMetrics,
        SyntheticCirculatorArrayMetrics,
        ClusterEntanglementRouterMetrics,
    ) {
        let transceiver_solver = FloquetChiralTransceiverSolver::new(self.transceiver_params.clone());
        let transceiver_metrics = transceiver_solver.solve();

        let circulator_solver = SyntheticCirculatorArraySolver::new(self.circulator_params.clone());
        let circulator_metrics = circulator_solver.solve();

        let router_solver = ClusterEntanglementRouterSolver::new(self.router_params.clone());
        let router_metrics = router_solver.solve();

        (transceiver_metrics, circulator_metrics, router_metrics)
    }

    /// Executes the full 10-point physics audit checklist.
    pub fn audit(&self) -> FloquetMagnonCrossbarAuditReport {
        let (trx, circ, rtr) = self.solve_all();

        let floquet_time_reversal_symmetry_breaking = trx.wavevector_asymmetry_rad_per_um >= 0.05;
        let magnetoelastic_transduction_coupling =
            self.transceiver_params.magnetoelastic_coupling_mhz >= 35.0;
        let non_reciprocal_chiral_isolation = trx.reverse_isolation_db >= 40.0;
        let forward_insertion_loss = trx.insertion_loss_db <= 0.30;
        let transceiver_operational_bandwidth = trx.transduction_bandwidth_mhz >= 120.0;
        let multi_terminal_circulator_directivity = circ.directivity_db >= 38.0;
        let port_return_loss_matching = circ.return_loss_db >= 22.0;
        let cv_squeezing_below_shot_noise = rtr.squeezing_depth_db >= 6.0;
        let duan_simon_epr_inseparability = rtr.duan_simon_nullifier <= 0.50;
        let cryo_cmos_quantum_limited_noise =
            rtr.added_noise_quanta <= 0.08 && rtr.thermal_phonon_occupancy <= 1.0e-3;

        FloquetMagnonCrossbarAuditReport {
            floquet_time_reversal_symmetry_breaking,
            magnetoelastic_transduction_coupling,
            non_reciprocal_chiral_isolation,
            forward_insertion_loss,
            transceiver_operational_bandwidth,
            multi_terminal_circulator_directivity,
            port_return_loss_matching,
            cv_squeezing_below_shot_noise,
            duan_simon_epr_inseparability,
            cryo_cmos_quantum_limited_noise,
        }
    }
}
