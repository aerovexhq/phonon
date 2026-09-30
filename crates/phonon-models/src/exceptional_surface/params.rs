#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Non-Hermitian Exceptional Surface Sensor
//! & Hypersensitive Phononic Metrology Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous non-Hermitian exceptional surface sensor and hypersensitive phononic metrology engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExceptionalSurfaceParams {
    /// Non-Hermitian exceptional surface coupling energy in meV (clamp 1.0 to 35.0, default 23.5).
    pub surface_coupling_mev: f64,
    /// Topological surface bandgap energy in meV (clamp 2.0 to 45.0, default 29.5).
    pub topological_surface_gap_mev: f64,
    /// Acoustic drive carrier frequency in GHz (clamp 1.0 to 12.0, default 9.2).
    pub acoustic_drive_frequency_ghz: f64,
    /// Metrology perturbation dispatch speed in m/s (clamp 200.0 to 3000.0, default 2050.0).
    pub metrology_dispatch_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Dispersive microwave readout probe power in microwatts (clamp 0.5 to 30.0, default 9.2).
    pub microwave_probe_power_uw: f64,
    /// Synthetic exceptional surface sensors enhancement factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_sensors_factor: f64,
    /// Hypersensitive metrology sensor pitch in micrometers (clamp 0.5 to 20.0, default 8.2).
    pub sensor_pitch_um: f64,
}

impl Default for ExceptionalSurfaceParams {
    fn default() -> Self {
        Self {
            surface_coupling_mev: 23.5,
            topological_surface_gap_mev: 29.5,
            acoustic_drive_frequency_ghz: 9.2,
            metrology_dispatch_speed_m_per_s: 2050.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 9.2,
            synthetic_sensors_factor: 4.0,
            sensor_pitch_um: 8.2,
        }
    }
}

impl ExceptionalSurfaceParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        surface_coupling_mev: f64,
        topological_surface_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        metrology_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_sensors_factor: f64,
        sensor_pitch_um: f64,
    ) -> Self {
        Self {
            surface_coupling_mev: surface_coupling_mev.clamp(1.0, 35.0),
            topological_surface_gap_mev: topological_surface_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            metrology_dispatch_speed_m_per_s: metrology_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_sensors_factor: synthetic_sensors_factor.clamp(1.0, 8.0),
            sensor_pitch_um: sensor_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Non-Hermitian Exceptional Surface Sensor & Hypersensitive Phononic Metrology Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExceptionalSurfaceMetrics {
    /// Metrology fidelity across exceptional surface sensors (target >= 0.9980).
    pub metrology_fidelity: f64,
    /// Surface state quantum retention fraction (target >= 0.9970).
    pub surface_state_retention_fraction: f64,
    /// Multi-physics topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-sensor crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_sensor_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
