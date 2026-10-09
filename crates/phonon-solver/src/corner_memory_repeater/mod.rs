#![deny(unsafe_code)]

pub mod corner_state_memory;
pub mod synthetic_gauge_transduction;
pub mod cv_quantum_repeater;

pub use corner_state_memory::{
    HigherOrderCornerMemoryMetrics, HigherOrderCornerMemoryParams, HigherOrderCornerMemorySolver,
    HigherOrderCornerSpatialPoint,
};
pub use synthetic_gauge_transduction::{
    SyntheticGaugeTransductionParams, SyntheticGaugeTransductionSolver,
    TransductionBusMetrics, TransductionSpectrumPoint,
};
pub use cv_quantum_repeater::{
    CvQuantumRepeaterParams, CvQuantumRepeaterSolver, CvRepeaterMetrics,
    RepeaterNodePoint, RepeaterSqueezingProfilePoint,
};

/// 10-point comprehensive physics audit report for corner memory and quantum repeater.
#[derive(Debug, Clone, PartialEq)]
pub struct CornerMemoryRepeaterAuditReport {
    /// 1. Quantized quadrupole bulk moment q_xy = 0.5 and bulk gap >= 15.0 MHz.
    pub quadrupole_bulk_moment_pass: bool,
    /// 2. Corner acoustic state spatial energy confinement eta_corner >= 90.0%.
    pub corner_state_confinement_pass: bool,
    /// 3. Corner cavity storage lifetime tau_store >= 2.0 ms and Q >= 150,000.
    pub corner_storage_lifetime_pass: bool,
    /// 4. Dilution refrigerator thermal phonon population n_th <= 1.0e-4 at 15 mK.
    pub thermal_occupancy_pass: bool,
    /// 5. Synthetic gauge chiral reverse isolation ISO >= 40.0 dB.
    pub chiral_isolation_pass: bool,
    /// 6. Inter-corner state transduction insertion loss IL <= 0.35 dB.
    pub transduction_loss_pass: bool,
    /// 7. Coherent state transfer fidelity F_transfer >= 99.5%.
    pub state_transfer_fidelity_pass: bool,
    /// 8. Quadrature squeezing depth >= 6.5 dB below SQL.
    pub quadrature_squeezing_pass: bool,
    /// 9. Continuous-variable entanglement swapping fidelity F_swap >= 99.0%.
    pub entanglement_swapping_pass: bool,
    /// 10. Duan-Simon EPR inseparability nullifier Delta_EPR <= 0.40 < 1.0.
    pub duan_simon_nullifier_pass: bool,
    /// Total passed audit criteria (out of 10).
    pub passed_count: usize,
    /// Total evaluated audit criteria (10).
    pub total_count: usize,
}

impl CornerMemoryRepeaterAuditReport {
    /// Check if all 10 physics audit criteria passed.
    pub fn is_all_pass(&self) -> bool {
        self.passed_count == self.total_count && self.total_count == 10
    }
}

/// Unified coordinator for SOTI corner acoustic memory registers, chiral transduction bus, and CV repeaters.
#[derive(Debug, Clone)]
pub struct CornerMemoryRepeaterProcessor {
    /// SOTI corner acoustic quantum memory solver.
    pub corner_memory: HigherOrderCornerMemorySolver,
    /// Synthetic gauge field chiral inter-cavity transduction solver.
    pub transduction_bus: SyntheticGaugeTransductionSolver,
    /// Continuous-variable quantum repeater super-array solver.
    pub cv_repeater: CvQuantumRepeaterSolver,
}

impl Default for CornerMemoryRepeaterProcessor {
    fn default() -> Self {
        Self::new(
            HigherOrderCornerMemoryParams::default(),
            SyntheticGaugeTransductionParams::default(),
            CvQuantumRepeaterParams::default(),
        )
    }
}

impl CornerMemoryRepeaterProcessor {
    /// Create a new processor with the specified subsystem parameters.
    pub fn new(
        memory_params: HigherOrderCornerMemoryParams,
        transduction_params: SyntheticGaugeTransductionParams,
        repeater_params: CvQuantumRepeaterParams,
    ) -> Self {
        Self {
            corner_memory: HigherOrderCornerMemorySolver::new(memory_params),
            transduction_bus: SyntheticGaugeTransductionSolver::new(transduction_params),
            cv_repeater: CvQuantumRepeaterSolver::new(repeater_params),
        }
    }

    /// Perform a rigorous 10-point physics audit.
    pub fn audit(&self) -> CornerMemoryRepeaterAuditReport {
        let m_metrics = self.corner_memory.compute_metrics();
        let t_metrics = self.transduction_bus.compute_metrics();
        let r_metrics = self.cv_repeater.compute_metrics();

        // 1. Quantized quadrupole bulk moment and gap
        let quadrupole_bulk_moment_pass = (m_metrics.quadrupole_moment_qxy - 0.5).abs() < 1e-4
            && m_metrics.bulk_bandgap_mhz >= 15.0;

        // 2. Corner state spatial energy confinement
        let corner_state_confinement_pass = m_metrics.corner_confinement_ratio >= 0.90;

        // 3. Corner cavity storage lifetime
        let corner_storage_lifetime_pass = m_metrics.storage_lifetime_ms >= 2.0
            && self.corner_memory.params().quality_factor >= 150_000.0;

        // 4. Dilution refrigerator thermal phonon population
        let thermal_occupancy_pass = m_metrics.thermal_phonon_occupancy <= 1.0e-4;

        // 5. Synthetic gauge chiral reverse isolation
        let chiral_isolation_pass = t_metrics.reverse_chiral_isolation_db >= 40.0;

        // 6. Inter-corner state transduction insertion loss
        let transduction_loss_pass = t_metrics.forward_insertion_loss_db <= 0.35;

        // 7. Coherent state transfer fidelity
        let state_transfer_fidelity_pass = t_metrics.state_transfer_fidelity >= 0.995;

        // 8. Quadrature squeezing depth
        let quadrature_squeezing_pass = r_metrics.squeezing_depth_db >= 6.5;

        // 9. Continuous-variable entanglement swapping fidelity
        let entanglement_swapping_pass = r_metrics.entanglement_swapping_fidelity >= 0.990;

        // 10. Duan-Simon EPR inseparability nullifier
        let duan_simon_nullifier_pass = r_metrics.duan_simon_nullifier <= 0.40;

        let checks = [
            quadrupole_bulk_moment_pass,
            corner_state_confinement_pass,
            corner_storage_lifetime_pass,
            thermal_occupancy_pass,
            chiral_isolation_pass,
            transduction_loss_pass,
            state_transfer_fidelity_pass,
            quadrature_squeezing_pass,
            entanglement_swapping_pass,
            duan_simon_nullifier_pass,
        ];

        let passed_count = checks.iter().filter(|&&c| c).count();

        CornerMemoryRepeaterAuditReport {
            quadrupole_bulk_moment_pass,
            corner_state_confinement_pass,
            corner_storage_lifetime_pass,
            thermal_occupancy_pass,
            chiral_isolation_pass,
            transduction_loss_pass,
            state_transfer_fidelity_pass,
            quadrature_squeezing_pass,
            entanglement_swapping_pass,
            duan_simon_nullifier_pass,
            passed_count,
            total_count: 10,
        }
    }
}
