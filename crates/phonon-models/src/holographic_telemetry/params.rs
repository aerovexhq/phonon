#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Real-Time Holographic Telemetry Engine
//! & Immersive Spatial CAD Fabric.

/// Physical parameter configuration for the universal multi-scale visual CAD studio
/// real-time holographic telemetry engine and immersive spatial CAD fabric.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HolographicTelemetryParams {
    /// Holographic coupling energy in meV (clamp 1.0 to 35.0, default 18.5).
    pub holographic_coupling_mev: f64,
    /// Topological telemetry gap energy in meV (clamp 2.0 to 45.0, default 24.5).
    pub topological_telemetry_gap_mev: f64,
    /// Acoustic drive carrier frequency in GHz (clamp 1.0 to 12.0, default 6.8).
    pub acoustic_drive_frequency_ghz: f64,
    /// Spatial voxel dispatch propagation speed in m/s (clamp 200.0 to 3000.0, default 1550.0).
    pub spatial_voxel_dispatch_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave field probe diagnostic power in microwatts (clamp 0.5 to 30.0, default 6.8).
    pub microwave_field_probe_power_uw: f64,
    /// Synthetic lightfield depth factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_lightfield_depth_factor: f64,
    /// Voxel grid spatial pitch in micrometers (clamp 0.5 to 20.0, default 5.8).
    pub voxel_grid_pitch_um: f64,
}

impl Default for HolographicTelemetryParams {
    fn default() -> Self {
        Self {
            holographic_coupling_mev: 18.5,
            topological_telemetry_gap_mev: 24.5,
            acoustic_drive_frequency_ghz: 6.8,
            spatial_voxel_dispatch_speed_m_per_s: 1550.0,
            cryogenic_temperature_mk: 10.0,
            microwave_field_probe_power_uw: 6.8,
            synthetic_lightfield_depth_factor: 4.0,
            voxel_grid_pitch_um: 5.8,
        }
    }
}

impl HolographicTelemetryParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        holographic_coupling_mev: f64,
        topological_telemetry_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        spatial_voxel_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_field_probe_power_uw: f64,
        synthetic_lightfield_depth_factor: f64,
        voxel_grid_pitch_um: f64,
    ) -> Self {
        Self {
            holographic_coupling_mev: holographic_coupling_mev.clamp(1.0, 35.0),
            topological_telemetry_gap_mev: topological_telemetry_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            spatial_voxel_dispatch_speed_m_per_s: spatial_voxel_dispatch_speed_m_per_s
                .clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_field_probe_power_uw: microwave_field_probe_power_uw.clamp(0.5, 30.0),
            synthetic_lightfield_depth_factor: synthetic_lightfield_depth_factor.clamp(1.0, 8.0),
            voxel_grid_pitch_um: voxel_grid_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Real-Time Holographic Telemetry Engine & Immersive Spatial CAD Fabric.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HolographicTelemetryMetrics {
    /// Holographic visual rendering fidelity (target >= 0.9980).
    pub holographic_render_fidelity: f64,
    /// Spatial state retention fraction across holographic frame updates (target >= 0.9970).
    pub spatial_state_retention_fraction: f64,
    /// Multi-physics topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-view crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_view_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
