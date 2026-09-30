#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Topological
//! Valley-Hall Photonic Waveguide & Chiral Quantum Network Router Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous acoustically driven topological valley-Hall photonic waveguide
/// and chiral quantum network router engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalValleyHallRouterParams {
    /// Waveguide coupling energy in meV (clamp 1.0 to 35.0, default 35.0).
    pub waveguide_coupling_mev: f64,
    /// Topological valley gap energy in meV (clamp 2.0 to 45.0, default 45.0).
    pub topological_valley_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 12.0).
    pub acoustic_drive_frequency_ghz: f64,
    /// Routing dispatch speed in m/s (clamp 200.0 to 3000.0, default 3000.0).
    pub routing_dispatch_speed_m_per_s: f64,
    /// Cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Optical probe power in microwatts (clamp 0.5 to 30.0, default 19.5).
    pub optical_probe_power_uw: f64,
    /// Synthetic router nodes factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_router_nodes_factor: f64,
    /// Waveguide pitch in micrometers (clamp 0.5 to 20.0, default 18.5).
    pub waveguide_pitch_um: f64,
}

impl Default for TopologicalValleyHallRouterParams {
    fn default() -> Self {
        Self {
            waveguide_coupling_mev: 35.0,
            topological_valley_gap_mev: 45.0,
            acoustic_drive_frequency_ghz: 12.0,
            routing_dispatch_speed_m_per_s: 3000.0,
            cryogenic_temperature_mk: 10.0,
            optical_probe_power_uw: 19.5,
            synthetic_router_nodes_factor: 4.0,
            waveguide_pitch_um: 18.5,
        }
    }
}

impl TopologicalValleyHallRouterParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        waveguide_coupling_mev: f64,
        topological_valley_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        routing_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        optical_probe_power_uw: f64,
        synthetic_router_nodes_factor: f64,
        waveguide_pitch_um: f64,
    ) -> Self {
        Self {
            waveguide_coupling_mev: waveguide_coupling_mev.clamp(1.0, 35.0),
            topological_valley_gap_mev: topological_valley_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            routing_dispatch_speed_m_per_s: routing_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            optical_probe_power_uw: optical_probe_power_uw.clamp(0.5, 30.0),
            synthetic_router_nodes_factor: synthetic_router_nodes_factor.clamp(1.0, 8.0),
            waveguide_pitch_um: waveguide_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Acoustically Driven Topological Valley-Hall Photonic Waveguide
/// & Chiral Quantum Network Router Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalValleyHallRouterMetrics {
    /// Valley-Hall routing fidelity (target >= 0.9980).
    pub valley_hall_routing_fidelity: f64,
    /// Valley state retention fraction (target >= 0.9970).
    pub valley_state_retention_fraction: f64,
    /// Topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-channel crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_channel_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
