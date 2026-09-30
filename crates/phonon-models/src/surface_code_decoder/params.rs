#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian chiral topological surface code anyon decoders and fault-tolerant syndrome
//! processors.

/// Physical parameter configuration for quantum acoustic non-Abelian chiral topological
/// surface code anyon decoders and fault-tolerant syndrome processors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceCodeDecoderParams {
    /// Syndrome extraction coupling energy in meV (clamp 1.0 to 35.0, default 16.5).
    pub syndrome_coupling_energy_mev: f64,
    /// Superconducting energy gap in meV (clamp 2.0 to 45.0, default 22.0).
    pub superconducting_gap_mev: f64,
    /// Acoustic clock frequency in GHz (clamp 1.0 to 12.0, default 5.8).
    pub acoustic_clock_frequency_ghz: f64,
    /// Minimum-weight perfect matching anyon shuttling speed in m/s (clamp 200.0 to 3000.0, default 1400.0).
    pub matching_shuttling_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave syndrome readout power in micro-watts (clamp 0.5 to 30.0, default 5.6).
    pub microwave_readout_power_uw: f64,
    /// Topological surface code distance d (clamp 3.0 to 25.0, default 9.0).
    pub code_distance_d: f64,
    /// Physical qubit pitch separation in micrometers (clamp 0.5 to 15.0, default 4.2).
    pub qubit_pitch_um: f64,
}

impl Default for SurfaceCodeDecoderParams {
    fn default() -> Self {
        Self {
            syndrome_coupling_energy_mev: 16.5,
            superconducting_gap_mev: 22.0,
            acoustic_clock_frequency_ghz: 5.8,
            matching_shuttling_speed_m_per_s: 1400.0,
            cryogenic_temperature_mk: 10.0,
            microwave_readout_power_uw: 5.6,
            code_distance_d: 9.0,
            qubit_pitch_um: 4.2,
        }
    }
}

impl SurfaceCodeDecoderParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        syndrome_coupling_energy_mev: f64,
        superconducting_gap_mev: f64,
        acoustic_clock_frequency_ghz: f64,
        matching_shuttling_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_readout_power_uw: f64,
        code_distance_d: f64,
        qubit_pitch_um: f64,
    ) -> Self {
        Self {
            syndrome_coupling_energy_mev: syndrome_coupling_energy_mev.clamp(1.0, 35.0),
            superconducting_gap_mev: superconducting_gap_mev.clamp(2.0, 45.0),
            acoustic_clock_frequency_ghz: acoustic_clock_frequency_ghz.clamp(1.0, 12.0),
            matching_shuttling_speed_m_per_s: matching_shuttling_speed_m_per_s
                .clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_readout_power_uw: microwave_readout_power_uw.clamp(0.5, 30.0),
            code_distance_d: code_distance_d.clamp(3.0, 25.0),
            qubit_pitch_um: qubit_pitch_um.clamp(0.5, 15.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian chiral topological
/// surface code anyon decoders and fault-tolerant syndrome processors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceCodeDecoderMetrics {
    /// Quantum acoustic surface code decoding fidelity (target >= 0.9980).
    pub decoding_fidelity: f64,
    /// Code space retention fraction under syndrome extraction cycles (target >= 0.9970).
    pub code_space_retention_fraction: f64,
    /// Topological protection energy gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-qubit crosstalk acoustic isolation in decibels (target >= 54.0 dB).
    pub inter_qubit_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
