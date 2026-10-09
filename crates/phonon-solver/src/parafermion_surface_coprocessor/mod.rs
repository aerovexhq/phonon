#![deny(unsafe_code)]

//! Phase 461: Quantum Metamaterial Non-Abelian Parafermion Lattice Co-Processor & Universal Quantum Acoustic Surface Engine.
//!
//! Master coordinator and physics invariant audit engine integrating:
//! 1. Non-Abelian Z_4 (and Z_3) parafermion lattice zero modes with topological protection gap
//!    Delta_para >= 2.0 MHz, exact commutation algebra alpha_j alpha_k = omega alpha_k alpha_j,
//!    Artin non-Abelian braid relations, and adiabatic braid gate fidelity >= 99.5%.
//! 2. Higher-genus topological quantum acoustic surface code with commuting plaquette and star
//!    stabilizers, exponential logical qudit error suppression (P_L <= 1.0e-4 at p_phys <= 1.0%),
//!    and dispersive cavity parity readout (splitting Delta_omega >= 3.0 MHz, SNR >= 16.0 dB).
//! 3. Cryogenic multi-qudit quantum acoustic co-processor operating at 15 mK dilution temperatures
//!    with thermal phonon occupancy n_th <= 1.0e-3 quanta, clock rate >= 500 kHz, universal
//!    entanglement concurrence C >= 0.90, and inter-qudit crosstalk isolation >= 42.0 dB.

pub mod cryogenic_coprocessor;
pub mod parafermion_lattice;
pub mod quantum_acoustic_surface;

pub use cryogenic_coprocessor::{
    CompiledGateReport, CryogenicCoprocessorMetrics, CryogenicCoprocessorParams,
    CryogenicCoprocessorSolver, UniversalQuditGate,
};
pub use parafermion_lattice::{
    SurfaceParafermionBraidPoint, SurfaceParafermionMetrics, SurfaceParafermionModePoint,
    SurfaceParafermionParams, SurfaceParafermionSolver,
};
pub use quantum_acoustic_surface::{
    ParafermionReadoutSpectrumPoint, ParafermionSurfaceCodeMetrics, ParafermionSurfaceCodeParams,
    ParafermionSurfaceCodeSolver, ParafermionThresholdCurvePoint,
};

/// 10-point rigorous physics audit report for Phase 461.
#[derive(Debug, Clone)]
pub struct ParafermionSurfaceAuditReport {
    /// 1. Parafermion fractional exchange commutation phase theta = 2*pi / m (|theta - 2pi/m| <= 1e-4).
    pub parafermion_commutation_algebra: bool,
    /// 2. Topological protection energy gap Delta_para >= 2.0 MHz.
    pub topological_protection_gap: bool,
    /// 3. Non-Abelian Artin braid relations B_1 B_2 B_1 = B_2 B_1 B_2 (residual <= 1e-4).
    pub artin_braid_relations: bool,
    /// 4. Adiabatic braid gate process fidelity F_braid >= 99.5%.
    pub adiabatic_braid_fidelity: bool,
    /// 5. Surface code stabilizer algebra commutation |[A_s, B_p]| <= 1e-6.
    pub stabilizer_algebra_commutation: bool,
    /// 6. Logical qudit error rate suppression P_L <= 1.0e-4 at p_phys <= 1.0%.
    pub logical_qudit_error_suppression: bool,
    /// 7. Dispersive cavity parity readout splitting Delta_omega >= 3.0 MHz and SNR >= 16.0 dB.
    pub dispersive_parity_readout: bool,
    /// 8. Dilution refrigerator thermal phonon occupancy n_th <= 1.0e-3 at 15 mK.
    pub cryogenic_thermal_occupancy: bool,
    /// 9. Universal multi-qudit bipartite entanglement concurrence C >= 0.90.
    pub universal_qudit_concurrence: bool,
    /// 10. Inter-qudit acoustic crosstalk isolation >= 42.0 dB and bus loss IL <= 0.35 dB.
    pub inter_qudit_crosstalk_isolation: bool,
}

impl ParafermionSurfaceAuditReport {
    /// Returns the (passed_count, total_count) score.
    pub fn score(&self) -> (usize, usize) {
        let items = [
            self.parafermion_commutation_algebra,
            self.topological_protection_gap,
            self.artin_braid_relations,
            self.adiabatic_braid_fidelity,
            self.stabilizer_algebra_commutation,
            self.logical_qudit_error_suppression,
            self.dispersive_parity_readout,
            self.cryogenic_thermal_occupancy,
            self.universal_qudit_concurrence,
            self.inter_qudit_crosstalk_isolation,
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

/// Master coordinator processor for Parafermion Lattice Co-Processor & Surface Engine (Phase 461).
#[derive(Debug, Clone)]
pub struct ParafermionSurfaceProcessor {
    pub lattice_params: SurfaceParafermionParams,
    pub surface_params: ParafermionSurfaceCodeParams,
    pub coprocessor_params: CryogenicCoprocessorParams,
}

impl ParafermionSurfaceProcessor {
    /// Creates a new master processor instance.
    pub fn new(
        lattice_params: SurfaceParafermionParams,
        surface_params: ParafermionSurfaceCodeParams,
        coprocessor_params: CryogenicCoprocessorParams,
    ) -> Self {
        Self {
            lattice_params,
            surface_params,
            coprocessor_params,
        }
    }

    /// Runs a comprehensive 10-point physics audit across all subsystems.
    pub fn audit_system(&self) -> ParafermionSurfaceAuditReport {
        let lat_solver = SurfaceParafermionSolver::new(self.lattice_params.clone());
        let lat_m = lat_solver.evaluate_metrics();

        let surf_solver = ParafermionSurfaceCodeSolver::new(self.surface_params.clone());
        let surf_m = surf_solver.evaluate_metrics();

        let coproc_solver = CryogenicCoprocessorSolver::new(self.coprocessor_params.clone());
        let coproc_m = coproc_solver.evaluate_metrics();

        let expected_phase = 2.0 * std::f64::consts::PI / (self.lattice_params.parafermion_order_m.max(2) as f64);

        ParafermionSurfaceAuditReport {
            parafermion_commutation_algebra: (lat_m.commutation_phase_rad - expected_phase).abs() <= 1e-4,
            topological_protection_gap: lat_m.topological_gap_mhz >= 2.0,
            artin_braid_relations: lat_m.artin_braid_error <= 1e-4,
            adiabatic_braid_fidelity: lat_m.braid_fidelity_percent >= 99.5,
            stabilizer_algebra_commutation: surf_m.stabilizer_commutator_residual <= 1e-6,
            logical_qudit_error_suppression: surf_m.logical_error_rate <= 1.0e-4,
            dispersive_parity_readout: surf_m.parity_readout_splitting_mhz >= 3.0
                && surf_m.readout_snr_db >= 16.0,
            cryogenic_thermal_occupancy: coproc_m.thermal_phonon_occupancy <= 1.0e-3,
            universal_qudit_concurrence: coproc_m.entanglement_concurrence >= 0.90,
            inter_qudit_crosstalk_isolation: coproc_m.inter_qudit_isolation_db >= 42.0
                && coproc_m.bus_insertion_loss_db <= 0.35,
        }
    }
}
