#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Levitated Nanoparticle
//! Metrology & Quantum Force Sensor Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous acoustically levitated nanoparticle metrology and quantum force sensor engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcousticallyLevitatedNanoparticleParams {
    /// Optical-acoustic levitation coupling energy in meV (clamp 1.0 to 35.0, default 31.0).
    pub levitation_coupling_mev: f64,
    /// Topological force gap energy in meV (clamp 2.0 to 45.0, default 37.0).
    pub topological_force_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 12.0).
    pub acoustic_drive_frequency_ghz: f64,
    /// Levitation dispatch speed in m/s (clamp 200.0 to 3000.0, default 2800.0).
    pub levitation_dispatch_speed_m_per_s: f64,
    /// Cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave probe power in microwatts (clamp 0.5 to 30.0, default 13.0).
    pub microwave_probe_power_uw: f64,
    /// Synthetic trap nodes factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_trap_nodes_factor: f64,
    /// Levitation trap pitch in micrometers (clamp 0.5 to 20.0, default 12.0).
    pub levitation_trap_pitch_um: f64,
}

impl Default for AcousticallyLevitatedNanoparticleParams {
    fn default() -> Self {
        Self {
            levitation_coupling_mev: 31.0,
            topological_force_gap_mev: 37.0,
            acoustic_drive_frequency_ghz: 12.0,
            levitation_dispatch_speed_m_per_s: 2800.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 13.0,
            synthetic_trap_nodes_factor: 4.0,
            levitation_trap_pitch_um: 12.0,
        }
    }
}

impl AcousticallyLevitatedNanoparticleParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        levitation_coupling_mev: f64,
        topological_force_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        levitation_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_trap_nodes_factor: f64,
        levitation_trap_pitch_um: f64,
    ) -> Self {
        Self {
            levitation_coupling_mev: levitation_coupling_mev.clamp(1.0, 35.0),
            topological_force_gap_mev: topological_force_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            levitation_dispatch_speed_m_per_s: levitation_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_trap_nodes_factor: synthetic_trap_nodes_factor.clamp(1.0, 8.0),
            levitation_trap_pitch_um: levitation_trap_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Acoustically Levitated Nanoparticle Metrology & Quantum Force Sensor Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcousticallyLevitatedNanoparticleMetrics {
    /// Ultrasensitive force measurement fidelity (target >= 0.9980).
    pub force_sensitivity_fidelity: f64,
    /// Center-of-mass quantum coherent state retention fraction (target >= 0.9970).
    pub coherent_state_retention_fraction: f64,
    /// Topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-trap crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_trap_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
