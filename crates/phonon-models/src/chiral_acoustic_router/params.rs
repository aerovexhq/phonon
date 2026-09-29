#![deny(unsafe_code)]

//! Physical parameters and metrics configuration for chiral quantum acoustic
//! metamaterial circulators and multi-terminal non-reciprocal router networks.

/// Physical parameter configuration for chiral quantum acoustic metamaterial routers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralAcousticRouterParams {
    /// Center acoustic operating frequency in GHz (clamp 1.0 to 15.0, default 5.0).
    pub center_frequency_ghz: f64,
    /// Synthetic angular momentum bias modulation rate in MHz (clamp 10.0 to 200.0, default 80.0).
    pub synthetic_angular_momentum_mhz: f64,
    /// Number of circulating waveguide terminals / ports (clamp 3 to 8, default 4).
    pub ports_count: usize,
    /// Hydrodynamic non-zero odd viscosity coefficient (clamp 0.01 to 0.50, default 0.15).
    pub odd_viscosity_coefficient: f64,
    /// Acoustic ring resonator unloaded quality factor Q (clamp 1.0e5 to 1.0e8, default 2.0e7).
    pub resonator_q_factor: f64,
    /// Input/output bus waveguide coupling rate in MHz (clamp 5.0 to 50.0, default 25.0).
    pub waveguide_coupling_rate_mhz: f64,
    /// Cryogenic operating temperature in milli-Kelvin (clamp 1.0 to 50.0, default 15.0).
    pub operating_temp_m_k: f64,
    /// Spatial metamaterial fabrication disorder fraction (clamp 0.0 to 0.10, default 0.02).
    pub fabrication_disorder_fraction: f64,
}

impl Default for ChiralAcousticRouterParams {
    fn default() -> Self {
        Self {
            center_frequency_ghz: 5.0,
            synthetic_angular_momentum_mhz: 80.0,
            ports_count: 4,
            odd_viscosity_coefficient: 0.15,
            resonator_q_factor: 2.0e7,
            waveguide_coupling_rate_mhz: 25.0,
            operating_temp_m_k: 15.0,
            fabrication_disorder_fraction: 0.02,
        }
    }
}

impl ChiralAcousticRouterParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        center_frequency_ghz: f64,
        synthetic_angular_momentum_mhz: f64,
        ports_count: usize,
        odd_viscosity_coefficient: f64,
        resonator_q_factor: f64,
        waveguide_coupling_rate_mhz: f64,
        operating_temp_m_k: f64,
        fabrication_disorder_fraction: f64,
    ) -> Self {
        Self {
            center_frequency_ghz: center_frequency_ghz.clamp(1.0, 15.0),
            synthetic_angular_momentum_mhz: synthetic_angular_momentum_mhz.clamp(10.0, 200.0),
            ports_count: ports_count.clamp(3, 8),
            odd_viscosity_coefficient: odd_viscosity_coefficient.clamp(0.01, 0.50),
            resonator_q_factor: resonator_q_factor.clamp(1.0e5, 1.0e8),
            waveguide_coupling_rate_mhz: waveguide_coupling_rate_mhz.clamp(5.0, 50.0),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 50.0),
            fabrication_disorder_fraction: fabrication_disorder_fraction.clamp(0.0, 0.10),
        }
    }
}

/// Multi-physics evaluation metrics for chiral quantum acoustic metamaterial routers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralAcousticRouterMetrics {
    /// Non-reciprocal backward port isolation in dB (target >= 35.0).
    pub non_reciprocal_isolation_db: f64,
    /// Forward waveguide bus insertion loss in dB (target <= 0.40).
    pub insertion_loss_db: f64,
    /// Multi-terminal quantum phase coherence fidelity across ports (target >= 0.9920).
    pub phase_coherence_fidelity: f64,
    /// Inter-port cross-talk rejection in dB (target >= 30.0).
    pub cross_talk_rejection_db: f64,
    /// Operating circulation 3-dB bandwidth in MHz (target >= 12.0).
    pub operating_bandwidth_mhz: f64,
    /// Overall physical compliance flag across all roadmap performance targets.
    pub is_physically_compliant: bool,
}
