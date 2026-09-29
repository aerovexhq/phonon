#![deny(unsafe_code)]

//! Physical parameters and multi-physics evaluation metrics for non-Hermitian
//! topological acoustic edge solitons and dissipationless phononic shockwave routers.

/// Physical parameter configuration for non-Hermitian topological acoustic edge soliton waveguides.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonHermitianEdgeSolitonParams {
    /// Carrier acoustic frequency in GHz (clamp 1.0 to 12.0, default 3.8).
    pub carrier_frequency_ghz: f64,
    /// Anomalous acoustic dispersion parameter D2 in kHz (clamp 5.0 to 100.0, default 32.0).
    pub dispersion_parameter_d2_khz: f64,
    /// Non-linear Kerr acoustic coefficient in Hz (clamp 0.5 to 50.0, default 12.0).
    pub kerr_nonlinearity_hz: f64,
    /// Non-Hermitian acoustic gain rate in MHz (clamp 1.0 to 40.0, default 18.0).
    pub non_hermitian_gain_mhz: f64,
    /// Non-Hermitian acoustic loss rate in MHz (clamp 1.0 to 40.0, default 18.0).
    pub non_hermitian_loss_mhz: f64,
    /// Peak acoustic soliton pressure amplitude in Pa (clamp 10.0 to 500.0, default 120.0).
    pub soliton_amplitude_pa: f64,
    /// Operating cryostat bath temperature in milli-Kelvin (clamp 1.0 to 50.0, default 15.0).
    pub operating_temp_m_k: f64,
    /// Phononic topological edge waveguide length in micrometers (clamp 50.0 to 1000.0, default 250.0).
    pub waveguide_length_um: f64,
}

impl Default for NonHermitianEdgeSolitonParams {
    fn default() -> Self {
        Self {
            carrier_frequency_ghz: 3.8,
            dispersion_parameter_d2_khz: 32.0,
            kerr_nonlinearity_hz: 12.0,
            non_hermitian_gain_mhz: 18.0,
            non_hermitian_loss_mhz: 18.0,
            soliton_amplitude_pa: 120.0,
            operating_temp_m_k: 15.0,
            waveguide_length_um: 250.0,
        }
    }
}

impl NonHermitianEdgeSolitonParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        carrier_frequency_ghz: f64,
        dispersion_parameter_d2_khz: f64,
        kerr_nonlinearity_hz: f64,
        non_hermitian_gain_mhz: f64,
        non_hermitian_loss_mhz: f64,
        soliton_amplitude_pa: f64,
        operating_temp_m_k: f64,
        waveguide_length_um: f64,
    ) -> Self {
        Self {
            carrier_frequency_ghz: carrier_frequency_ghz.clamp(1.0, 12.0),
            dispersion_parameter_d2_khz: dispersion_parameter_d2_khz.clamp(5.0, 100.0),
            kerr_nonlinearity_hz: kerr_nonlinearity_hz.clamp(0.5, 50.0),
            non_hermitian_gain_mhz: non_hermitian_gain_mhz.clamp(1.0, 40.0),
            non_hermitian_loss_mhz: non_hermitian_loss_mhz.clamp(1.0, 40.0),
            soliton_amplitude_pa: soliton_amplitude_pa.clamp(10.0, 500.0),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 50.0),
            waveguide_length_um: waveguide_length_um.clamp(50.0, 1000.0),
        }
    }
}

/// Multi-physics performance evaluation metrics for non-Hermitian topological acoustic edge solitons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonHermitianEdgeSolitonMetrics {
    /// Dissipationless acoustic soliton transmission fidelity (target >= 0.9920).
    pub soliton_transmission_fidelity: f64,
    /// Non-linear harmonic distortion suppression in dB (target <= -45.0).
    pub harmonic_distortion_db: f64,
    /// Topological backscattering immunity in dB (target >= 35.0).
    pub backscattering_immunity_db: f64,
    /// Soliton temporal pulse width in nanoseconds (target <= 15.0).
    pub soliton_pulse_width_ns: f64,
    /// Spectral Lyapunov dynamic stability exponent (target <= 0.050).
    pub lyapunov_stability_exponent: f64,
    /// Overall physical compliance flag across all roadmap targets.
    pub is_physically_compliant: bool,
}
