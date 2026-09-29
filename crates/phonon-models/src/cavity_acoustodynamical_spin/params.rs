#![deny(unsafe_code)]

//! Physical parameters and multi-physics evaluation metrics for cavity quantum
//! acoustodynamical (cQAD) spin-phonon interfaces and chiral squeezed vacuum synthesizers.

/// Physical parameter configuration for cavity quantum acoustodynamical spin-phonon interfaces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CavityAcoustodynamicalSpinParams {
    /// Non-linear parametric acoustic pump power in milliwatts (clamp 0.1 to 20.0, default 4.5).
    pub pump_power_mw: f64,
    /// Acoustic cavity dissipation decay rate kappa / (2 pi) in kHz (clamp 10.0 to 500.0, default 85.0).
    pub cavity_decay_rate_khz: f64,
    /// Coherent spin-phonon coupling strength g_sp / (2 pi) in MHz (clamp 0.5 to 25.0, default 5.8).
    pub spin_phonon_coupling_mhz: f64,
    /// Non-linear parametric phononic amplifier gain in dB (clamp 5.0 to 30.0, default 16.5).
    pub non_linear_gain_db: f64,
    /// Operating cryogenic dilution bath temperature in millikelvin (clamp 5.0 to 100.0, default 20.0).
    pub cryogenic_temp_mk: f64,
    /// Phononic crystal cavity resonance frequency in GHz (clamp 1.0 to 15.0, default 5.2).
    pub acoustic_frequency_ghz: f64,
    /// Single-spin intrinsic dephasing rate gamma_phi / (2 pi) in Hz (clamp 1.0 to 50.0, default 12.0).
    pub spin_dephasing_rate_hz: f64,
    /// Chiral phonon isolation ratio in dB against backscattering (clamp 20.0 to 60.0, default 38.0).
    pub chiral_isolation_db: f64,
}

impl Default for CavityAcoustodynamicalSpinParams {
    fn default() -> Self {
        Self {
            pump_power_mw: 4.5,
            cavity_decay_rate_khz: 85.0,
            spin_phonon_coupling_mhz: 5.8,
            non_linear_gain_db: 16.5,
            cryogenic_temp_mk: 20.0,
            acoustic_frequency_ghz: 5.2,
            spin_dephasing_rate_hz: 12.0,
            chiral_isolation_db: 38.0,
        }
    }
}

impl CavityAcoustodynamicalSpinParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        pump_power_mw: f64,
        cavity_decay_rate_khz: f64,
        spin_phonon_coupling_mhz: f64,
        non_linear_gain_db: f64,
        cryogenic_temp_mk: f64,
        acoustic_frequency_ghz: f64,
        spin_dephasing_rate_hz: f64,
        chiral_isolation_db: f64,
    ) -> Self {
        Self {
            pump_power_mw: pump_power_mw.clamp(0.1, 20.0),
            cavity_decay_rate_khz: cavity_decay_rate_khz.clamp(10.0, 500.0),
            spin_phonon_coupling_mhz: spin_phonon_coupling_mhz.clamp(0.5, 25.0),
            non_linear_gain_db: non_linear_gain_db.clamp(5.0, 30.0),
            cryogenic_temp_mk: cryogenic_temp_mk.clamp(5.0, 100.0),
            acoustic_frequency_ghz: acoustic_frequency_ghz.clamp(1.0, 15.0),
            spin_dephasing_rate_hz: spin_dephasing_rate_hz.clamp(1.0, 50.0),
            chiral_isolation_db: chiral_isolation_db.clamp(20.0, 60.0),
        }
    }
}

/// Multi-physics performance evaluation metrics for cavity quantum
/// acoustodynamical spin-phonon interfaces and chiral squeezed vacuum synthesizers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CavityAcoustodynamicalSpinMetrics {
    /// Acoustic quadrature squeezing below vacuum level in dB (target >= 12.0).
    pub acoustic_quadrature_squeezing_db: f64,
    /// Spin-phonon coherent quantum state transfer fidelity (target >= 0.9970).
    pub spin_phonon_fidelity: f64,
    /// Spin defect coherence lifetime T2 in milliseconds (target >= 50.0).
    pub spin_coherence_lifetime_ms: f64,
    /// Thermal equilibrium phonon occupancy n_th in quanta (target <= 0.05).
    pub thermal_phonon_occupancy: f64,
    /// Spin-cavity acoustic Purcell enhancement factor F_P (target >= 25.0).
    pub purcell_enhancement_factor: f64,
    /// Overall physical compliance flag across all roadmap targets.
    pub is_physically_compliant: bool,
}
