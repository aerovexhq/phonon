#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Generative Inverse-Design Diffusion Engine
//! & Automated Metamaterial Synthesizer.

/// Physical parameter configuration for the universal multi-scale visual CAD studio
/// generative inverse-design diffusion engine and automated metamaterial synthesizer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GenerativeDiffusionParams {
    /// Score-based diffusion coupling energy in meV (clamp 1.0 to 35.0, default 19.0).
    pub diffusion_score_coupling_mev: f64,
    /// Topological diffusion gap energy in meV (clamp 2.0 to 45.0, default 25.0).
    pub topological_diffusion_gap_mev: f64,
    /// Acoustic drive carrier frequency in GHz (clamp 1.0 to 12.0, default 7.0).
    pub acoustic_drive_frequency_ghz: f64,
    /// Denoising dispatch propagation speed in m/s (clamp 200.0 to 3000.0, default 1600.0).
    pub denoising_dispatch_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave diffusion probe diagnostic power in microwatts (clamp 0.5 to 30.0, default 7.0).
    pub microwave_diffusion_probe_power_uw: f64,
    /// Synthetic score steps factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_score_steps_factor: f64,
    /// Metamaterial unit cell spatial pitch in micrometers (clamp 0.5 to 20.0, default 6.0).
    pub metamaterial_cell_pitch_um: f64,
}

impl Default for GenerativeDiffusionParams {
    fn default() -> Self {
        Self {
            diffusion_score_coupling_mev: 19.0,
            topological_diffusion_gap_mev: 25.0,
            acoustic_drive_frequency_ghz: 7.0,
            denoising_dispatch_speed_m_per_s: 1600.0,
            cryogenic_temperature_mk: 10.0,
            microwave_diffusion_probe_power_uw: 7.0,
            synthetic_score_steps_factor: 4.0,
            metamaterial_cell_pitch_um: 6.0,
        }
    }
}

impl GenerativeDiffusionParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        diffusion_score_coupling_mev: f64,
        topological_diffusion_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        denoising_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_diffusion_probe_power_uw: f64,
        synthetic_score_steps_factor: f64,
        metamaterial_cell_pitch_um: f64,
    ) -> Self {
        Self {
            diffusion_score_coupling_mev: diffusion_score_coupling_mev.clamp(1.0, 35.0),
            topological_diffusion_gap_mev: topological_diffusion_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            denoising_dispatch_speed_m_per_s: denoising_dispatch_speed_m_per_s
                .clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_diffusion_probe_power_uw: microwave_diffusion_probe_power_uw
                .clamp(0.5, 30.0),
            synthetic_score_steps_factor: synthetic_score_steps_factor.clamp(1.0, 8.0),
            metamaterial_cell_pitch_um: metamaterial_cell_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Generative Inverse-Design Diffusion Engine & Automated Metamaterial Synthesizer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GenerativeDiffusionMetrics {
    /// Generative diffusion synthesis fidelity (target >= 0.9980).
    pub diffusion_synthesis_fidelity: f64,
    /// Metamaterial state retention fraction across denoising trajectories (target >= 0.9970).
    pub metamaterial_state_retention_fraction: f64,
    /// Multi-physics topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-mode crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_mode_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
