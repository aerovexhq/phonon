#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Skyrmionic-Phononic Memory Lattice &
//! Chiral Domain Wall Track Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous skyrmionic-phononic memory lattice and chiral domain wall track engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyrmionicMemoryParams {
    /// Acoustic-skyrmion exchange coupling energy in meV (clamp 1.0 to 35.0, default 24.5).
    pub skyrmion_coupling_mev: f64,
    /// Topological skyrmion excitation bandgap energy in meV (clamp 2.0 to 45.0, default 30.5).
    pub topological_skyrmion_gap_mev: f64,
    /// Acoustic drive carrier frequency in GHz (clamp 1.0 to 12.0, default 9.8).
    pub acoustic_drive_frequency_ghz: f64,
    /// Chiral domain wall racetrack dispatch speed in m/s (clamp 200.0 to 3000.0, default 2150.0).
    pub racetrack_dispatch_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave readout probe power in microwatts (clamp 0.5 to 30.0, default 9.8).
    pub microwave_probe_power_uw: f64,
    /// Synthetic domain wall track multiplication factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_tracks_factor: f64,
    /// Skyrmionic racetrack pitch in micrometers (clamp 0.5 to 20.0, default 8.8).
    pub track_pitch_um: f64,
}

impl Default for SkyrmionicMemoryParams {
    fn default() -> Self {
        Self {
            skyrmion_coupling_mev: 24.5,
            topological_skyrmion_gap_mev: 30.5,
            acoustic_drive_frequency_ghz: 9.8,
            racetrack_dispatch_speed_m_per_s: 2150.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 9.8,
            synthetic_tracks_factor: 4.0,
            track_pitch_um: 8.8,
        }
    }
}

impl SkyrmionicMemoryParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        skyrmion_coupling_mev: f64,
        topological_skyrmion_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        racetrack_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_tracks_factor: f64,
        track_pitch_um: f64,
    ) -> Self {
        Self {
            skyrmion_coupling_mev: skyrmion_coupling_mev.clamp(1.0, 35.0),
            topological_skyrmion_gap_mev: topological_skyrmion_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            racetrack_dispatch_speed_m_per_s: racetrack_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_tracks_factor: synthetic_tracks_factor.clamp(1.0, 8.0),
            track_pitch_um: track_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Skyrmionic-Phononic Memory Lattice & Chiral Domain Wall Track Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyrmionicMemoryMetrics {
    /// Skyrmion nucleation and pinning gate fidelity (target >= 0.9980).
    pub nucleation_fidelity: f64,
    /// Skyrmion topological state non-volatile retention fraction (target >= 0.9970).
    pub skyrmion_state_retention_fraction: f64,
    /// Multi-physics topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-track crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_track_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
