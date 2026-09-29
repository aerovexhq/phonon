#![deny(unsafe_code)]

//! Physical parameters and metrics configuration for topological acoustic
//! higher-order corner mode lasers and non-Hermitian phonon cavities.

/// Physical parameter configuration for topological acoustic higher-order corner mode lasers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalCornerLaserParams {
    /// Acoustic operating frequency in GHz (clamp 1.0 to 15.0, default 4.5).
    pub acoustic_frequency_ghz: f64,
    /// Inter-cell acoustic coupling hopping rate in MHz (clamp 10.0 to 100.0, default 45.0).
    pub inter_cell_hopping_mhz: f64,
    /// Intra-cell acoustic coupling hopping rate in MHz (clamp 2.0 to 40.0, default 12.0).
    pub intra_cell_hopping_mhz: f64,
    /// Optical pump power in micro-Watts (clamp 5.0 to 100.0, default 25.0).
    pub optical_pump_power_uw: f64,
    /// Non-Hermitian localized acoustic gain rate in MHz (clamp 1.0 to 30.0, default 15.0).
    pub non_hermitian_gain_mhz: f64,
    /// Phononic intrinsic and radiative acoustic loss rate in MHz (clamp 0.5 to 10.0, default 2.5).
    pub acoustic_loss_rate_mhz: f64,
    /// Cryogenic operating temperature in milli-Kelvin (clamp 1.0 to 50.0, default 15.0).
    pub operating_temp_m_k: f64,
    /// Metamaterial fabrication disorder amplitude percentage (clamp 0.0 to 10.0, default 2.0).
    pub disorder_amplitude_percent: f64,
}

impl Default for TopologicalCornerLaserParams {
    fn default() -> Self {
        Self {
            acoustic_frequency_ghz: 4.5,
            inter_cell_hopping_mhz: 45.0,
            intra_cell_hopping_mhz: 12.0,
            optical_pump_power_uw: 25.0,
            non_hermitian_gain_mhz: 15.0,
            acoustic_loss_rate_mhz: 2.5,
            operating_temp_m_k: 15.0,
            disorder_amplitude_percent: 2.0,
        }
    }
}

impl TopologicalCornerLaserParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        acoustic_frequency_ghz: f64,
        inter_cell_hopping_mhz: f64,
        intra_cell_hopping_mhz: f64,
        optical_pump_power_uw: f64,
        non_hermitian_gain_mhz: f64,
        acoustic_loss_rate_mhz: f64,
        operating_temp_m_k: f64,
        disorder_amplitude_percent: f64,
    ) -> Self {
        Self {
            acoustic_frequency_ghz: acoustic_frequency_ghz.clamp(1.0, 15.0),
            inter_cell_hopping_mhz: inter_cell_hopping_mhz.clamp(10.0, 100.0),
            intra_cell_hopping_mhz: intra_cell_hopping_mhz.clamp(2.0, 40.0),
            optical_pump_power_uw: optical_pump_power_uw.clamp(5.0, 100.0),
            non_hermitian_gain_mhz: non_hermitian_gain_mhz.clamp(1.0, 30.0),
            acoustic_loss_rate_mhz: acoustic_loss_rate_mhz.clamp(0.5, 10.0),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 50.0),
            disorder_amplitude_percent: disorder_amplitude_percent.clamp(0.0, 10.0),
        }
    }
}

/// Multi-physics evaluation metrics for topological acoustic higher-order corner mode lasers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalCornerLaserMetrics {
    /// Corner mode lasing slope efficiency above threshold (target >= 0.750).
    pub corner_lasing_efficiency: f64,
    /// Threshold optical pump power in micro-Watts (target <= 10.0).
    pub threshold_power_uw: f64,
    /// Spatial energy localization fraction within the corner unit cell (target >= 0.920).
    pub corner_mode_localization: f64,
    /// Non-Hermitian mode discrimination suppressing edge and bulk modes in dB (target >= 25.0).
    pub mode_discrimination_db: f64,
    /// Coherent acoustic phonon emission linewidth in kHz (target <= 5.0).
    pub emission_linewidth_khz: f64,
    /// Overall physical compliance flag across all roadmap performance targets.
    pub is_physically_compliant: bool,
}
