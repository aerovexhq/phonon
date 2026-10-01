#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Floquet-Chern
//! Topological Photonic Isolator & Multi-Scale Routing Hub Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous acoustically driven Floquet-Chern topological photonic isolator and
/// multi-scale routing hub engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetChernPhotonicIsolatorParams {
    /// Isolator coupling energy in meV (clamp 1.0 to 35.0, default 35.0).
    pub isolator_coupling_mev: f64,
    /// Topological Chern gap energy in meV (clamp 2.0 to 45.0, default 45.0).
    pub topological_chern_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 12.0).
    pub acoustic_drive_frequency_ghz: f64,
    /// Routing dispatch speed in m/s (clamp 200.0 to 3000.0, default 3000.0).
    pub routing_dispatch_speed_m_per_s: f64,
    /// Cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Optical probe power in microwatts (clamp 0.5 to 30.0, default 26.0).
    pub optical_probe_power_uw: f64,
    /// Synthetic hub ports factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_hub_ports_factor: f64,
    /// Isolator junction pitch in micrometers (clamp 0.5 to 25.0, default 25.0).
    pub isolator_junction_pitch_um: f64,
}

impl Default for FloquetChernPhotonicIsolatorParams {
    fn default() -> Self {
        Self {
            isolator_coupling_mev: 35.0,
            topological_chern_gap_mev: 45.0,
            acoustic_drive_frequency_ghz: 12.0,
            routing_dispatch_speed_m_per_s: 3000.0,
            cryogenic_temperature_mk: 10.0,
            optical_probe_power_uw: 26.0,
            synthetic_hub_ports_factor: 4.0,
            isolator_junction_pitch_um: 25.0,
        }
    }
}

impl FloquetChernPhotonicIsolatorParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        isolator_coupling_mev: f64,
        topological_chern_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        routing_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        optical_probe_power_uw: f64,
        synthetic_hub_ports_factor: f64,
        isolator_junction_pitch_um: f64,
    ) -> Self {
        Self {
            isolator_coupling_mev: isolator_coupling_mev.clamp(1.0, 35.0),
            topological_chern_gap_mev: topological_chern_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            routing_dispatch_speed_m_per_s: routing_dispatch_speed_m_per_s
                .clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            optical_probe_power_uw: optical_probe_power_uw.clamp(0.5, 30.0),
            synthetic_hub_ports_factor: synthetic_hub_ports_factor.clamp(1.0, 8.0),
            isolator_junction_pitch_um: isolator_junction_pitch_um.clamp(0.5, 25.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Acoustically Driven Floquet-Chern Topological Photonic Isolator
/// & Multi-Scale Routing Hub Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetChernPhotonicIsolatorMetrics {
    /// Isolator fidelity (target >= 0.9980).
    pub isolator_fidelity: f64,
    /// Topological state retention fraction (target >= 0.9970).
    pub topological_state_retention_fraction: f64,
    /// Topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Isolation directivity in decibels (target >= 55.0 dB).
    pub isolation_directivity_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
