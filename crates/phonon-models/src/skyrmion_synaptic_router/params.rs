#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Skyrmion
//! Synaptic Logic Router & Neuromorphic Crossbar Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous acoustically driven skyrmion synaptic logic router and neuromorphic crossbar engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyrmionSynapticRouterParams {
    /// Acoustically driven skyrmion synaptic coupling energy in meV (clamp 1.0 to 35.0, default 34.0).
    pub synaptic_coupling_mev: f64,
    /// Topological synapse bandgap energy in meV (clamp 2.0 to 45.0, default 40.0).
    pub topological_synapse_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 12.0).
    pub acoustic_drive_frequency_ghz: f64,
    /// Synaptic state dispatch speed in m/s (clamp 200.0 to 3000.0, default 3000.0).
    pub synaptic_dispatch_speed_m_per_s: f64,
    /// Cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave control probe power in microwatts (clamp 0.5 to 30.0, default 14.5).
    pub microwave_probe_power_uw: f64,
    /// Synthetic synapse nodes factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_synapse_nodes_factor: f64,
    /// Synaptic crossbar array pitch in micrometers (clamp 0.5 to 20.0, default 13.5).
    pub synaptic_crossbar_pitch_um: f64,
}

impl Default for SkyrmionSynapticRouterParams {
    fn default() -> Self {
        Self {
            synaptic_coupling_mev: 34.0,
            topological_synapse_gap_mev: 40.0,
            acoustic_drive_frequency_ghz: 12.0,
            synaptic_dispatch_speed_m_per_s: 3000.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 14.5,
            synthetic_synapse_nodes_factor: 4.0,
            synaptic_crossbar_pitch_um: 13.5,
        }
    }
}

impl SkyrmionSynapticRouterParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        synaptic_coupling_mev: f64,
        topological_synapse_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        synaptic_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_synapse_nodes_factor: f64,
        synaptic_crossbar_pitch_um: f64,
    ) -> Self {
        Self {
            synaptic_coupling_mev: synaptic_coupling_mev.clamp(1.0, 35.0),
            topological_synapse_gap_mev: topological_synapse_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            synaptic_dispatch_speed_m_per_s: synaptic_dispatch_speed_m_per_s
                .clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_synapse_nodes_factor: synthetic_synapse_nodes_factor.clamp(1.0, 8.0),
            synaptic_crossbar_pitch_um: synaptic_crossbar_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Acoustically Driven Skyrmion Synaptic Logic Router & Neuromorphic Crossbar Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyrmionSynapticRouterMetrics {
    /// Synaptic routing fidelity (target >= 0.9980).
    pub synaptic_routing_fidelity: f64,
    /// Magnetic skyrmion topological state retention fraction (target >= 0.9970).
    pub skyrmion_state_retention_fraction: f64,
    /// Topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-synapse crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_synapse_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
