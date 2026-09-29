#![deny(unsafe_code)]

//! Parameter configurations and multi-physics evaluation metrics for
//! hybrid superconducting opto-acoustic quantum repeaters and entanglement
//! distribution networks.

/// Physical parameter configuration for opto-acoustic quantum repeater nodes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OptoAcousticQuantumRepeaterParams {
    /// Number of intermediate repeater nodes in network link (clamp 2 to 16, default 4).
    pub repeater_nodes_count: usize,
    /// Total end-to-end channel distance in kilometers (clamp 1.0 to 100.0, default 50.0 km).
    pub channel_distance_km: f64,
    /// Electro-optomechanical transducer bidirectional quantum efficiency (clamp 0.50 to 0.99, default 0.85).
    pub transducer_efficiency: f64,
    /// Phononic crystal acoustic memory coherence dephasing time T2 in milliseconds (clamp 1.0 to 50.0, default 15.0 ms).
    pub acoustic_memory_coherence_ms: f64,
    /// Optical fiber propagation attenuation in dB/km (clamp 0.15 to 0.35, default 0.20 dB/km).
    pub optical_fiber_attenuation_db_per_km: f64,
    /// Successive entanglement purification rounds (clamp 1 to 5, default 2).
    pub purification_rounds: usize,
    /// Cryogenic dilution refrigerator operating temperature in milli-Kelvin (clamp 1.0 to 50.0, default 15.0 mK).
    pub operating_temp_m_k: f64,
    /// Write/read laser pump repetition frequency in megahertz (clamp 0.5 to 20.0, default 5.0 MHz).
    pub pump_repetition_freq_mhz: f64,
}

impl Default for OptoAcousticQuantumRepeaterParams {
    fn default() -> Self {
        Self {
            repeater_nodes_count: 4,
            channel_distance_km: 50.0,
            transducer_efficiency: 0.85,
            acoustic_memory_coherence_ms: 15.0,
            optical_fiber_attenuation_db_per_km: 0.20,
            purification_rounds: 2,
            operating_temp_m_k: 15.0,
            pump_repetition_freq_mhz: 5.0,
        }
    }
}

impl OptoAcousticQuantumRepeaterParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        repeater_nodes_count: usize,
        channel_distance_km: f64,
        transducer_efficiency: f64,
        acoustic_memory_coherence_ms: f64,
        optical_fiber_attenuation_db_per_km: f64,
        purification_rounds: usize,
        operating_temp_m_k: f64,
        pump_repetition_freq_mhz: f64,
    ) -> Self {
        Self {
            repeater_nodes_count: repeater_nodes_count.clamp(2, 16),
            channel_distance_km: channel_distance_km.clamp(1.0, 100.0),
            transducer_efficiency: transducer_efficiency.clamp(0.50, 0.99),
            acoustic_memory_coherence_ms: acoustic_memory_coherence_ms.clamp(1.0, 50.0),
            optical_fiber_attenuation_db_per_km: optical_fiber_attenuation_db_per_km.clamp(0.15, 0.35),
            purification_rounds: purification_rounds.clamp(1, 5),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 50.0),
            pump_repetition_freq_mhz: pump_repetition_freq_mhz.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for hybrid superconducting opto-acoustic quantum repeaters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OptoAcousticQuantumRepeaterMetrics {
    /// Remote heralded Bell-state generation fidelity (target >= 0.950).
    pub bell_state_fidelity: f64,
    /// Quantum entanglement distribution repetition rate in kilohertz (target >= 100.0 kHz).
    pub repetition_rate_khz: f64,
    /// End-to-end network entanglement distribution latency in microseconds (target <= 10.0 us).
    pub distribution_latency_us: f64,
    /// Quantum memory storage-transduction roundtrip fidelity (target >= 0.980).
    pub memory_transduction_roundtrip_fidelity: f64,
    /// Entanglement purification distillation yield efficiency (target >= 0.850).
    pub purification_efficiency: f64,
    /// Overall physical compliance flag across all quantum repeater roadmap targets.
    pub is_physically_compliant: bool,
}
