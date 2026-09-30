#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Quantum Dot
//! Spin Qubit Shuttle & Spin-Orbit Logic Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous acoustically driven quantum dot spin qubit shuttle and spin-orbit logic engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumDotSpinShuttleParams {
    /// Acoustic-piezoelectric spin shuttle coupling energy in meV (clamp 1.0 to 35.0, default 31.5).
    pub shuttle_coupling_mev: f64,
    /// Topological spin shuttle protection gap energy in meV (clamp 2.0 to 45.0, default 37.5).
    pub topological_shuttle_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 12.0).
    pub acoustic_drive_frequency_ghz: f64,
    /// Quantum dot shuttle dispatch speed in m/s (clamp 200.0 to 3000.0, default 2850.0).
    pub shuttle_dispatch_speed_m_per_s: f64,
    /// Cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave control probe power in microwatts (clamp 0.5 to 30.0, default 13.2).
    pub microwave_probe_power_uw: f64,
    /// Synthetic multi-rail shuttle channels factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_shuttle_channels_factor: f64,
    /// Inter-channel shuttle pitch in micrometers (clamp 0.5 to 20.0, default 12.2).
    pub shuttle_channel_pitch_um: f64,
}

impl Default for QuantumDotSpinShuttleParams {
    fn default() -> Self {
        Self {
            shuttle_coupling_mev: 31.5,
            topological_shuttle_gap_mev: 37.5,
            acoustic_drive_frequency_ghz: 12.0,
            shuttle_dispatch_speed_m_per_s: 2850.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 13.2,
            synthetic_shuttle_channels_factor: 4.0,
            shuttle_channel_pitch_um: 12.2,
        }
    }
}

impl QuantumDotSpinShuttleParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        shuttle_coupling_mev: f64,
        topological_shuttle_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        shuttle_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_shuttle_channels_factor: f64,
        shuttle_channel_pitch_um: f64,
    ) -> Self {
        Self {
            shuttle_coupling_mev: shuttle_coupling_mev.clamp(1.0, 35.0),
            topological_shuttle_gap_mev: topological_shuttle_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            shuttle_dispatch_speed_m_per_s: shuttle_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_shuttle_channels_factor: synthetic_shuttle_channels_factor.clamp(1.0, 8.0),
            shuttle_channel_pitch_um: shuttle_channel_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Acoustically Driven Quantum Dot Spin Qubit Shuttle & Spin-Orbit Logic Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumDotSpinShuttleMetrics {
    /// Coherent quantum dot spin transportation fidelity (target >= 0.9980).
    pub shuttle_fidelity: f64,
    /// Spin qubit coherence retention fraction across shuttling channels (target >= 0.9970).
    pub coherence_retention_fraction: f64,
    /// Topological acoustic confinement protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-channel crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_channel_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
