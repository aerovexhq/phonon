#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Topological
//! Axion-Polariton Waveguide & Quantum Hall Beam Splitter Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous acoustically driven topological axion-polariton waveguide and
/// quantum Hall beam splitter engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxionPolaritonBeamSplitterParams {
    /// Splitter coupling energy in meV (clamp 1.0 to 35.0, default 35.0).
    pub splitter_coupling_mev: f64,
    /// Topological axion gap energy in meV (clamp 2.0 to 45.0, default 45.0).
    pub topological_axion_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 12.0).
    pub acoustic_drive_frequency_ghz: f64,
    /// Beam dispatch speed in m/s (clamp 200.0 to 3000.0, default 3000.0).
    pub beam_dispatch_speed_m_per_s: f64,
    /// Cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave probe power in microwatts (clamp 0.5 to 30.0, default 21.0).
    pub microwave_probe_power_uw: f64,
    /// Synthetic Hall ports factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_hall_ports_factor: f64,
    /// Waveguide pitch in micrometers (clamp 0.5 to 20.0, default 20.0).
    pub waveguide_pitch_um: f64,
}

impl Default for AxionPolaritonBeamSplitterParams {
    fn default() -> Self {
        Self {
            splitter_coupling_mev: 35.0,
            topological_axion_gap_mev: 45.0,
            acoustic_drive_frequency_ghz: 12.0,
            beam_dispatch_speed_m_per_s: 3000.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 21.0,
            synthetic_hall_ports_factor: 4.0,
            waveguide_pitch_um: 20.0,
        }
    }
}

impl AxionPolaritonBeamSplitterParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        splitter_coupling_mev: f64,
        topological_axion_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        beam_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_hall_ports_factor: f64,
        waveguide_pitch_um: f64,
    ) -> Self {
        Self {
            splitter_coupling_mev: splitter_coupling_mev.clamp(1.0, 35.0),
            topological_axion_gap_mev: topological_axion_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            beam_dispatch_speed_m_per_s: beam_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_hall_ports_factor: synthetic_hall_ports_factor.clamp(1.0, 8.0),
            waveguide_pitch_um: waveguide_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Acoustically Driven Topological Axion-Polariton Waveguide & Quantum Hall Beam Splitter Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxionPolaritonBeamSplitterMetrics {
    /// Beam splitter fidelity (target >= 0.9980).
    pub beam_splitter_fidelity: f64,
    /// Quantum Hall state retention fraction (target >= 0.9970).
    pub quantum_hall_state_retention_fraction: f64,
    /// Topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-port crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_port_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
