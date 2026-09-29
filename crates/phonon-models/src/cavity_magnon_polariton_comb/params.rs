//! Parameter models and metric structures for cavity quantum magnon-polariton
//! frequency combs and non-linear squeezed dark matter halometry.

/// Physical parameter configuration for cavity magnon-polariton frequency combs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MagnonPolaritonCombParams {
    /// Center microwave cavity resonance frequency $f_c$ in GHz (default 10.0 GHz).
    pub cavity_freq_ghz: f64,
    /// Ferrimagnetic acoustic breathing mode frequency $\Omega_b$ in MHz (default 25.0 MHz).
    pub breathing_mode_freq_mhz: f64,
    /// Non-linear Kerr self-phase modulation constant $K / (2\pi)$ in Hz (default 0.75 Hz).
    pub kerr_nonlinearity_hz: f64,
    /// Magnetoelastic acoustic-magnon coupling rate $g_{mb} / (2\pi)$ in MHz (default 12.5 MHz).
    pub magnetoelastic_coupling_mhz: f64,
    /// Microwave pump drive power $P_{\text{pump}}$ in milliwatts (default 1.50 mW).
    pub pump_power_mw: f64,
    /// Cavity mode loaded decay rate $\kappa_c / (2\pi)$ in MHz (default 1.20 MHz).
    pub cavity_linewidth_mhz: f64,
    /// Magnon mode intrinsic damping rate $\kappa_m / (2\pi)$ in MHz (default 0.85 MHz).
    pub magnon_linewidth_mhz: f64,
    /// Quantum squeezing parameter of the magnon state $r$ (default 0.88).
    pub squeezing_param_r: f64,
    /// Cryogenic operating temperature in Kelvin (default 0.020 K for dilution fridge).
    pub operating_temp_k: f64,
}

impl Default for MagnonPolaritonCombParams {
    fn default() -> Self {
        Self {
            cavity_freq_ghz: 10.0,
            breathing_mode_freq_mhz: 25.0,
            kerr_nonlinearity_hz: 0.75,
            magnetoelastic_coupling_mhz: 12.5,
            pump_power_mw: 1.50,
            cavity_linewidth_mhz: 1.20,
            magnon_linewidth_mhz: 0.85,
            squeezing_param_r: 0.88,
            operating_temp_k: 0.020,
        }
    }
}

impl MagnonPolaritonCombParams {
    /// Creates a new parameter configuration with bounds clamping.
    pub fn new(
        f_c_ghz: f64,
        f_b_mhz: f64,
        k_hz: f64,
        g_mb_mhz: f64,
        p_pump_mw: f64,
        kappa_c_mhz: f64,
        kappa_m_mhz: f64,
        r: f64,
        temp_k: f64,
    ) -> Self {
        Self {
            cavity_freq_ghz: f_c_ghz.clamp(1.0, 50.0),
            breathing_mode_freq_mhz: f_b_mhz.clamp(0.5, 1000.0),
            kerr_nonlinearity_hz: k_hz.clamp(0.01, 100.0),
            magnetoelastic_coupling_mhz: g_mb_mhz.clamp(0.1, 100.0),
            pump_power_mw: p_pump_mw.clamp(0.01, 100.0),
            cavity_linewidth_mhz: kappa_c_mhz.clamp(0.01, 50.0),
            magnon_linewidth_mhz: kappa_m_mhz.clamp(0.01, 50.0),
            squeezing_param_r: r.clamp(0.0, 3.5),
            operating_temp_k: temp_k.clamp(0.001, 300.0),
        }
    }
}

/// Multi-physics evaluation metrics for cavity magnon-polariton frequency combs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MagnonPolaritonCombMetrics {
    /// Generated frequency comb spectral span in octaves (target >= 1.50 octaves).
    pub comb_octave_span: f64,
    /// Sub-shot-noise halometer sensitivity improvement beyond SQL in dB (target >= 6.00 dB).
    pub sub_shot_noise_improvement_db: f64,
    /// Continuous-variable polariton entanglement logarithmic negativity $E_N$ (target >= 0.850).
    pub polariton_log_negativity: f64,
    /// Parametric comb generation threshold pump power $P_{\text{th}}$ in mW (target <= 1.00 mW).
    pub comb_threshold_power_mw: f64,
    /// Discrete comb teeth count generated across the spectral bandwidth (target >= 40).
    pub comb_teeth_count: usize,
    /// Physical compliance verification flag.
    pub is_physically_compliant: bool,
}
