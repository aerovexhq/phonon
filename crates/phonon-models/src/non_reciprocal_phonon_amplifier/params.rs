//! Physical parameter models and evaluation metrics for non-reciprocal
//! topological phonon amplification and directional quantum routing.

/// Physical parameter configuration for Floquet-engineered chiral acoustic lattices
/// and non-reciprocal parametric phonon amplifiers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonReciprocalAmplifierParams {
    /// Operating acoustic frequency in GHz (clamp 1.0 to 12.0, default 3.8).
    pub operating_frequency_ghz: f64,
    /// Parametric pump modulation frequency in MHz (clamp 10.0 to 200.0, default 45.0).
    pub pump_modulation_frequency_mhz: f64,
    /// Synthetic phase gradient per unit cell in radians (clamp 0.1 to 3.14159, default 1.5708).
    pub synthetic_phase_gradient_rad: f64,
    /// Parametric coupling / drive rate in MHz (clamp 1.0 to 50.0, default 18.5).
    pub parametric_coupling_rate_mhz: f64,
    /// Intrinsic acoustic dissipation loss rate in MHz (clamp 0.05 to 5.0, default 0.85).
    pub acoustic_loss_rate_mhz: f64,
    /// Inter-site acoustic resonator hopping rate in MHz (clamp 5.0 to 100.0, default 25.0).
    pub inter_site_hopping_mhz: f64,
    /// Traveling-wave parametric pump power in milliwatts (clamp 0.1 to 50.0, default 8.5).
    pub pump_power_mw: f64,
    /// Cryogenic operating temperature in milliKelvin (clamp 1.0 to 100.0, default 15.0).
    pub operating_temp_m_k: f64,
}

impl Default for NonReciprocalAmplifierParams {
    fn default() -> Self {
        Self {
            operating_frequency_ghz: 3.8,
            pump_modulation_frequency_mhz: 45.0,
            synthetic_phase_gradient_rad: 1.5708,
            parametric_coupling_rate_mhz: 18.5,
            acoustic_loss_rate_mhz: 0.85,
            inter_site_hopping_mhz: 25.0,
            pump_power_mw: 8.5,
            operating_temp_m_k: 15.0,
        }
    }
}

impl NonReciprocalAmplifierParams {
    /// Creates a new parameter configuration with physical clamping bounds.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        operating_frequency_ghz: f64,
        pump_modulation_frequency_mhz: f64,
        synthetic_phase_gradient_rad: f64,
        parametric_coupling_rate_mhz: f64,
        acoustic_loss_rate_mhz: f64,
        inter_site_hopping_mhz: f64,
        pump_power_mw: f64,
        operating_temp_m_k: f64,
    ) -> Self {
        Self {
            operating_frequency_ghz: operating_frequency_ghz.clamp(1.0, 12.0),
            pump_modulation_frequency_mhz: pump_modulation_frequency_mhz.clamp(10.0, 200.0),
            synthetic_phase_gradient_rad: synthetic_phase_gradient_rad.clamp(0.1, 3.14159),
            parametric_coupling_rate_mhz: parametric_coupling_rate_mhz.clamp(1.0, 50.0),
            acoustic_loss_rate_mhz: acoustic_loss_rate_mhz.clamp(0.05, 5.0),
            inter_site_hopping_mhz: inter_site_hopping_mhz.clamp(5.0, 100.0),
            pump_power_mw: pump_power_mw.clamp(0.1, 50.0),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 100.0),
        }
    }
}

/// Evaluation metrics for non-reciprocal topological phonon amplification and directional routing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonReciprocalAmplifierMetrics {
    /// Forward non-reciprocal acoustic gain in dB (target >= 20.0 dB).
    pub forward_gain_db: f64,
    /// Backward acoustic isolation in dB (target >= 30.0 dB).
    pub backward_isolation_db: f64,
    /// Added noise quanta near the Caves quantum limit (target <= 0.50 quanta).
    pub added_noise_photons: f64,
    /// Instantaneous 3-dB parametric amplification bandwidth in MHz (target >= 15.0 MHz).
    pub instantaneous_bandwidth_mhz: f64,
    /// Directional quantum routing fidelity (target >= 0.960).
    pub directional_routing_fidelity: f64,
    /// Physical compliance flag confirming all Phase 117 roadmap benchmarks are satisfied.
    pub is_physically_compliant: bool,
}
