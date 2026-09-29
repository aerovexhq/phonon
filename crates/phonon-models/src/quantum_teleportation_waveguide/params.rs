//! Physical parameter definitions and evaluation metrics for superconducting
//! optomechanical quantum teleportation across phononic crystal waveguides.

/// Physical parameters for phononic waveguide-mediated quantum teleportation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumTeleportationParams {
    /// Superconducting qubit transition frequency in GHz (clamp 3.0 to 10.0, default 5.2).
    pub qubit_frequency_ghz: f64,
    /// Phononic crystal waveguide link length in cm (clamp 0.1 to 20.0, default 2.5).
    pub waveguide_length_cm: f64,
    /// Acoustic waveguide propagation attenuation loss in dB/cm (clamp 0.001 to 0.10, default 0.025).
    pub waveguide_loss_db_per_cm: f64,
    /// Piezoelectric optomechanical / electro-acoustic cooperativity (clamp 10.0 to 200.0, default 45.0).
    pub piezoelectric_cooperativity: f64,
    /// Two-mode squeezing parameter r for propagating acoustic Bell pair (clamp 0.5 to 3.0, default 1.45).
    pub two_mode_squeezing_param: f64,
    /// Joint Bell-state measurement (BSM) detector efficiency (clamp 0.70 to 0.999, default 0.96).
    pub bell_measurement_efficiency: f64,
    /// Quantum memory coherence dephasing time T2 in milliseconds (clamp 0.1 to 50.0, default 2.2).
    pub quantum_memory_t2_ms: f64,
    /// Cryogenic operating temperature in milliKelvin (clamp 1.0 to 100.0, default 15.0).
    pub operating_temp_m_k: f64,
}

impl Default for QuantumTeleportationParams {
    fn default() -> Self {
        Self {
            qubit_frequency_ghz: 5.2,
            waveguide_length_cm: 2.5,
            waveguide_loss_db_per_cm: 0.025,
            piezoelectric_cooperativity: 45.0,
            two_mode_squeezing_param: 1.45,
            bell_measurement_efficiency: 0.96,
            quantum_memory_t2_ms: 2.2,
            operating_temp_m_k: 15.0,
        }
    }
}

impl QuantumTeleportationParams {
    /// Creates a new parameter configuration with physical clamping bounds.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        qubit_frequency_ghz: f64,
        waveguide_length_cm: f64,
        waveguide_loss_db_per_cm: f64,
        piezoelectric_cooperativity: f64,
        two_mode_squeezing_param: f64,
        bell_measurement_efficiency: f64,
        quantum_memory_t2_ms: f64,
        operating_temp_m_k: f64,
    ) -> Self {
        Self {
            qubit_frequency_ghz: qubit_frequency_ghz.clamp(3.0, 10.0),
            waveguide_length_cm: waveguide_length_cm.clamp(0.1, 20.0),
            waveguide_loss_db_per_cm: waveguide_loss_db_per_cm.clamp(0.001, 0.10),
            piezoelectric_cooperativity: piezoelectric_cooperativity.clamp(10.0, 200.0),
            two_mode_squeezing_param: two_mode_squeezing_param.clamp(0.5, 3.0),
            bell_measurement_efficiency: bell_measurement_efficiency.clamp(0.70, 0.999),
            quantum_memory_t2_ms: quantum_memory_t2_ms.clamp(0.1, 50.0),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 100.0),
        }
    }
}

/// Evaluation metrics for superconducting optomechanical quantum teleportation across phononic waveguides.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumTeleportationMetrics {
    /// Quantum state teleportation fidelity (target >= 0.850 or 85.0%).
    pub teleportation_fidelity: f64,
    /// Distilled entanglement pair purity via purification protocol (target >= 0.920 or 92.0%).
    pub entanglement_distillation_purity: f64,
    /// Phononic crystal waveguide propagation loss in dB/cm (target <= 0.050 dB/cm).
    pub waveguide_propagation_loss_db_per_cm: f64,
    /// Remote quantum memory coherence time T2 in ms (target >= 1.00 ms).
    pub quantum_memory_coherence_time_ms: f64,
    /// Propagating Bell-state concurrence C (target >= 0.800).
    pub bell_state_concurrence: f64,
    /// Physical compliance flag verifying all Phase 116 roadmap benchmarks are met.
    pub is_physically_compliant: bool,
}
