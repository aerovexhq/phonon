#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Optomechanical Superradiance Lattice &
//! Chiral Phonon Laser Array Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous optomechanical superradiance lattice and chiral phonon laser array engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SuperradianceLaserParams {
    /// Collective Dicke superradiance and optomechanical coupling energy in meV (clamp 1.0 to 35.0, default 22.5).
    pub superradiance_coupling_mev: f64,
    /// Topological chiral laser protection bandgap energy in meV (clamp 2.0 to 45.0, default 28.5).
    pub topological_laser_gap_mev: f64,
    /// Acoustic phonon drive carrier frequency in GHz (clamp 1.0 to 12.0, default 8.8).
    pub acoustic_drive_frequency_ghz: f64,
    /// Stimulated acoustic phonon emission and dispatch speed in m/s (clamp 200.0 to 3000.0, default 1950.0).
    pub stimulated_emission_dispatch_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Dispersive microwave readout probe power in microwatts (clamp 0.5 to 30.0, default 8.8).
    pub microwave_probe_power_uw: f64,
    /// Synthetic chiral phonon laser emitters enhancement factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_laser_emitters_factor: f64,
    /// Chiral phonon laser array emitter pitch in micrometers (clamp 0.5 to 20.0, default 7.8).
    pub laser_array_pitch_um: f64,
}

impl Default for SuperradianceLaserParams {
    fn default() -> Self {
        Self {
            superradiance_coupling_mev: 22.5,
            topological_laser_gap_mev: 28.5,
            acoustic_drive_frequency_ghz: 8.8,
            stimulated_emission_dispatch_speed_m_per_s: 1950.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 8.8,
            synthetic_laser_emitters_factor: 4.0,
            laser_array_pitch_um: 7.8,
        }
    }
}

impl SuperradianceLaserParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        superradiance_coupling_mev: f64,
        topological_laser_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        stimulated_emission_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_laser_emitters_factor: f64,
        laser_array_pitch_um: f64,
    ) -> Self {
        Self {
            superradiance_coupling_mev: superradiance_coupling_mev.clamp(1.0, 35.0),
            topological_laser_gap_mev: topological_laser_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            stimulated_emission_dispatch_speed_m_per_s: stimulated_emission_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_laser_emitters_factor: synthetic_laser_emitters_factor.clamp(1.0, 8.0),
            laser_array_pitch_um: laser_array_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Optomechanical Superradiance Lattice & Chiral Phonon Laser Array Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SuperradianceLaserMetrics {
    /// Stimulated lasing emission fidelity across chiral phonon laser arrays (target >= 0.9980).
    pub lasing_emission_fidelity: f64,
    /// Quantum phonon state retention fraction across superradiance lattices (target >= 0.9970).
    pub phonon_state_retention_fraction: f64,
    /// Multi-physics topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-mode crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_mode_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
