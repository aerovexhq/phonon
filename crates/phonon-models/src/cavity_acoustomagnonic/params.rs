//! Multi-physics parameter structures and metric definitions for
//! cavity acoustomagnonic dark matter haloscopes and axion-magnon hybridization.

/// Physical parameter configuration for cavity-enhanced acoustic-magnonic haloscopes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcoustomagnonicParams {
    /// Microwave cavity resonance frequency $f_c$ in GHz (default 10.2 GHz).
    pub cavity_freq_ghz: f64,
    /// Kittel magnon resonance frequency $f_m$ in GHz (default 10.2 GHz).
    pub magnon_freq_ghz: f64,
    /// High-Q acoustic phonon resonance frequency $f_b$ in GHz (default 10.2 GHz).
    pub phonon_freq_ghz: f64,
    /// Microwave photon to magnon dipole coupling rate $g_{cm} / (2\pi)$ in MHz (default 25.0 MHz).
    pub photon_magnon_coupling_mhz: f64,
    /// Magneto-elastic acoustic phonon to magnon coupling rate $g_{ma} / (2\pi)$ in MHz (default 12.5 MHz).
    pub magnon_phonon_coupling_mhz: f64,
    /// Loaded microwave cavity quality factor $Q_c$ (default 1.5e5).
    pub cavity_q_factor: f64,
    /// Intrinsic YIG magnon Gilbert damping coefficient $\alpha$ (default 1.2e-4).
    pub magnon_gilbert_damping: f64,
    /// Acoustic phononic resonator quality factor $Q_b$ (default 2.5e7).
    pub phonon_q_factor: f64,
    /// YIG sphere diameter in micrometers (default 500.0 um).
    pub yig_sphere_diameter_um: f64,
    /// Cryogenic dilution refrigerator physical temperature in milliKelvin (default 20.0 mK).
    pub ambient_temperature_mk: f64,
    /// Quantum parametric amplifier (JTWPA) power gain in dB (default 22.0 dB).
    pub twpa_gain_db: f64,
    /// Secondary cryogenic HEMT amplifier noise temperature in Kelvin (default 2.8 K).
    pub hemt_noise_temp_k: f64,
    /// Radiometer dwell integration time in seconds (default 1.0 s).
    pub integration_time_s: f64,
}

impl Default for AcoustomagnonicParams {
    fn default() -> Self {
        Self {
            cavity_freq_ghz: 10.2,
            magnon_freq_ghz: 10.2,
            phonon_freq_ghz: 10.2,
            photon_magnon_coupling_mhz: 25.0,
            magnon_phonon_coupling_mhz: 12.5,
            cavity_q_factor: 1.5e5,
            magnon_gilbert_damping: 1.2e-4,
            phonon_q_factor: 2.5e7,
            yig_sphere_diameter_um: 500.0,
            ambient_temperature_mk: 20.0,
            twpa_gain_db: 22.0,
            hemt_noise_temp_k: 2.8,
            integration_time_s: 1.0,
        }
    }
}

impl AcoustomagnonicParams {
    /// Creates a new parameter configuration with bounds clamping.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        f_c_ghz: f64,
        f_m_ghz: f64,
        f_b_ghz: f64,
        g_cm_mhz: f64,
        g_ma_mhz: f64,
        q_c: f64,
        alpha: f64,
        q_b: f64,
        d_um: f64,
        t_mk: f64,
        gain_db: f64,
        hemt_k: f64,
        tau_s: f64,
    ) -> Self {
        Self {
            cavity_freq_ghz: f_c_ghz.clamp(1.0, 30.0),
            magnon_freq_ghz: f_m_ghz.clamp(1.0, 30.0),
            phonon_freq_ghz: f_b_ghz.clamp(1.0, 30.0),
            photon_magnon_coupling_mhz: g_cm_mhz.clamp(1.0, 100.0),
            magnon_phonon_coupling_mhz: g_ma_mhz.clamp(1.0, 50.0),
            cavity_q_factor: q_c.clamp(1.0e3, 1.0e8),
            magnon_gilbert_damping: alpha.clamp(1.0e-5, 1.0e-2),
            phonon_q_factor: q_b.clamp(1.0e5, 1.0e10),
            yig_sphere_diameter_um: d_um.clamp(50.0, 5000.0),
            ambient_temperature_mk: t_mk.clamp(1.0, 1000.0),
            twpa_gain_db: gain_db.clamp(10.0, 40.0),
            hemt_noise_temp_k: hemt_k.clamp(0.5, 20.0),
            integration_time_s: tau_s.clamp(0.01, 100.0),
        }
    }
}

/// Multi-physics evaluation metrics for cavity acoustomagnonic haloscopes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcoustomagnonicMetrics {
    /// Acoustomagnonic cooperativity $C_{ma} = \frac{4 g_{ma}^2}{\gamma_m \gamma_b}$ (target >= 150.0).
    pub acoustomagnonic_cooperativity: f64,
    /// Axion-magnon power conversion gain in dB (target >= 22.0 dB).
    pub conversion_gain_db: f64,
    /// Haloscope readout signal-to-noise ratio in dB (target >= 28.0 dB).
    pub haloscope_readout_snr_db: f64,
    /// Dark matter exclusion frequency scan rate in GHz/day (target >= 1.0 GHz/day).
    pub exclusion_scan_rate_ghz_per_day: f64,
    /// Effective total system noise temperature $T_{sys}$ in Kelvin (sub-Kelvin target < 0.50 K).
    pub effective_system_noise_temp_k: f64,
    /// Hybrid polariton mode anti-crossing frequency splitting in MHz.
    pub polariton_splitting_mhz: f64,
    /// Physical compliance verification flag.
    pub is_physically_compliant: bool,
}
