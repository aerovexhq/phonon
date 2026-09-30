#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Quantum
//! Metamaterial Frequency-Comb Beamformer & Hyperspectral Lidar Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous acoustically driven quantum metamaterial frequency-comb beamformer
/// and hyperspectral lidar engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumMetamaterialBeamformerParams {
    /// Beamformer coupling energy in meV (clamp 1.0 to 35.0, default 35.0).
    pub beamformer_coupling_mev: f64,
    /// Topological comb gap energy in meV (clamp 2.0 to 45.0, default 45.0).
    pub topological_comb_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 12.0).
    pub acoustic_drive_frequency_ghz: f64,
    /// Beam dispatch speed in m/s (clamp 200.0 to 3000.0, default 3000.0).
    pub beam_dispatch_speed_m_per_s: f64,
    /// Cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Optical probe power in microwatts (clamp 0.5 to 30.0, default 20.8).
    pub optical_probe_power_uw: f64,
    /// Synthetic comb elements factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_comb_elements_factor: f64,
    /// Beamformer pitch in micrometers (clamp 0.5 to 20.0, default 19.8).
    pub beamformer_pitch_um: f64,
}

impl Default for QuantumMetamaterialBeamformerParams {
    fn default() -> Self {
        Self {
            beamformer_coupling_mev: 35.0,
            topological_comb_gap_mev: 45.0,
            acoustic_drive_frequency_ghz: 12.0,
            beam_dispatch_speed_m_per_s: 3000.0,
            cryogenic_temperature_mk: 10.0,
            optical_probe_power_uw: 20.8,
            synthetic_comb_elements_factor: 4.0,
            beamformer_pitch_um: 19.8,
        }
    }
}

impl QuantumMetamaterialBeamformerParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        beamformer_coupling_mev: f64,
        topological_comb_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        beam_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        optical_probe_power_uw: f64,
        synthetic_comb_elements_factor: f64,
        beamformer_pitch_um: f64,
    ) -> Self {
        Self {
            beamformer_coupling_mev: beamformer_coupling_mev.clamp(1.0, 35.0),
            topological_comb_gap_mev: topological_comb_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            beam_dispatch_speed_m_per_s: beam_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            optical_probe_power_uw: optical_probe_power_uw.clamp(0.5, 30.0),
            synthetic_comb_elements_factor: synthetic_comb_elements_factor.clamp(1.0, 8.0),
            beamformer_pitch_um: beamformer_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Acoustically Driven Quantum Metamaterial Frequency-Comb Beamformer &
/// Hyperspectral Lidar Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumMetamaterialBeamformerMetrics {
    /// Beamformer fidelity (target >= 0.9980).
    pub beamformer_fidelity: f64,
    /// Frequency comb state retention fraction (target >= 0.9970).
    pub frequency_comb_state_retention_fraction: f64,
    /// Topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-element crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_element_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
