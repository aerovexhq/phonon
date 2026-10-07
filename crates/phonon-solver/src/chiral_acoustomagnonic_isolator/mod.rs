#![deny(unsafe_code)]

//! Quantum Metamaterial Chiral Acoustomagnonic Isolator & Cryogenic Microwave Qubit Circulator co-processor.
//!
//! Master orchestrator integrating:
//! 1. Coupled acoustomagnonic polariton dispersion with non-reciprocal synthetic gauge fields.
//! 2. Non-reciprocal surface acoustic wave (SAW) isolator on YIG/LiNbO3.
//! 3. Cryogenic dilution-refrigerator 3-port circulator protecting transmon qubits.
//! 4. Comprehensive 10-point physics audit checklist.

pub mod acoustomagnonic_dispersion;
pub mod cryogenic_qubit_circulator;
pub mod nonreciprocal_saw_isolator;

pub use acoustomagnonic_dispersion::{
    AcoustomagnonicDispersionPoint, AcoustomagnonicDispersionSolver, AcoustomagnonicParams,
};
pub use cryogenic_qubit_circulator::{
    CryogenicCirculatorMetrics, CryogenicCirculatorParams, CryogenicQubitCirculator,
    QubitCirculatorSMatrix,
};
pub use nonreciprocal_saw_isolator::{
    NonReciprocalSawIsolator, SawFrequencyResponsePoint, SawIsolatorMetrics,
};

/// 10-point physics audit checklist result for the acoustomagnonic processor.
#[derive(Debug, Clone)]
pub struct AcoustomagnonicAuditReport {
    /// 1. Avoided crossing polariton gap >= 40.0 MHz.
    pub pass_polariton_gap: bool,
    /// 2. Non-reciprocal wavevector splitting Delta k > 0.
    pub pass_wavevector_splitting: bool,
    /// 3. Low forward insertion loss IL <= 0.6 dB.
    pub pass_insertion_loss: bool,
    /// 4. Deep reverse isolation >= 35.0 dB.
    pub pass_isolation_depth: bool,
    /// 5. Isolation contrast (ISO - IL) >= 34.0 dB.
    pub pass_isolation_contrast: bool,
    /// 6. 3-dB operational bandwidth >= 30.0 MHz.
    pub pass_bandwidth: bool,
    /// 7. 3-port cyclic circulator unitarity error < 0.05.
    pub pass_circulator_unitarity: bool,
    /// 8. Near-quantum-limited added noise n_add <= 0.55 quanta at 20 mK.
    pub pass_quantum_added_noise: bool,
    /// 9. Thermal back-action leakage suppression < 1e-3 photons.
    pub pass_thermal_leakage_suppression: bool,
    /// 10. Qubit dispersive readout SNR >= 18.5 dB and fidelity >= 0.998.
    pub pass_readout_performance: bool,
    /// Overall pass count out of 10.
    pub pass_count: usize,
    /// All 10 passed.
    pub all_passed: bool,
}

/// Unified master orchestrator for the chiral acoustomagnonic processor.
#[derive(Debug, Clone)]
pub struct ChiralAcoustomagnonicProcessor {
    pub dispersion_solver: AcoustomagnonicDispersionSolver,
    pub saw_isolator: NonReciprocalSawIsolator,
    pub qubit_circulator: CryogenicQubitCirculator,
}

impl Default for ChiralAcoustomagnonicProcessor {
    fn default() -> Self {
        let params = AcoustomagnonicParams::default();
        let circ_params = CryogenicCirculatorParams::default();
        Self {
            dispersion_solver: AcoustomagnonicDispersionSolver::new(params.clone()),
            saw_isolator: NonReciprocalSawIsolator::new(params, 2.5),
            qubit_circulator: CryogenicQubitCirculator::new(circ_params),
        }
    }
}

impl ChiralAcoustomagnonicProcessor {
    /// Creates a new processor with custom physical configurations.
    pub fn new(
        ac_params: AcoustomagnonicParams,
        waveguide_length_mm: f64,
        circ_params: CryogenicCirculatorParams,
    ) -> Self {
        Self {
            dispersion_solver: AcoustomagnonicDispersionSolver::new(ac_params.clone()),
            saw_isolator: NonReciprocalSawIsolator::new(ac_params, waveguide_length_mm),
            qubit_circulator: CryogenicQubitCirculator::new(circ_params),
        }
    }

    /// Executes the comprehensive 10-point physics audit checklist.
    pub fn audit_acoustomagnonic_processor(&self) -> AcoustomagnonicAuditReport {
        let gap_mhz = self.dispersion_solver.polariton_gap_mhz();
        let pass_polariton_gap = gap_mhz >= 40.0;

        let delta_k = self.dispersion_solver.nonreciprocal_wavevector_splitting();
        let pass_wavevector_splitting = delta_k >= 0.035;

        let saw_metrics = self.saw_isolator.compute_metrics();
        let pass_insertion_loss = saw_metrics.insertion_loss_db <= 0.60;
        let pass_isolation_depth = saw_metrics.isolation_depth_db >= 35.0;
        let pass_isolation_contrast = saw_metrics.isolation_contrast_db >= 34.0;
        let pass_bandwidth = saw_metrics.bandwidth_3db_mhz >= 30.0;

        let s_mat = self.qubit_circulator.evaluate_s_matrix();
        let pass_circulator_unitarity = s_mat.unitarity_deficit < 0.05;

        let circ_metrics = self.qubit_circulator.compute_metrics();
        let pass_quantum_added_noise = circ_metrics.added_noise_quanta <= 0.55;
        let pass_thermal_leakage_suppression = circ_metrics.thermal_leakage_photons < 1.0e-3;
        let pass_readout_performance =
            circ_metrics.readout_snr_db >= 18.5 && circ_metrics.qnd_readout_fidelity >= 0.998;

        let checks = [
            pass_polariton_gap,
            pass_wavevector_splitting,
            pass_insertion_loss,
            pass_isolation_depth,
            pass_isolation_contrast,
            pass_bandwidth,
            pass_circulator_unitarity,
            pass_quantum_added_noise,
            pass_thermal_leakage_suppression,
            pass_readout_performance,
        ];

        let pass_count = checks.iter().filter(|&&c| c).count();
        let all_passed = pass_count == 10;

        AcoustomagnonicAuditReport {
            pass_polariton_gap,
            pass_wavevector_splitting,
            pass_insertion_loss,
            pass_isolation_depth,
            pass_isolation_contrast,
            pass_bandwidth,
            pass_circulator_unitarity,
            pass_quantum_added_noise,
            pass_thermal_leakage_suppression,
            pass_readout_performance,
            pass_count,
            all_passed,
        }
    }
}
