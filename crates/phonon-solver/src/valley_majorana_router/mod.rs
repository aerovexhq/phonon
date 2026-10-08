#![deny(unsafe_code)]

//! Quantum Metamaterial Valley-Locked Majorana Zero Mode Acoustic Interconnect & Chiral Majorana Transmon Router Module (Phase 453).
//!
//! Master orchestrator integrating:
//! 1. 2D honeycomb acoustic metamaterial with broken spatial inversion symmetry, opening valley bandgaps (Delta >= 18.0 MHz)
//!    and supporting valley-Hall topological edge states with Delta C_V = 2.
//! 2. Backscattering immunity around sharp 60-degree and 120-degree bends (T_bend / T_straight >= 94.0%) with sub-2.0 cell decay depth.
//! 3. Coherent Majorana-transmon quantum acoustic electromechanical coupling (g >= 25.0 MHz, C >= 150.0, chi_MZM >= 3.5 MHz).
//! 4. 4-port chiral beam splitter routing with high backward isolation (ISO >= 35.0 dB) and wavepacket transfer fidelity >= 99.0%.
//! 5. Fault-tolerant cryogenic quantum interconnect bus operating at 20 mK with ultra-low thermal noise (n_th <= 0.05 quanta)
//!    and suppressed quasiparticle poisoning (Gamma_qp <= 25.0 Hz).

pub mod chiral_beam_splitter;
pub mod fault_tolerant_interconnect;
pub mod majorana_transmon_coupling;
pub mod valley_majorana_lattice;

pub use chiral_beam_splitter::{
    BeamSplitterSMatrixPoint, ChiralBeamSplitterMetrics, ChiralBeamSplitterParams,
    ChiralBeamSplitterSolver,
};
pub use fault_tolerant_interconnect::{
    FaultTolerantInterconnectSolver, InterconnectBusMetrics, InterconnectBusParams,
    InterconnectThermalPoint,
};
pub use majorana_transmon_coupling::{
    MajoranaTransmonCouplingSolver, MajoranaTransmonMetrics, MajoranaTransmonParams,
    TransmonRabiSpectrumPoint,
};
pub use valley_majorana_lattice::{
    ValleyMajoranaDispersionPoint, ValleyMajoranaEdgeSpatialPoint,
    ValleyMajoranaLatticeMetrics, ValleyMajoranaLatticeParams,
    ValleyMajoranaLatticeSolver,
};

/// 10-point rigorous physics audit report for the Valley-Locked Majorana Router.
#[derive(Debug, Clone)]
pub struct ValleyMajoranaAuditReport {
    /// 1. Valley topological bandgap Delta_valley >= 18.0 MHz.
    pub valley_bandgap_pass: bool,
    /// 2. Difference in valley Chern number across domain wall |Delta C_V| == 2.
    pub valley_chern_pass: bool,
    /// 3. Edge mode spatial confinement depth xi <= 2.0 unit cells.
    pub edge_decay_depth_pass: bool,
    /// 4. Sharp bend transmission ratio T_bend / T_straight >= 94.0%.
    pub bend_transmission_pass: bool,
    /// 5. Coherent transmon-Majorana electromechanical coupling g >= 25.0 MHz.
    pub transmon_coupling_pass: bool,
    /// 6. Strong coupling cooperativity C >= 150.0.
    pub cooperativity_pass: bool,
    /// 7. Parity-dependent dispersive shift chi_MZM >= 3.5 MHz.
    pub dispersive_shift_pass: bool,
    /// 8. 4-port chiral backward isolation ISO >= 35.0 dB.
    pub chiral_isolation_pass: bool,
    /// 9. Flying Majorana wavepacket routing fidelity F >= 99.0%.
    pub transfer_fidelity_pass: bool,
    /// 10. Added thermal noise occupancy at 20 mK n_thermal <= 0.05 quanta.
    pub thermal_noise_pass: bool,
}

impl ValleyMajoranaAuditReport {
    /// Returns true if all 10 physics audit criteria evaluated to PASS.
    pub fn all_passed(&self) -> bool {
        self.valley_bandgap_pass
            && self.valley_chern_pass
            && self.edge_decay_depth_pass
            && self.bend_transmission_pass
            && self.transmon_coupling_pass
            && self.cooperativity_pass
            && self.dispersive_shift_pass
            && self.chiral_isolation_pass
            && self.transfer_fidelity_pass
            && self.thermal_noise_pass
    }

    /// Returns the audit score as (passed_count, total_count).
    pub fn score(&self) -> (usize, usize) {
        let items = [
            self.valley_bandgap_pass,
            self.valley_chern_pass,
            self.edge_decay_depth_pass,
            self.bend_transmission_pass,
            self.transmon_coupling_pass,
            self.cooperativity_pass,
            self.dispersive_shift_pass,
            self.chiral_isolation_pass,
            self.transfer_fidelity_pass,
            self.thermal_noise_pass,
        ];
        let passed = items.iter().filter(|&&p| p).count();
        (passed, items.len())
    }

