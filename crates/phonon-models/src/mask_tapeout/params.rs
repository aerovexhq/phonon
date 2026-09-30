#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Automated GDSII/OASIS Photolithography Mask
//! & Cryogenic Foundry Tapeout Synthesis Engine.

/// Physical parameter configuration for the universal multi-scale visual CAD studio
/// automated GDSII/OASIS photolithography mask generation and cryogenic foundry tapeout synthesis engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaskTapeoutParams {
    /// Mask polygon geometry fracture coupling energy in meV (clamp 1.0 to 35.0, default 19.5).
    pub mask_fracture_coupling_mev: f64,
    /// Topological tapeout bandgap protection energy in meV (clamp 2.0 to 45.0, default 25.5).
    pub topological_tapeout_gap_mev: f64,
    /// Acoustic drive carrier frequency in GHz (clamp 1.0 to 12.0, default 7.2).
    pub acoustic_drive_frequency_ghz: f64,
    /// Polygon rasterization and dispatch speed in m/s (clamp 200.0 to 3000.0, default 1650.0).
    pub polygon_raster_dispatch_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave tapeout probe diagnostic power in microwatts (clamp 0.5 to 30.0, default 7.2).
    pub microwave_tapeout_probe_power_uw: f64,
    /// Synthetic optical proximity correction (OPC) layer factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_opc_layer_factor: f64,
    /// Photolithography mask feature spatial pitch in micrometers (clamp 0.5 to 20.0, default 6.2).
    pub mask_feature_pitch_um: f64,
}

impl Default for MaskTapeoutParams {
    fn default() -> Self {
        Self {
            mask_fracture_coupling_mev: 19.5,
            topological_tapeout_gap_mev: 25.5,
            acoustic_drive_frequency_ghz: 7.2,
            polygon_raster_dispatch_speed_m_per_s: 1650.0,
            cryogenic_temperature_mk: 10.0,
            microwave_tapeout_probe_power_uw: 7.2,
            synthetic_opc_layer_factor: 4.0,
            mask_feature_pitch_um: 6.2,
        }
    }
}

impl MaskTapeoutParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        mask_fracture_coupling_mev: f64,
        topological_tapeout_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        polygon_raster_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_tapeout_probe_power_uw: f64,
        synthetic_opc_layer_factor: f64,
        mask_feature_pitch_um: f64,
    ) -> Self {
        Self {
            mask_fracture_coupling_mev: mask_fracture_coupling_mev.clamp(1.0, 35.0),
            topological_tapeout_gap_mev: topological_tapeout_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            polygon_raster_dispatch_speed_m_per_s: polygon_raster_dispatch_speed_m_per_s
                .clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_tapeout_probe_power_uw: microwave_tapeout_probe_power_uw
                .clamp(0.5, 30.0),
            synthetic_opc_layer_factor: synthetic_opc_layer_factor.clamp(1.0, 8.0),
            mask_feature_pitch_um: mask_feature_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Automated GDSII/OASIS Photolithography Mask & Cryogenic Foundry Tapeout Synthesis Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaskTapeoutMetrics {
    /// Photolithography mask synthesis fidelity (target >= 0.9980).
    pub mask_synthesis_fidelity: f64,
    /// Layout state retention fraction across geometric fracturing (target >= 0.9970).
    pub layout_state_retention_fraction: f64,
    /// Multi-physics topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-layer design rule checking (DRC) crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_layer_drc_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
