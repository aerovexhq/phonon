#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Topological
//! Axion-Polariton Circulator & Quantum Interconnect Hub Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous acoustically driven topological axion-polariton circulator
/// and quantum interconnect hub engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxionPolaritonCirculatorParams {
    /// Circulator coupling energy in meV (clamp 1.0 to 35.0, default 35.0).
    pub circulator_coupling_mev: f64,
    /// Topological axion gap energy in meV (clamp 2.0 to 45.0, default 45.0).
    pub topological_axion_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 12.0).
    pub acoustic_drive_frequency_ghz: f64,
    /// Circulation dispatch speed in m/s (clamp 200.0 to 3000.0, default 3000.0).
    pub circulation_dispatch_speed_m_per_s: f64,
    /// Cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave probe power in microwatts (clamp 0.5 to 30.0, default 23.0).
    pub microwave_probe_power_uw: f64,
    /// Synthetic circulator ports factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_circulator_ports_factor: f64,
    /// Circulator junction pitch in micrometers (clamp 0.5 to 25.0, default 22.0).
    pub circulator_junction_pitch_um: f64,
}

impl Default for AxionPolaritonCirculatorParams {
    fn default() -> Self {
        Self {
            circulator_coupling_mev: 35.0,
            topological_axion_gap_mev: 45.0,
            acoustic_drive_frequency_ghz: 12.0,
            circulation_dispatch_speed_m_per_s: 3000.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 23.0,
            synthetic_circulator_ports_factor: 4.0,
            circulator_junction_pitch_um: 22.0,
        }
    }
}

impl AxionPolaritonCirculatorParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        circulator_coupling_mev: f64,
        topological_axion_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        circulation_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_circulator_ports_factor: f64,
        circulator_junction_pitch_um: f64,
    ) -> Self {
        Self {
            circulator_coupling_mev: circulator_coupling_mev.clamp(1.0, 35.0),
            topological_axion_gap_mev: topological_axion_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            circulation_dispatch_speed_m_per_s: circulation_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_circulator_ports_factor: synthetic_circulator_ports_factor.clamp(1.0, 8.0),
            circulator_junction_pitch_um: circulator_junction_pitch_um.clamp(0.5, 25.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Acoustically Driven Topological Axion-Polariton Circulator & Quantum Interconnect Hub Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxionPolaritonCirculatorMetrics {
    /// Circulator fidelity (target >= 0.9980).
    pub circulator_fidelity: f64,
    /// Polariton state retention fraction (target >= 0.9970).
    pub polariton_state_retention_fraction: f64,
    /// Topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Isolation directivity in decibels (target >= 55.0 dB).
    pub isolation_directivity_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
