#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Distributed Multi-Cluster Simulation Mesh
//! & Cloud Synthesis Fabric.

/// Physical parameter configuration for the universal multi-scale visual CAD studio
/// distributed multi-cluster simulation mesh and cloud synthesis fabric.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistributedMeshParams {
    /// Cluster node coupling energy in meV (clamp 1.0 to 35.0, default 17.5).
    pub cluster_node_coupling_mev: f64,
    /// Topological cluster gap energy in meV (clamp 2.0 to 45.0, default 23.5).
    pub topological_cluster_gap_mev: f64,
    /// Acoustic drive carrier frequency in GHz (clamp 1.0 to 12.0, default 6.2).
    pub acoustic_drive_frequency_ghz: f64,
    /// Federation streaming speed in m/s (clamp 200.0 to 3000.0, default 1450.0).
    pub federation_streaming_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave mesh diagnostic power in microwatts (clamp 0.5 to 30.0, default 6.2).
    pub microwave_mesh_power_uw: f64,
    /// Synthetic spatial domain decomposition count (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_spatial_domains: f64,
    /// Domain boundary spatial pitch in micrometers (clamp 0.5 to 20.0, default 5.2).
    pub domain_boundary_pitch_um: f64,
}

impl Default for DistributedMeshParams {
    fn default() -> Self {
        Self {
            cluster_node_coupling_mev: 17.5,
            topological_cluster_gap_mev: 23.5,
            acoustic_drive_frequency_ghz: 6.2,
            federation_streaming_speed_m_per_s: 1450.0,
            cryogenic_temperature_mk: 10.0,
            microwave_mesh_power_uw: 6.2,
            synthetic_spatial_domains: 4.0,
            domain_boundary_pitch_um: 5.2,
        }
    }
}

impl DistributedMeshParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        cluster_node_coupling_mev: f64,
        topological_cluster_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        federation_streaming_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_mesh_power_uw: f64,
        synthetic_spatial_domains: f64,
        domain_boundary_pitch_um: f64,
    ) -> Self {
        Self {
            cluster_node_coupling_mev: cluster_node_coupling_mev.clamp(1.0, 35.0),
            topological_cluster_gap_mev: topological_cluster_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            federation_streaming_speed_m_per_s: federation_streaming_speed_m_per_s
                .clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_mesh_power_uw: microwave_mesh_power_uw.clamp(0.5, 30.0),
            synthetic_spatial_domains: synthetic_spatial_domains.clamp(1.0, 8.0),
            domain_boundary_pitch_um: domain_boundary_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Distributed Multi-Cluster Simulation Mesh & Cloud Synthesis Fabric.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistributedMeshMetrics {
    /// Multi-cluster distributed mesh synchronization fidelity (target >= 0.9980).
    pub mesh_sync_fidelity: f64,
    /// Distributed state retention fraction across cluster federation cycles (target >= 0.9970).
    pub distributed_state_retention_fraction: f64,
    /// Multi-physics topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-cluster crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_cluster_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