    /// Formats a human-readable text summary of the audit checklist.
    pub fn summary(&self) -> String {
        let (passed, total) = self.score();
        format!(
            "Valley-Locked Majorana Router Audit: {}/{} PASS\n\
             1. Valley Topological Bandgap (Delta >= 18.0 MHz): {}\n\
             2. Valley Chern Index Difference (|Delta C_V| == 2): {}\n\
             3. Edge Modal Localization Depth (xi <= 2.0 cells): {}\n\
             4. Sharp Bend Transmission (T_bend >= 94.0%): {}\n\
             5. Transmon-Majorana Coupling (g >= 25.0 MHz): {}\n\
             6. Strong Coupling Cooperativity (C >= 150.0): {}\n\
             7. Dispersive Parity Shift (chi_MZM >= 3.5 MHz): {}\n\
             8. 4-Port Chiral Backward Isolation (ISO >= 35.0 dB): {}\n\
             9. Flying Wavepacket Transfer Fidelity (F >= 99.0%): {}\n\
             10. Cryogenic Added Noise at 20 mK (n_th <= 0.05 quanta): {}",
            passed,
            total,
            if self.valley_bandgap_pass { "PASS" } else { "FAIL" },
            if self.valley_chern_pass { "PASS" } else { "FAIL" },
            if self.edge_decay_depth_pass { "PASS" } else { "FAIL" },
            if self.bend_transmission_pass { "PASS" } else { "FAIL" },
            if self.transmon_coupling_pass { "PASS" } else { "FAIL" },
            if self.cooperativity_pass { "PASS" } else { "FAIL" },
            if self.dispersive_shift_pass { "PASS" } else { "FAIL" },
            if self.chiral_isolation_pass { "PASS" } else { "FAIL" },
            if self.transfer_fidelity_pass { "PASS" } else { "FAIL" },
            if self.thermal_noise_pass { "PASS" } else { "FAIL" }
        )
    }
}

/// Master processor orchestrating the Quantum Metamaterial Valley-Locked Majorana Router.
#[derive(Debug, Clone)]
pub struct ValleyMajoranaRouterProcessor {
    pub lattice_params: ValleyMajoranaLatticeParams,
    pub transmon_params: MajoranaTransmonParams,
    pub splitter_params: ChiralBeamSplitterParams,
    pub interconnect_params: InterconnectBusParams,
}

impl Default for ValleyMajoranaRouterProcessor {
    fn default() -> Self {
        Self {
            lattice_params: ValleyMajoranaLatticeParams::default(),
            transmon_params: MajoranaTransmonParams::default(),
            splitter_params: ChiralBeamSplitterParams::default(),
            interconnect_params: InterconnectBusParams::default(),
        }
    }
}

impl ValleyMajoranaRouterProcessor {
    /// Constructs a new master processor.
    pub fn new(
        lattice_params: ValleyMajoranaLatticeParams,
        transmon_params: MajoranaTransmonParams,
        splitter_params: ChiralBeamSplitterParams,
        interconnect_params: InterconnectBusParams,
    ) -> Self {
        Self {
            lattice_params,
            transmon_params,
            splitter_params,
            interconnect_params,
        }
    }

    /// Evaluates the complete 10-point physics audit.
    pub fn evaluate_audit(&self) -> ValleyMajoranaAuditReport {
        let lattice_solver = ValleyMajoranaLatticeSolver::new(self.lattice_params.clone());
        let transmon_solver = MajoranaTransmonCouplingSolver::new(self.transmon_params.clone());
        let splitter_solver = ChiralBeamSplitterSolver::new(self.splitter_params.clone());
        let interconnect_solver = FaultTolerantInterconnectSolver::new(self.interconnect_params.clone());

        let lat_m = lattice_solver.evaluate_metrics();
        let tr_m = transmon_solver.evaluate_metrics();
        let sp_m = splitter_solver.evaluate_metrics();
        let bus_m = interconnect_solver.evaluate_metrics();

        ValleyMajoranaAuditReport {
            valley_bandgap_pass: lat_m.valley_bandgap_mhz >= 18.0,
            valley_chern_pass: lat_m.delta_valley_chern_number.abs() == 2,
            edge_decay_depth_pass: lat_m.edge_mode_decay_depth_cells <= 2.0,
            bend_transmission_pass: lat_m.bend_transmission_ratio >= 0.940,
            transmon_coupling_pass: tr_m.coupling_rate_g_mhz >= 25.0,
            cooperativity_pass: tr_m.cooperativity >= 150.0,
            dispersive_shift_pass: tr_m.dispersive_shift_chi_mhz >= 3.5,
            chiral_isolation_pass: sp_m.backward_isolation_db >= 35.0,
            transfer_fidelity_pass: sp_m.transfer_fidelity_pct >= 99.0,
            thermal_noise_pass: bus_m.thermal_noise_occupancy <= 0.05,
        }
    }
}
