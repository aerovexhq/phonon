//! Parameter models and metric structures for quantum cavity acoustomechanical
//! squeezing and backaction evasion.

/// Physical parameter configuration for quantum cavity acoustomechanics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcoustomechanicalSqueezingParams {
    /// Phononic crystal membrane mechanical resonance frequency $\Omega_m$ in MHz (default 15.0 MHz).
    pub membrane_freq_mhz: f64,
    /// Optical/microwave cavity resonance frequency $f_{\text{cav}}$ in THz (default 193.4 THz for 1550 nm).
    pub cavity_freq_thz: f64,
    /// Single-photon optomechanical coupling rate $g_0 / (2\pi)$ in Hz (default 250.0 Hz).
    pub single_photon_coupling_hz: f64,
    /// Intracavity coherent photon number $n_c$ (default 6.5e5).
    pub intracavity_photons: f64,
    /// Mechanical membrane acoustic quality factor $Q_m$ (default 1.2e8).
    pub mechanical_q_factor: f64,
    /// Cavity loaded decay linewidth $\kappa / (2\pi)$ in MHz (default 2.5 MHz).
    pub cavity_decay_rate_mhz: f64,
    /// Two-tone drive power imbalance ratio $|\Delta P| / P$ (default 0.005).
    pub two_tone_imbalance_ratio: f64,
    /// Dilution refrigerator ambient temperature in milli-Kelvin (default 15.0 mK).
    pub ambient_temp_m_k: f64,
}

impl Default for AcoustomechanicalSqueezingParams {
    fn default() -> Self {
        Self {
            membrane_freq_mhz: 15.0,
            cavity_freq_thz: 193.4,
            single_photon_coupling_hz: 250.0,
            intracavity_photons: 6.5e5,
            mechanical_q_factor: 1.2e8,
            cavity_decay_rate_mhz: 2.5,
            two_tone_imbalance_ratio: 0.005,
            ambient_temp_m_k: 15.0,
        }
    }
}

impl AcoustomechanicalSqueezingParams {
    /// Creates a new parameter configuration with bounds clamping.
    pub fn new(
        f_m_mhz: f64,
        f_c_thz: f64,
        g_0_hz: f64,
        n_c: f64,
        q_m: f64,
        kappa_mhz: f64,
        imb: f64,
        temp_mk: f64,
    ) -> Self {
        Self {
            membrane_freq_mhz: f_m_mhz.clamp(0.5, 500.0),
            cavity_freq_thz: f_c_thz.clamp(0.1, 1000.0),
            single_photon_coupling_hz: g_0_hz.clamp(1.0, 1.0e5),
            intracavity_photons: n_c.clamp(1.0e3, 1.0e9),
            mechanical_q_factor: q_m.clamp(1.0e4, 1.0e11),
            cavity_decay_rate_mhz: kappa_mhz.clamp(0.01, 100.0),
            two_tone_imbalance_ratio: imb.clamp(0.0, 0.20),
            ambient_temp_m_k: temp_mk.clamp(0.1, 1000.0),
        }
    }
}

/// Multi-physics evaluation metrics for cavity acoustomechanical squeezing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcoustomechanicalSqueezingMetrics {
    /// Ponderomotive mechanical quadrature squeezing below ZPF in dB (target >= 10.0 dB).
    pub ponderomotive_squeezing_db: f64,
    /// Quantum non-demolition (QND) measurement fidelity (target >= 0.980 or 98.0%).
    pub qnd_measurement_fidelity: f64,
    /// Mechanical thermal decoherence rate $\gamma_m$ in Hz (target <= 10.0 Hz).
    pub mechanical_decoherence_rate_hz: f64,
    /// Coherent intracavity photon occupancy $n_c$ (target >= 5.0e5).
    pub intracavity_photon_number: f64,
    /// Quantum backaction evasion purity (target >= 0.950).
    pub backaction_evasion_purity: f64,
    /// Physical compliance verification flag.
    pub is_physically_compliant: bool,
}
