#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Levitated Diamond
//! Optomechanical Spin Sensor & Micro-Tesla Magnetometer Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous acoustically levitated diamond optomechanical spin sensor and micro-Tesla magnetometer engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LevitatedDiamondMagnetometerParams {
    /// Acoustically levitated diamond spin-strain coupling energy in meV (clamp 1.0 to 35.0, default 33.5).
    pub magnetometer_coupling_mev: f64,
    /// Topological magneto-acoustic gap energy in meV (clamp 2.0 to 45.0, default 39.5).
    pub topological_magneto_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 12.0).
    pub acoustic_drive_frequency_ghz: f64,
    /// Magnetometer state dispatch speed in m/s (clamp 200.0 to 3000.0, default 3000.0).
    pub magnetometer_dispatch_speed_m_per_s: f64,
    /// Cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave control probe power in microwatts (clamp 0.5 to 30.0, default 14.2).
    pub microwave_probe_power_uw: f64,
    /// Synthetic sensor nodes factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_sensor_nodes_factor: f64,
    /// Inter-sensor spacing pitch in micrometers (clamp 0.5 to 20.0, default 13.2).
    pub magnetometer_pitch_um: f64,
}

impl Default for LevitatedDiamondMagnetometerParams {
    fn default() -> Self {
        Self {
            magnetometer_coupling_mev: 33.5,
            topological_magneto_gap_mev: 39.5,
            acoustic_drive_frequency_ghz: 12.0,
            magnetometer_dispatch_speed_m_per_s: 3000.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 14.2,
            synthetic_sensor_nodes_factor: 4.0,
            magnetometer_pitch_um: 13.2,
        }
    }
}

impl LevitatedDiamondMagnetometerParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        magnetometer_coupling_mev: f64,
        topological_magneto_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        magnetometer_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_sensor_nodes_factor: f64,
        magnetometer_pitch_um: f64,
    ) -> Self {
        Self {
            magnetometer_coupling_mev: magnetometer_coupling_mev.clamp(1.0, 35.0),
            topological_magneto_gap_mev: topological_magneto_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            magnetometer_dispatch_speed_m_per_s: magnetometer_dispatch_speed_m_per_s
                .clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_sensor_nodes_factor: synthetic_sensor_nodes_factor.clamp(1.0, 8.0),
            magnetometer_pitch_um: magnetometer_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Acoustically Levitated Diamond Optomechanical Spin Sensor & Micro-Tesla Magnetometer Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LevitatedDiamondMagnetometerMetrics {
    /// Magnetometer sensing fidelity (target >= 0.9980).
    pub magnetometer_sensing_fidelity: f64,
    /// Nitrogen-vacancy spin state retention fraction across synthetic sensor nodes (target >= 0.9970).
    pub spin_state_retention_fraction: f64,
    /// Topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-sensor crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_sensor_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
