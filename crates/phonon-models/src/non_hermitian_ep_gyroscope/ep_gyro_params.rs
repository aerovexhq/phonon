//! Multi-physics parameter structures and metric definitions for
//! non-Hermitian phononic exceptional point gyroscopes and Sagnac enhancers.

/// Physical parameter configuration for non-Hermitian phononic exceptional point gyroscopes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EpGyroscopeParams {
    /// Acoustic ring cavity radius $R$ in micrometers (default 200.0 um).
    pub ring_radius_um: f64,
    /// Acoustic center operating frequency $f_0$ in MHz (default 500.0 MHz).
    pub acoustic_center_freq_mhz: f64,
    /// Bulk acoustic wave velocity $v_{\\text{ac}}$ in m/s (default 3800.0 m/s).
    pub base_acoustic_velocity_m_s: f64,
    /// Inter-mode coupling rate $\\kappa / (2\\pi)$ in MHz (default 2.0 MHz).
    pub coupling_rate_mhz: f64,
    /// Balanced gain and loss rate $\\gamma / (2\\pi)$ in MHz (default 1.98 MHz).
    pub gain_loss_rate_mhz: f64,
    /// Physical rotation rate $\\Omega_{\\text{rot}}$ in deg/s (default 10.0 deg/s).
    pub rotation_rate_deg_s: f64,
    /// Ambient thermal noise temperature $T$ in Kelvin (default 300.0 K).
    pub thermal_noise_temp_k: f64,
    /// Intrinsic acoustic quality factor $Q_{\\text{ac}}$ (default 25000.0).
    pub quality_factor: f64,
}

impl Default for EpGyroscopeParams {
    fn default() -> Self {
        Self {
            ring_radius_um: 200.0,
            acoustic_center_freq_mhz: 500.0,
            base_acoustic_velocity_m_s: 3800.0,
            coupling_rate_mhz: 2.0,
            gain_loss_rate_mhz: 1.98,
            rotation_rate_deg_s: 10.0,
            thermal_noise_temp_k: 300.0,
            quality_factor: 25000.0,
        }
    }
}

impl EpGyroscopeParams {
    /// Creates a new parameter configuration for non-Hermitian EP gyroscopes.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        radius_um: f64,
        f0_mhz: f64,
        v_ac: f64,
        kappa_mhz: f64,
        gamma_mhz: f64,
        rot_deg_s: f64,
        temp_k: f64,
        q_factor: f64,
    ) -> Self {
        Self {
            ring_radius_um: radius_um.clamp(10.0, 5000.0),
            acoustic_center_freq_mhz: f0_mhz.clamp(10.0, 5000.0),
            base_acoustic_velocity_m_s: v_ac.clamp(1000.0, 12000.0),
            coupling_rate_mhz: kappa_mhz.clamp(0.01, 100.0),
            gain_loss_rate_mhz: gamma_mhz.clamp(0.01, 100.0),
            rotation_rate_deg_s: rot_deg_s.clamp(1e-6, 10000.0),
            thermal_noise_temp_k: temp_k.clamp(0.01, 500.0),
            quality_factor: q_factor.clamp(100.0, 1e7),
        }
    }
}

/// Multi-physics evaluation metrics for non-Hermitian phononic EP gyroscopes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EpGyroscopeMetrics {
    /// Sagnac scale-factor enhancement $\\eta_{\\text{enhancement}}$ over standard reciprocal limit (target >= 15.0x).
    pub scale_factor_enhancement: f64,
    /// Sensor dynamic range in dB (target >= 120.0 dB).
    pub dynamic_range_db: f64,
    /// Angle random walk (ARW) in deg/sqrt(hr) (target <= 0.001 deg/sqrt(hr)).
    pub angle_random_walk_deg_sqrthr: f64,
    /// Bias stability in deg/hr (target <= 0.005 deg/hr).
    pub bias_stability_deg_hr: f64,
    /// Petermann excess noise factor $K$ accounting for mode non-orthogonality.
    pub petermann_factor: f64,
    /// EP-induced frequency splitting in kHz.
    pub frequency_splitting_khz: f64,
    /// Physical compliance flag verifying all engineering criteria are satisfied.
    pub is_physically_compliant: bool,
}
