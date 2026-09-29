#![deny(unsafe_code)]

//! Physical parameters and multi-physics evaluation metrics for non-Hermitian
//! higher-order topological phononic lasers and chiral quadrupole acoustical frequency synthesizers.

/// Physical parameter configuration for non-Hermitian higher-order quadrupole lasers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonHermitianQuadrupoleLaserParams {
    /// Active phononic pump gain rate in kHz (clamp 10.0 to 500.0, default 120.0).
    pub pump_gain_rate_khz: f64,
    /// Intrinsic and radiative loss dissipation rate in kHz (clamp 10.0 to 500.0, default 110.0).
    pub loss_dissipation_rate_khz: f64,
    /// Topological quadrupole coupling strength in MHz (clamp 5.0 to 60.0, default 28.0).
    pub quadrupole_coupling_mhz: f64,
    /// Higher-order topological corner confinement factor (clamp 0.50 to 0.99, default 0.88).
    pub corner_confinement_factor: f64,
    /// Center acoustic resonator operating frequency in GHz (clamp 1.0 to 12.0, default 4.8).
    pub acoustic_resonator_frequency_ghz: f64,
    /// Cryogenic operating temperature in millikelvin (clamp 1.0 to 50.0, default 15.0).
    pub cryogenic_temp_mk: f64,
    /// Non-linear phononic gain saturation parameter (clamp 0.001 to 0.05, default 0.012).
    pub non_linear_saturation_parameter: f64,
    /// 2D synthetic quadrupole lattice dimension N (clamp 4 to 32, default 12).
    pub lattice_dimension: usize,
}

impl Default for NonHermitianQuadrupoleLaserParams {
    fn default() -> Self {
        Self {
            pump_gain_rate_khz: 120.0,
            loss_dissipation_rate_khz: 110.0,
            quadrupole_coupling_mhz: 28.0,
            corner_confinement_factor: 0.88,
            acoustic_resonator_frequency_ghz: 4.8,
            cryogenic_temp_mk: 15.0,
            non_linear_saturation_parameter: 0.012,
            lattice_dimension: 12,
        }
    }
}

impl NonHermitianQuadrupoleLaserParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        pump_gain_rate_khz: f64,
        loss_dissipation_rate_khz: f64,
        quadrupole_coupling_mhz: f64,
        corner_confinement_factor: f64,
        acoustic_resonator_frequency_ghz: f64,
        cryogenic_temp_mk: f64,
        non_linear_saturation_parameter: f64,
        lattice_dimension: usize,
    ) -> Self {
        Self {
            pump_gain_rate_khz: pump_gain_rate_khz.clamp(10.0, 500.0),
            loss_dissipation_rate_khz: loss_dissipation_rate_khz.clamp(10.0, 500.0),
            quadrupole_coupling_mhz: quadrupole_coupling_mhz.clamp(5.0, 60.0),
            corner_confinement_factor: corner_confinement_factor.clamp(0.50, 0.99),
            acoustic_resonator_frequency_ghz: acoustic_resonator_frequency_ghz.clamp(1.0, 12.0),
            cryogenic_temp_mk: cryogenic_temp_mk.clamp(1.0, 50.0),
            non_linear_saturation_parameter: non_linear_saturation_parameter.clamp(0.001, 0.05),
            lattice_dimension: lattice_dimension.clamp(4, 32),
        }
    }
}

/// Multi-physics performance evaluation metrics for non-Hermitian quadrupole phononic lasers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonHermitianQuadrupoleLaserMetrics {
    /// Single-mode topological corner mode lasing fidelity (target >= 0.9970).
    pub corner_mode_lasing_fidelity: f64,
    /// Fractional frequency instability Allan deviation floor (target <= 1.5e-12).
    pub fractional_frequency_instability: f64,
    /// Side-mode suppression ratio against bulk and edge modes in dB (target >= 45.0).
    pub side_mode_suppression_ratio_db: f64,
    /// Topological corner mode coherence lifetime in milliseconds (target >= 80.0).
    pub topological_corner_mode_lifetime_ms: f64,
    /// Parity-time (PT) symmetry confinement ratio in the quadrupole sector (target >= 0.920).
    pub pt_symmetry_confinement_ratio: f64,
    /// Overall physical compliance flag across all roadmap targets.
    pub is_physically_compliant: bool,
}
