#![deny(unsafe_code)]

//! Physical parameters and multi-physics evaluation metrics for chiral
//! phonon-magnon polariton frequency combs and quantum topological acoustomagnonics.

/// Physical parameter configuration for chiral acoustomagnonic frequency combs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcoustomagnonicCombParams {
    /// Parametric RF pump frequency in GHz (clamp 5.0 to 30.0, default 14.0).
    pub pump_frequency_ghz: f64,
    /// Magnetoelastic coupling strength in MHz (clamp 20.0 to 200.0, default 85.0).
    pub magnetoelastic_coupling_mhz: f64,
    /// Four-wave mixing Kerr nonlinearity in kHz (clamp 1.0 to 50.0, default 15.0).
    pub kerr_nonlinearity_khz: f64,
    /// Dimensionless Gilbert damping coefficient alpha (clamp 1.0e-5 to 1.0e-3, default 1.2e-4).
    pub gilbert_damping_alpha: f64,
    /// Intrinsic acoustic loss rate in MHz (clamp 0.05 to 5.0, default 0.35).
    pub acoustic_loss_rate_mhz: f64,
    /// Cryogenic operating bath temperature in milli-Kelvin (clamp 1.0 to 100.0, default 20.0).
    pub operating_temp_m_k: f64,
    /// RF drive power in milliwatts (clamp 1.0 to 100.0, default 25.0).
    pub rf_drive_power_mw: f64,
    /// Chiral asymmetry ratio eta_chiral (clamp 0.50 to 0.99, default 0.90).
    pub chiral_asymmetry_ratio: f64,
}

impl Default for AcoustomagnonicCombParams {
    fn default() -> Self {
        Self {
            pump_frequency_ghz: 14.0,
            magnetoelastic_coupling_mhz: 85.0,
            kerr_nonlinearity_khz: 15.0,
            gilbert_damping_alpha: 1.2e-4,
            acoustic_loss_rate_mhz: 0.35,
            operating_temp_m_k: 20.0,
            rf_drive_power_mw: 25.0,
            chiral_asymmetry_ratio: 0.90,
        }
    }
}

impl AcoustomagnonicCombParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        pump_frequency_ghz: f64,
        magnetoelastic_coupling_mhz: f64,
        kerr_nonlinearity_khz: f64,
        gilbert_damping_alpha: f64,
        acoustic_loss_rate_mhz: f64,
        operating_temp_m_k: f64,
        rf_drive_power_mw: f64,
        chiral_asymmetry_ratio: f64,
    ) -> Self {
        Self {
            pump_frequency_ghz: pump_frequency_ghz.clamp(5.0, 30.0),
            magnetoelastic_coupling_mhz: magnetoelastic_coupling_mhz.clamp(20.0, 200.0),
            kerr_nonlinearity_khz: kerr_nonlinearity_khz.clamp(1.0, 50.0),
            gilbert_damping_alpha: gilbert_damping_alpha.clamp(1.0e-5, 1.0e-3),
            acoustic_loss_rate_mhz: acoustic_loss_rate_mhz.clamp(0.05, 5.0),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 100.0),
            rf_drive_power_mw: rf_drive_power_mw.clamp(1.0, 100.0),
            chiral_asymmetry_ratio: chiral_asymmetry_ratio.clamp(0.50, 0.99),
        }
    }
}

/// Multi-physics performance evaluation metrics for chiral acoustomagnonic combs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcoustomagnonicCombMetrics {
    /// Four-wave mixing polariton microcomb spectral span in GHz (target >= 60.0).
    pub comb_spectral_span_ghz: f64,
    /// Single-sideband phase noise at 10 kHz offset in dBc/Hz (target <= -125.0).
    pub phase_noise_at_10khz_dbc: f64,
    /// Polariton quantum state conversion efficiency (target >= 0.880).
    pub polariton_conversion_efficiency: f64,
    /// Inter-modal non-reciprocal chiral isolation in dB (target >= 32.0).
    pub inter_modal_isolation_db: f64,
    /// Dimensionless polariton cooperativity C_pol (target >= 80.0).
    pub polariton_cooperativity: f64,
    /// Overall physical compliance flag across all roadmap targets.
    pub is_physically_compliant: bool,
}
