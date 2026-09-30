#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Superconducting
//! Anyon Interferometer & Non-Abelian Parity Qubit Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous acoustically driven superconducting anyon interferometer and non-Abelian
/// parity qubit engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SuperconductingAnyonInterferometerParams {
    /// Interferometer anyon coupling energy in meV (clamp 1.0 to 35.0, default 35.0).
    pub interferometer_coupling_mev: f64,
    /// Topological anyon gap energy in meV (clamp 2.0 to 45.0, default 44.0).
    pub topological_anyon_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 12.0).
    pub acoustic_drive_frequency_ghz: f64,
    /// Anyon interferometer dispatch speed in m/s (clamp 200.0 to 3000.0, default 3000.0).
    pub interferometer_dispatch_speed_m_per_s: f64,
    /// Cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave probe power in microwatts (clamp 0.5 to 30.0, default 16.5).
    pub microwave_probe_power_uw: f64,
    /// Synthetic interferometer arms multiplexing factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_interferometer_arms_factor: f64,
    /// Anyon arm pitch in micrometers (clamp 0.5 to 20.0, default 15.5).
    pub anyon_arm_pitch_um: f64,
}

impl Default for SuperconductingAnyonInterferometerParams {
    fn default() -> Self {
        Self {
            interferometer_coupling_mev: 35.0,
            topological_anyon_gap_mev: 44.0,
            acoustic_drive_frequency_ghz: 12.0,
            interferometer_dispatch_speed_m_per_s: 3000.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 16.5,
            synthetic_interferometer_arms_factor: 4.0,
            anyon_arm_pitch_um: 15.5,
        }
    }
}

impl SuperconductingAnyonInterferometerParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        interferometer_coupling_mev: f64,
        topological_anyon_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        interferometer_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_interferometer_arms_factor: f64,
        anyon_arm_pitch_um: f64,
    ) -> Self {
        Self {
            interferometer_coupling_mev: interferometer_coupling_mev.clamp(1.0, 35.0),
            topological_anyon_gap_mev: topological_anyon_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            interferometer_dispatch_speed_m_per_s: interferometer_dispatch_speed_m_per_s
                .clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_interferometer_arms_factor: synthetic_interferometer_arms_factor
                .clamp(1.0, 8.0),
            anyon_arm_pitch_um: anyon_arm_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Acoustically Driven Superconducting Anyon Interferometer & Non-Abelian
/// Parity Qubit Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SuperconductingAnyonInterferometerMetrics {
    /// Anyon interferometry fidelity (target >= 0.9980).
    pub anyon_interferometry_fidelity: f64,
    /// Parity qubit retention fraction (target >= 0.9970).
    pub parity_qubit_retention_fraction: f64,
    /// Topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-arm crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_arm_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
