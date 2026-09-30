#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Chiral Valley-Phonon Heat Pump
//! and Reversible Nanoscale Cryo-Cooling Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous chiral valley-phonon heat pump and reversible nanoscale cryo-cooling engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValleyHeatPumpParams {
    /// Heat pump coupling energy in meV (clamp 1.0 to 35.0, default 28.5).
    pub heat_pump_coupling_mev: f64,
    /// Topological cooling bandgap energy in meV (clamp 2.0 to 45.0, default 34.5).
    pub topological_cooling_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 11.8).
    pub acoustic_drive_frequency_ghz: f64,
    /// Thermal dispatch speed in m/s (clamp 200.0 to 3000.0, default 2550.0).
    pub thermal_dispatch_speed_m_per_s: f64,
    /// Cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave probe power in microwatts (clamp 0.5 to 30.0, default 11.8).
    pub microwave_probe_power_uw: f64,
    /// Synthetic cooling elements factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_cooling_elements_factor: f64,
    /// Heat pump pitch in micrometers (clamp 0.5 to 20.0, default 10.8).
    pub heat_pump_pitch_um: f64,
}

impl Default for ValleyHeatPumpParams {
    fn default() -> Self {
        Self {
            heat_pump_coupling_mev: 28.5,
            topological_cooling_gap_mev: 34.5,
            acoustic_drive_frequency_ghz: 11.8,
            thermal_dispatch_speed_m_per_s: 2550.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 11.8,
            synthetic_cooling_elements_factor: 4.0,
            heat_pump_pitch_um: 10.8,
        }
    }
}

impl ValleyHeatPumpParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        heat_pump_coupling_mev: f64,
        topological_cooling_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        thermal_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_cooling_elements_factor: f64,
        heat_pump_pitch_um: f64,
    ) -> Self {
        Self {
            heat_pump_coupling_mev: heat_pump_coupling_mev.clamp(1.0, 35.0),
            topological_cooling_gap_mev: topological_cooling_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            thermal_dispatch_speed_m_per_s: thermal_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_cooling_elements_factor: synthetic_cooling_elements_factor.clamp(1.0, 8.0),
            heat_pump_pitch_um: heat_pump_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Chiral Valley-Phonon Heat Pump and Reversible Nanoscale Cryo-Cooling Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValleyHeatPumpMetrics {
    /// Heat pump fidelity (target >= 0.9980).
    pub heat_pump_fidelity: f64,
    /// Refrigeration state retention fraction (target >= 0.9970).
    pub refrigeration_state_retention_fraction: f64,
    /// Topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-element thermal isolation in decibels (target >= 55.0 dB).
    pub inter_element_thermal_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
