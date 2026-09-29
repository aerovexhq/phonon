#![deny(unsafe_code)]

//! Parameter configurations and multi-physics evaluation metrics for
//! quantum acoustic tensor network simulators and continuous-variable
//! fault-tolerant magic state distillation.

/// Physical parameter configuration for quantum acoustic tensor network distillation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumAcousticTensorDistillationParams {
    /// Number of acoustic resonator modes in tensor network (clamp 4 to 64, default 16).
    pub resonator_modes: usize,
    /// Tensor network virtual bond dimension chi (clamp 8 to 128, default 32).
    pub bond_dimension: usize,
    /// Acoustic center frequency in gigahertz (clamp 1.0 to 12.0, default 5.2 GHz).
    pub acoustic_frequency_ghz: f64,
    /// Acoustic cavity mechanical quality factor Q (clamp 1.0e5 to 1.0e8, default 2.0e7).
    pub cavity_q_factor: f64,
    /// Squeezing parameter r for continuous-variable non-Gaussian preparation (clamp 0.5 to 2.5, default 1.25).
    pub squeezing_param_r: f64,
    /// Non-linear electro-acoustic coupling strength in megahertz (clamp 1.0 to 50.0, default 18.0 MHz).
    pub non_linear_coupling_mhz: f64,
    /// Cryostat dilution refrigerator operating ambient temperature in milli-Kelvin (clamp 1.0 to 50.0, default 15.0 mK).
    pub operating_temp_m_k: f64,
    /// Single-phonon subtraction detection efficiency eta (clamp 0.50 to 0.99, default 0.88).
    pub photon_subtraction_efficiency: f64,
}

impl Default for QuantumAcousticTensorDistillationParams {
    fn default() -> Self {
        Self {
            resonator_modes: 16,
            bond_dimension: 32,
            acoustic_frequency_ghz: 5.2,
            cavity_q_factor: 2.0e7,
            squeezing_param_r: 1.25,
            non_linear_coupling_mhz: 18.0,
            operating_temp_m_k: 15.0,
            photon_subtraction_efficiency: 0.88,
        }
    }
}

impl QuantumAcousticTensorDistillationParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        resonator_modes: usize,
        bond_dimension: usize,
        acoustic_frequency_ghz: f64,
        cavity_q_factor: f64,
        squeezing_param_r: f64,
        non_linear_coupling_mhz: f64,
        operating_temp_m_k: f64,
        photon_subtraction_efficiency: f64,
    ) -> Self {
        Self {
            resonator_modes: resonator_modes.clamp(4, 64),
            bond_dimension: bond_dimension.clamp(8, 128),
            acoustic_frequency_ghz: acoustic_frequency_ghz.clamp(1.0, 12.0),
            cavity_q_factor: cavity_q_factor.clamp(1.0e5, 1.0e8),
            squeezing_param_r: squeezing_param_r.clamp(0.5, 2.5),
            non_linear_coupling_mhz: non_linear_coupling_mhz.clamp(1.0, 50.0),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 50.0),
            photon_subtraction_efficiency: photon_subtraction_efficiency.clamp(0.50, 0.99),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic tensor distillation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumAcousticTensorDistillationMetrics {
    /// Continuous-variable magic state output fidelity (target >= 0.990).
    pub magic_state_fidelity: f64,
    /// Single-phonon subtraction heralding success probability (target >= 0.150).
    pub photon_subtraction_prob: f64,
    /// Distillation cycle latency in microseconds including tensor contractions (target <= 5.0 us).
    pub distillation_cycle_latency_us: f64,
    /// Fault-tolerant non-Gaussian cubic phase gate fidelity (target >= 0.985).
    pub non_gaussian_gate_fidelity: f64,
    /// Continuous-variable quantum acoustic physical error threshold (target >= 0.015 or 1.5%).
    pub acoustic_error_threshold: f64,
    /// Overall physical compliance flag across all tensor distillation metrics.
    pub is_physically_compliant: bool,
}
