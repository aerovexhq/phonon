//! Parameter models and metric structures for quantum acoustoelectric
//! Josephson vortex ratchets, topological soliton transport, and Shapiro locking.

/// Physical parameter configuration for acoustoelectric Josephson vortex ratchets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JosephsonVortexRatchetParams {
    /// Long Josephson junction length $L$ in micrometers (default 250.0 um).
    pub junction_length_um: f64,
    /// Josephson penetration depth $\lambda_J$ in micrometers (default 22.0 um).
    pub penetration_depth_um: f64,
    /// Swihart characteristic velocity $c_{\text{sw}}$ in m/s (default 1.2 x 10^7 m/s).
    pub swihart_velocity_m_per_s: f64,
    /// Surface acoustic wave (SAW) drive frequency $f_{\text{saw}}$ in GHz (default 2.45 GHz).
    pub saw_frequency_ghz: f64,
    /// Ratchet potential spatial asymmetry parameter $\epsilon$ (default 0.36).
    pub ratchet_asymmetry: f64,
    /// Incident acoustic drive power $P_{\text{ac}}$ in microwatts (default 0.35 uW).
    pub acoustic_power_uw: f64,
    /// McCumber-Stewart damping parameter $\alpha$ (default 0.025).
    pub damping_alpha: f64,
    /// Critical current density $J_c$ in A/cm^2 (default 1.2e3 A/cm^2).
    pub critical_current_density_a_cm2: f64,
    /// Cryogenic operating temperature in Kelvin (default 0.035 K).
    pub operating_temp_k: f64,
}

impl Default for JosephsonVortexRatchetParams {
    fn default() -> Self {
        Self {
            junction_length_um: 250.0,
            penetration_depth_um: 22.0,
            swihart_velocity_m_per_s: 1.2e7,
            saw_frequency_ghz: 2.45,
            ratchet_asymmetry: 0.36,
            acoustic_power_uw: 0.35,
            damping_alpha: 0.025,
            critical_current_density_a_cm2: 1.2e3,
            operating_temp_k: 0.035,
        }
    }
}

impl JosephsonVortexRatchetParams {
    /// Creates a new parameter configuration with bounds clamping.
    pub fn new(
        l_um: f64,
        lambda_j_um: f64,
        c_sw: f64,
        f_saw_ghz: f64,
        eps: f64,
        p_ac_uw: f64,
        alpha: f64,
        j_c: f64,
        temp_k: f64,
    ) -> Self {
        Self {
            junction_length_um: l_um.clamp(10.0, 5000.0),
            penetration_depth_um: lambda_j_um.clamp(1.0, 200.0),
            swihart_velocity_m_per_s: c_sw.clamp(1.0e5, 5.0e7),
            saw_frequency_ghz: f_saw_ghz.clamp(0.1, 20.0),
            ratchet_asymmetry: eps.clamp(0.01, 1.0),
            acoustic_power_uw: p_ac_uw.clamp(0.01, 50.0),
            damping_alpha: alpha.clamp(0.001, 1.0),
            critical_current_density_a_cm2: j_c.clamp(10.0, 1.0e6),
            operating_temp_k: temp_k.clamp(0.001, 10.0),
        }
    }
}

/// Multi-physics evaluation metrics for Josephson vortex ratchets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JosephsonVortexRatchetMetrics {
    /// Directional fluxon ratchet rectification efficiency (target >= 0.920 or 92.0%).
    pub ratchet_rectification_efficiency: f64,
    /// Normalized single-fluxon transport velocity $v / \bar{c}$ (target >= 0.850).
    pub normalized_soliton_velocity: f64,
    /// Acoustic depinning power threshold $P_{\text{ac,th}}$ in uW (target <= 0.50 uW).
    pub acoustic_threshold_power_uw: f64,
    /// Fractional phase-slip Shapiro locking precision $\Delta f / f_{\text{saw}}$ (target <= 1.0e-9).
    pub phase_slip_locking_precision: f64,
    /// Low-frequency voltage noise spectral density in V^2/Hz (target <= 1.0e-22).
    pub voltage_noise_spectral_density_v2_hz: f64,
    /// Physical compliance verification flag.
    pub is_physically_compliant: bool,
}
