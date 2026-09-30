#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian chiral topological skyrmion-lattice quantum neural processors and
//! synaptic braiding synthesizers.

/// Physical parameter configuration for quantum acoustic non-Abelian chiral topological
/// skyrmion-lattice quantum neural processors and synaptic braiding synthesizers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyrmionNeuralProcessorParams {
    /// Synaptic weight coupling energy in meV (clamp 1.0 to 35.0, default 16.5).
    pub synaptic_weight_coupling_mev: f64,
    /// Topological superconducting pairing energy gap in meV (clamp 2.0 to 45.0, default 22.0).
    pub topological_gap_mev: f64,
    /// Acoustic activation frequency in GHz (clamp 1.0 to 12.0, default 5.7).
    pub acoustic_activation_frequency_ghz: f64,
    /// Synaptic braiding speed of chiral skyrmion modes in m/s (clamp 200.0 to 3000.0, default 1400.0).
    pub synaptic_braiding_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave programming power in micro-watts (clamp 0.5 to 30.0, default 5.8).
    pub microwave_programming_power_uw: f64,
    /// Skyrmion lattice pitch in nanometers (clamp 30.0 to 250.0, default 85.0).
    pub skyrmion_lattice_pitch_nm: f64,
    /// Synaptic array dimension (effective crossbar dimension, clamp 4.0 to 64.0, default 16.0).
    pub synaptic_array_dimension: f64,
}

impl Default for SkyrmionNeuralProcessorParams {
    fn default() -> Self {
        Self {
            synaptic_weight_coupling_mev: 16.5,
            topological_gap_mev: 22.0,
            acoustic_activation_frequency_ghz: 5.7,
            synaptic_braiding_speed_m_per_s: 1400.0,
            cryogenic_temperature_mk: 10.0,
            microwave_programming_power_uw: 5.8,
            skyrmion_lattice_pitch_nm: 85.0,
            synaptic_array_dimension: 16.0,
        }
    }
}

impl SkyrmionNeuralProcessorParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        synaptic_weight_coupling_mev: f64,
        topological_gap_mev: f64,
        acoustic_activation_frequency_ghz: f64,
        synaptic_braiding_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_programming_power_uw: f64,
        skyrmion_lattice_pitch_nm: f64,
        synaptic_array_dimension: f64,
    ) -> Self {
        Self {
            synaptic_weight_coupling_mev: synaptic_weight_coupling_mev.clamp(1.0, 35.0),
            topological_gap_mev: topological_gap_mev.clamp(2.0, 45.0),
            acoustic_activation_frequency_ghz: acoustic_activation_frequency_ghz.clamp(1.0, 12.0),
            synaptic_braiding_speed_m_per_s: synaptic_braiding_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_programming_power_uw: microwave_programming_power_uw.clamp(0.5, 30.0),
            skyrmion_lattice_pitch_nm: skyrmion_lattice_pitch_nm.clamp(30.0, 250.0),
            synaptic_array_dimension: synaptic_array_dimension.clamp(4.0, 64.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian chiral topological
/// skyrmion-lattice quantum neural processors and synaptic braiding synthesizers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyrmionNeuralProcessorMetrics {
    /// Neuromorphic inference fidelity (target >= 0.9980).
    pub neuromorphic_inference_fidelity: f64,
    /// Synaptic state retention fraction (target >= 0.9970).
    pub synaptic_state_retention_fraction: f64,
    /// Topological protection energy gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-synapse crosstalk acoustic isolation in decibels (target >= 54.0 dB).
    pub inter_synapse_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
