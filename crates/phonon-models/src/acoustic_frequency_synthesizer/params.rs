#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Topological Phononic Acoustic
//! Frequency Synthesizer & Ultra-Low Phase Noise Local Oscillator Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous topological phononic acoustic frequency synthesizer and ultra-low phase noise local oscillator engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcousticFrequencySynthesizerParams {
    /// Acoustic-piezoelectric synthesizer coupling energy in meV (clamp 1.0 to 35.0, default 32.5).
    pub synthesizer_coupling_mev: f64,
    /// Topological frequency comb protection gap energy in meV (clamp 2.0 to 45.0, default 38.5).
    pub topological_comb_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 12.0).
    pub acoustic_drive_frequency_ghz: f64,
    /// Synthesizer dispatch speed in m/s (clamp 200.0 to 3000.0, default 2950.0).
    pub synthesizer_dispatch_speed_m_per_s: f64,
    /// Cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave control probe power in microwatts (clamp 0.5 to 30.0, default 13.8).
    pub microwave_probe_power_uw: f64,
    /// Synthetic multi-mode acoustic frequency comb factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_comb_modes_factor: f64,
    /// Inter-element local oscillator cavity pitch in micrometers (clamp 0.5 to 20.0, default 12.8).
    pub oscillator_cavity_pitch_um: f64,
}

impl Default for AcousticFrequencySynthesizerParams {
    fn default() -> Self {
        Self {
            synthesizer_coupling_mev: 32.5,
            topological_comb_gap_mev: 38.5,
            acoustic_drive_frequency_ghz: 12.0,
            synthesizer_dispatch_speed_m_per_s: 2950.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 13.8,
            synthetic_comb_modes_factor: 4.0,
            oscillator_cavity_pitch_um: 12.8,
        }
    }
}

impl AcousticFrequencySynthesizerParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        synthesizer_coupling_mev: f64,
        topological_comb_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        synthesizer_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_comb_modes_factor: f64,
        oscillator_cavity_pitch_um: f64,
    ) -> Self {
        Self {
            synthesizer_coupling_mev: synthesizer_coupling_mev.clamp(1.0, 35.0),
            topological_comb_gap_mev: topological_comb_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            synthesizer_dispatch_speed_m_per_s: synthesizer_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_comb_modes_factor: synthetic_comb_modes_factor.clamp(1.0, 8.0),
            oscillator_cavity_pitch_um: oscillator_cavity_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Topological Phononic Acoustic Frequency Synthesizer & Ultra-Low Phase Noise Local Oscillator Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcousticFrequencySynthesizerMetrics {
    /// Frequency synthesis fidelity (target >= 0.9980).
    pub frequency_synthesis_fidelity: f64,
    /// Oscillator state retention fraction across synthetic comb modes (target >= 0.9970).
    pub oscillator_state_retention_fraction: f64,
    /// Topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-mode crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_mode_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
