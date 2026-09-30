#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Superconducting Quatrit
//! State Synthesizer & Multi-Valued Quantum Logic Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous acoustically driven superconducting quatrit state synthesizer
/// and multi-valued quantum logic engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SuperconductingQuatritParams {
    /// Acoustically driven quatrit coupling energy in meV (clamp 1.0 to 35.0, default 35.0).
    pub quatrit_coupling_mev: f64,
    /// Topological quatrit gap energy in meV (clamp 2.0 to 45.0, default 41.5).
    pub topological_quatrit_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 12.0).
    pub acoustic_drive_frequency_ghz: f64,
    /// Quatrit dispatch speed in m/s (clamp 200.0 to 3000.0, default 3000.0).
    pub quatrit_dispatch_speed_m_per_s: f64,
    /// Cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave control probe power in microwatts (clamp 0.5 to 30.0, default 15.2).
    pub microwave_probe_power_uw: f64,
    /// Synthetic quatrit levels factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_quatrit_levels_factor: f64,
    /// Quatrit cell pitch in micrometers (clamp 0.5 to 20.0, default 14.2).
    pub quatrit_cell_pitch_um: f64,
}

impl Default for SuperconductingQuatritParams {
    fn default() -> Self {
        Self {
            quatrit_coupling_mev: 35.0,
            topological_quatrit_gap_mev: 41.5,
            acoustic_drive_frequency_ghz: 12.0,
            quatrit_dispatch_speed_m_per_s: 3000.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 15.2,
            synthetic_quatrit_levels_factor: 4.0,
            quatrit_cell_pitch_um: 14.2,
        }
    }
}

impl SuperconductingQuatritParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        quatrit_coupling_mev: f64,
        topological_quatrit_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        quatrit_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_quatrit_levels_factor: f64,
        quatrit_cell_pitch_um: f64,
    ) -> Self {
        Self {
            quatrit_coupling_mev: quatrit_coupling_mev.clamp(1.0, 35.0),
            topological_quatrit_gap_mev: topological_quatrit_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            quatrit_dispatch_speed_m_per_s: quatrit_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_quatrit_levels_factor: synthetic_quatrit_levels_factor.clamp(1.0, 8.0),
            quatrit_cell_pitch_um: quatrit_cell_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Acoustically Driven Superconducting Quatrit State Synthesizer
/// & Multi-Valued Quantum Logic Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SuperconductingQuatritMetrics {
    /// Quatrit synthesis fidelity (target >= 0.9980).
    pub quatrit_synthesis_fidelity: f64,
    /// Quatrit state retention fraction (target >= 0.9970).
    pub quatrit_state_retention_fraction: f64,
    /// Topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-level crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_level_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
