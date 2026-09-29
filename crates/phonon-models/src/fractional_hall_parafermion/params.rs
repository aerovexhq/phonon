#![deny(unsafe_code)]

//! Physical parameters and multi-physics evaluation metrics for fractional quantum Hall
//! acoustic metamaterials and non-Abelian parafermion interferometers.

/// Physical parameter configuration for fractional quantum Hall acoustic metamaterials.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FractionalHallParafermionParams {
    /// Acoustic resonance frequency in GHz (clamp 1.0 to 12.0, default 4.2).
    pub acoustic_resonance_freq_ghz: f64,
    /// Synthetic Lorentz coupling rate in MHz (clamp 10.0 to 120.0, default 65.0).
    pub synthetic_lorentz_coupling_mhz: f64,
    /// Fractional filling factor nu (clamp 0.20 to 0.80, default 0.3333333333333333).
    pub fractional_filling_factor: f64,
    /// Parafermion order Z_m (clamp 3 to 6, default 3).
    pub parafermion_order_z_m: usize,
    /// Interferometer arm length in micrometers (clamp 10.0 to 120.0, default 45.0).
    pub interferometer_arm_length_um: f64,
    /// Acoustic damping rate in kHz (clamp 0.2 to 20.0, default 1.8).
    pub acoustic_damping_rate_khz: f64,
    /// Operating temperature in millikelvin (clamp 1.0 to 50.0, default 12.0).
    pub operating_temp_m_k: f64,
    /// Quasiparticle tunneling rate in MHz (clamp 5.0 to 50.0, default 20.0).
    pub quasiparticle_tunneling_mhz: f64,
}

impl Default for FractionalHallParafermionParams {
    fn default() -> Self {
        Self {
            acoustic_resonance_freq_ghz: 4.2,
            synthetic_lorentz_coupling_mhz: 65.0,
            fractional_filling_factor: 1.0 / 3.0,
            parafermion_order_z_m: 3,
            interferometer_arm_length_um: 45.0,
            acoustic_damping_rate_khz: 1.8,
            operating_temp_m_k: 12.0,
            quasiparticle_tunneling_mhz: 20.0,
        }
    }
}

impl FractionalHallParafermionParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        acoustic_resonance_freq_ghz: f64,
        synthetic_lorentz_coupling_mhz: f64,
        fractional_filling_factor: f64,
        parafermion_order_z_m: usize,
        interferometer_arm_length_um: f64,
        acoustic_damping_rate_khz: f64,
        operating_temp_m_k: f64,
        quasiparticle_tunneling_mhz: f64,
    ) -> Self {
        Self {
            acoustic_resonance_freq_ghz: acoustic_resonance_freq_ghz.clamp(1.0, 12.0),
            synthetic_lorentz_coupling_mhz: synthetic_lorentz_coupling_mhz.clamp(10.0, 120.0),
            fractional_filling_factor: fractional_filling_factor.clamp(0.20, 0.80),
            parafermion_order_z_m: parafermion_order_z_m.clamp(3, 6),
            interferometer_arm_length_um: interferometer_arm_length_um.clamp(10.0, 120.0),
            acoustic_damping_rate_khz: acoustic_damping_rate_khz.clamp(0.2, 20.0),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 50.0),
            quasiparticle_tunneling_mhz: quasiparticle_tunneling_mhz.clamp(5.0, 50.0),
        }
    }
}

/// Multi-physics performance evaluation metrics for fractional quantum Hall
/// acoustic metamaterials and non-Abelian parafermion interferometers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FractionalHallParafermionMetrics {
    /// Fractional braid phase fidelity (target >= 0.9970).
    pub braid_phase_fidelity: f64,
    /// Fractional quasiparticle state fidelity (target >= 0.9950).
    pub fractional_state_fidelity: f64,
    /// Fractional quantization error (target <= 0.0050).
    pub fractional_quantization_error: f64,
    /// Many-body topological fractional gap in MHz (target >= 15.0).
    pub topological_fractional_gap_mhz: f64,
    /// Non-Abelian braiding interference visibility (target >= 0.9600).
    pub braiding_visibility: f64,
    /// Overall physical compliance flag across all roadmap targets.
    pub is_physically_compliant: bool,
}
