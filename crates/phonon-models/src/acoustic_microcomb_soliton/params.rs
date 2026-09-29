//! Physical parameter models and evaluation metrics for quantum acoustic
//! frequency combs and phononic microresonator soliton synthesizers.

/// Physical parameter configuration for phononic microresonators and dissipative
/// acoustic Kerr soliton synthesizers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcousticMicrocombParams {
    /// Microresonator outer radius in micrometers (clamp 10.0 to 500.0, default 85.0).
    pub microresonator_radius_um: f64,
    /// Fundamental acoustic resonance frequency in GHz (clamp 0.5 to 15.0, default 2.4).
    pub fundamental_resonance_ghz: f64,
    /// Intrinsic acoustic quality factor (clamp 1.0e5 to 1.0e8, default 2.5e6).
    pub acoustic_quality_factor: f64,
    /// Non-linear phononic Kerr coefficient in Hz (clamp 0.01 to 50.0, default 3.5).
    pub kerr_nonlinearity_hz: f64,
    /// Second-order anomalous dispersion parameter D2 in kHz (clamp 1.0 to 500.0, default 45.0).
    pub dispersion_parameter_d2_khz: f64,
    /// Phononic pump drive power in milliwatts (clamp 0.1 to 100.0, default 12.0).
    pub pump_power_mw: f64,
    /// Effective normalized pump-cavity detuning ratio (clamp 0.5 to 10.0, default 2.8).
    pub laser_detuning_ratio: f64,
    /// Cryogenic operating temperature in milliKelvin (clamp 1.0 to 1000.0, default 20.0).
    pub operating_temp_m_k: f64,
}

impl Default for AcousticMicrocombParams {
    fn default() -> Self {
        Self {
            microresonator_radius_um: 85.0,
            fundamental_resonance_ghz: 2.4,
            acoustic_quality_factor: 2.5e6,
            kerr_nonlinearity_hz: 3.5,
            dispersion_parameter_d2_khz: 45.0,
            pump_power_mw: 12.0,
            laser_detuning_ratio: 2.8,
            operating_temp_m_k: 20.0,
        }
    }
}

impl AcousticMicrocombParams {
    /// Creates a new parameter configuration with physical clamping bounds.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        microresonator_radius_um: f64,
        fundamental_resonance_ghz: f64,
        acoustic_quality_factor: f64,
        kerr_nonlinearity_hz: f64,
        dispersion_parameter_d2_khz: f64,
        pump_power_mw: f64,
        laser_detuning_ratio: f64,
        operating_temp_m_k: f64,
    ) -> Self {
        Self {
            microresonator_radius_um: microresonator_radius_um.clamp(10.0, 500.0),
            fundamental_resonance_ghz: fundamental_resonance_ghz.clamp(0.5, 15.0),
            acoustic_quality_factor: acoustic_quality_factor.clamp(1.0e5, 1.0e8),
            kerr_nonlinearity_hz: kerr_nonlinearity_hz.clamp(0.01, 50.0),
            dispersion_parameter_d2_khz: dispersion_parameter_d2_khz.clamp(1.0, 500.0),
            pump_power_mw: pump_power_mw.clamp(0.1, 100.0),
            laser_detuning_ratio: laser_detuning_ratio.clamp(0.5, 10.0),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 1000.0),
        }
    }
}

/// Multi-physics evaluation metrics for dissipative acoustic Kerr microcombs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcousticMicrocombMetrics {
    /// Comb repetition rate (free spectral range) in GHz (target >= 1.0 GHz).
    pub comb_repetition_rate_ghz: f64,
    /// Fractional line spacing stability Delta f_rep / f_rep (target <= 1.0e-11).
    pub comb_spacing_stability: f64,
    /// Pump-to-soliton conversion efficiency fraction (target >= 0.350 or 35.0%).
    pub conversion_efficiency: f64,
    /// Single-sideband phase noise at 10 kHz offset in dBc/Hz (target <= -125.0 dBc/Hz).
    pub phase_noise_at_10khz_dbc: f64,
    /// Optical/acoustic comb spectral span in octaves (target >= 1.00 octaves).
    pub comb_octave_span: f64,
    /// Integrated timing jitter over Nyquist bandwidth in femtoseconds (target <= 5.0 fs).
    pub timing_jitter_fs: f64,
    /// Physical compliance flag confirming all Phase 118 roadmap targets are satisfied.
    pub is_physically_compliant: bool,
}
