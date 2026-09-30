#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Topological Axion
//! Waveguide & Chiral Anomaly Synthesizer.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous acoustically driven topological axion waveguide and chiral anomaly synthesizer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalAxionParams {
    /// Dynamic axion electrodynamic coupling energy in meV (clamp 1.0 to 35.0, default 23.0).
    pub axion_coupling_mev: f64,
    /// Topological axion protection bandgap energy in meV (clamp 2.0 to 45.0, default 29.0).
    pub topological_axion_gap_mev: f64,
    /// Acoustic phonon drive carrier frequency in GHz (clamp 1.0 to 12.0, default 9.0).
    pub acoustic_drive_frequency_ghz: f64,
    /// Chiral anomaly acoustic transport dispatch speed in m/s (clamp 200.0 to 3000.0, default 2000.0).
    pub chiral_anomaly_dispatch_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Dispersive microwave readout probe power in microwatts (clamp 0.5 to 30.0, default 9.0).
    pub microwave_probe_power_uw: f64,
    /// Synthetic topological axion layers enhancement factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_axion_layers_factor: f64,
    /// Topological axion waveguide pitch in micrometers (clamp 0.5 to 20.0, default 8.0).
    pub waveguide_pitch_um: f64,
}

impl Default for TopologicalAxionParams {
    fn default() -> Self {
        Self {
            axion_coupling_mev: 23.0,
            topological_axion_gap_mev: 29.0,
            acoustic_drive_frequency_ghz: 9.0,
            chiral_anomaly_dispatch_speed_m_per_s: 2000.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 9.0,
            synthetic_axion_layers_factor: 4.0,
            waveguide_pitch_um: 8.0,
        }
    }
}

impl TopologicalAxionParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        axion_coupling_mev: f64,
        topological_axion_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        chiral_anomaly_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_axion_layers_factor: f64,
        waveguide_pitch_um: f64,
    ) -> Self {
        Self {
            axion_coupling_mev: axion_coupling_mev.clamp(1.0, 35.0),
            topological_axion_gap_mev: topological_axion_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            chiral_anomaly_dispatch_speed_m_per_s: chiral_anomaly_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_axion_layers_factor: synthetic_axion_layers_factor.clamp(1.0, 8.0),
            waveguide_pitch_um: waveguide_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Acoustically Driven Topological Axion Waveguide & Chiral Anomaly Synthesizer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalAxionMetrics {
    /// Chiral transport fidelity across topological axion waveguides (target >= 0.9980).
    pub chiral_transport_fidelity: f64,
    /// Axion-polariton quantum state retention fraction (target >= 0.9970).
    pub axion_polariton_retention_fraction: f64,
    /// Multi-physics topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Non-reciprocal isolation in decibels (target >= 55.0 dB).
    pub non_reciprocal_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
