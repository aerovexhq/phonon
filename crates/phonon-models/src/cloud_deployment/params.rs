#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Silicon-to-Cloud Deployment Gateway &
//! Production Digital Twin Cloud Fabric.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous silicon-to-cloud deployment gateway and production digital twin cloud fabric.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CloudDeploymentParams {
    /// Cloud deployment coupling energy in meV (clamp 1.0 to 35.0, default 21.0).
    pub deployment_coupling_mev: f64,
    /// Topological deployment bandgap energy in meV (clamp 2.0 to 45.0, default 27.0).
    pub topological_deployment_gap_mev: f64,
    /// Acoustic drive carrier frequency in GHz (clamp 1.0 to 12.0, default 8.0).
    pub acoustic_drive_frequency_ghz: f64,
    /// Real-time stream telemetry dispatch speed in m/s (clamp 200.0 to 3000.0, default 1800.0).
    pub stream_dispatch_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Dispersive microwave readout probe power in microwatts (clamp 0.5 to 30.0, default 8.0).
    pub microwave_probe_power_uw: f64,
    /// Synthetic cloud cluster nodes scaling factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_cloud_nodes_factor: f64,
    /// Production gateway interconnect routing pitch in micrometers (clamp 0.5 to 20.0, default 7.0).
    pub gateway_interconnect_pitch_um: f64,
}

impl Default for CloudDeploymentParams {
    fn default() -> Self {
        Self {
            deployment_coupling_mev: 21.0,
            topological_deployment_gap_mev: 27.0,
            acoustic_drive_frequency_ghz: 8.0,
            stream_dispatch_speed_m_per_s: 1800.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 8.0,
            synthetic_cloud_nodes_factor: 4.0,
            gateway_interconnect_pitch_um: 7.0,
        }
    }
}

impl CloudDeploymentParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        deployment_coupling_mev: f64,
        topological_deployment_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        stream_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_cloud_nodes_factor: f64,
        gateway_interconnect_pitch_um: f64,
    ) -> Self {
        Self {
            deployment_coupling_mev: deployment_coupling_mev.clamp(1.0, 35.0),
            topological_deployment_gap_mev: topological_deployment_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            stream_dispatch_speed_m_per_s: stream_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_cloud_nodes_factor: synthetic_cloud_nodes_factor.clamp(1.0, 8.0),
            gateway_interconnect_pitch_um: gateway_interconnect_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Silicon-to-Cloud Deployment Gateway & Production Digital Twin Cloud Fabric.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CloudDeploymentMetrics {
    /// Deployment fidelity across production cloud pipelines (target >= 0.9980).
    pub deployment_fidelity: f64,
    /// Cloud digital twin state retention fraction across stream transactions (target >= 0.9970).
    pub cloud_state_retention_fraction: f64,
    /// Multi-physics topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-node crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_node_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
