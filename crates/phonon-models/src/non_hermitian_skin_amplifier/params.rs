#![deny(unsafe_code)]

//! Physical parameters and multi-physics evaluation metrics for non-Hermitian
//! skin-topological phonon diodes and unidirectional quantum acoustic amplifiers.

/// Physical parameter configuration for non-Hermitian skin-topological phonon amplifiers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonHermitianSkinAmplifierParams {
    /// Acoustic center frequency in GHz (clamp 1.0 to 12.0, default 4.2).
    pub center_frequency_ghz: f64,
    /// Number of discrete acoustic resonator lattice sites (clamp 10 to 100, default 30).
    pub lattice_sites_count: usize,
    /// Forward directional acoustic hopping rate in MHz (clamp 10.0 to 100.0, default 45.0).
    pub forward_coupling_mhz: f64,
    /// Reverse directional acoustic hopping rate in MHz (clamp 0.5 to 20.0, default 5.0).
    pub reverse_coupling_mhz: f64,
    /// Parametric mechanical pump rate in MHz (clamp 5.0 to 50.0, default 20.0).
    pub parametric_pump_rate_mhz: f64,
    /// Non-Hermitian dissipation gradient rate across the lattice in MHz (clamp 1.0 to 30.0, default 12.0).
    pub dissipation_gradient_mhz: f64,
    /// Cryogenic bath operating temperature in milli-Kelvin (clamp 1.0 to 50.0, default 15.0).
    pub operating_temp_m_k: f64,
    /// Input microwave acoustic signal power in dBm (clamp -60.0 to -10.0, default -35.0).
    pub input_signal_power_dbm: f64,
}

impl Default for NonHermitianSkinAmplifierParams {
    fn default() -> Self {
        Self {
            center_frequency_ghz: 4.2,
            lattice_sites_count: 30,
            forward_coupling_mhz: 45.0,
            reverse_coupling_mhz: 5.0,
            parametric_pump_rate_mhz: 20.0,
            dissipation_gradient_mhz: 12.0,
            operating_temp_m_k: 15.0,
            input_signal_power_dbm: -35.0,
        }
    }
}

impl NonHermitianSkinAmplifierParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        center_frequency_ghz: f64,
        lattice_sites_count: usize,
        forward_coupling_mhz: f64,
        reverse_coupling_mhz: f64,
        parametric_pump_rate_mhz: f64,
        dissipation_gradient_mhz: f64,
        operating_temp_m_k: f64,
        input_signal_power_dbm: f64,
    ) -> Self {
        Self {
            center_frequency_ghz: center_frequency_ghz.clamp(1.0, 12.0),
            lattice_sites_count: lattice_sites_count.clamp(10, 100),
            forward_coupling_mhz: forward_coupling_mhz.clamp(10.0, 100.0),
            reverse_coupling_mhz: reverse_coupling_mhz.clamp(0.5, 20.0),
            parametric_pump_rate_mhz: parametric_pump_rate_mhz.clamp(5.0, 50.0),
            dissipation_gradient_mhz: dissipation_gradient_mhz.clamp(1.0, 30.0),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 50.0),
            input_signal_power_dbm: input_signal_power_dbm.clamp(-60.0, -10.0),
        }
    }
}

/// Multi-physics performance evaluation metrics for non-Hermitian skin-topological phonon amplifiers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonHermitianSkinAmplifierMetrics {
    /// Forward directional acoustic power gain in dB (target >= 28.0).
    pub forward_gain_db: f64,
    /// Reverse non-reciprocal transmission isolation in dB (target >= 42.0).
    pub reverse_isolation_db: f64,
    /// Quantum-limited added noise figure in quanta (target <= 0.250).
    pub added_noise_quanta: f64,
    /// Dynamic 1-dB compression power saturation threshold in dBm (target >= -15.0).
    pub power_saturation_threshold_dbm: f64,
    /// Non-Hermitian skin mode boundary localization ratio (target >= 0.900).
    pub skin_mode_localization_ratio: f64,
    /// Overall physical compliance flag across all roadmap performance targets.
    pub is_physically_compliant: bool,
}
