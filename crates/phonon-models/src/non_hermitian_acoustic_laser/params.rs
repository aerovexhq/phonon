//! Parameter models and metric structures for non-Hermitian Floquet topological
//! acoustic lasers, skin-effect metamaterials, and non-reciprocal acoustics.

/// Physical parameter configuration for non-Hermitian Floquet acoustic lasers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonHermitianLaserParams {
    /// Acoustic superlattice linear dimension $N$ (grid $N \times N$, default 20).
    pub lattice_size_n: usize,
    /// Center acoustic operating frequency $f_0$ in MHz (default 450.0 MHz).
    pub center_freq_mhz: f64,
    /// Forward non-reciprocal acoustic hopping rate $t_R$ in arb. units (default 2.20).
    pub forward_hopping_rate: f64,
    /// Backward non-reciprocal acoustic hopping rate $t_L$ in arb. units (default 0.065).
    pub backward_hopping_rate: f64,
    /// Non-Hermitian gain-loss onsite contrast $\gamma$ (default 0.85).
    pub gain_loss_contrast: f64,
    /// Spatiotemporal Floquet modulation frequency $\Omega_{\text{pump}}$ in MHz (default 45.0 MHz).
    pub floquet_pump_freq_mhz: f64,
    /// Acoustic pump drive power $P_{\text{pump}}$ in milliwatts (default 32.0 mW).
    pub pump_power_mw: f64,
    /// Cavity acoustic quality factor $Q_{\text{cav}}$ (default 6.5e3).
    pub cavity_q_factor: f64,
    /// Ambient operating temperature in Kelvin (default 295.0 K).
    pub ambient_temp_k: f64,
}

impl Default for NonHermitianLaserParams {
    fn default() -> Self {
        Self {
            lattice_size_n: 20,
            center_freq_mhz: 450.0,
            forward_hopping_rate: 2.20,
            backward_hopping_rate: 0.065,
            gain_loss_contrast: 0.85,
            floquet_pump_freq_mhz: 45.0,
            pump_power_mw: 32.0,
            cavity_q_factor: 6.5e3,
            ambient_temp_k: 295.0,
        }
    }
}

impl NonHermitianLaserParams {
    /// Creates a new parameter configuration with bounds clamping.
    pub fn new(
        n: usize,
        f_0_mhz: f64,
        t_r: f64,
        t_l: f64,
        gamma: f64,
        omega_p_mhz: f64,
        p_mw: f64,
        q_cav: f64,
        temp_k: f64,
    ) -> Self {
        Self {
            lattice_size_n: n.clamp(6, 120),
            center_freq_mhz: f_0_mhz.clamp(1.0, 5000.0),
            forward_hopping_rate: t_r.clamp(0.1, 10.0),
            backward_hopping_rate: t_l.clamp(0.001, 5.0),
            gain_loss_contrast: gamma.clamp(0.01, 10.0),
            floquet_pump_freq_mhz: omega_p_mhz.clamp(1.0, 1000.0),
            pump_power_mw: p_mw.clamp(1.0, 250.0),
            cavity_q_factor: q_cav.clamp(50.0, 1.0e7),
            ambient_temp_k: temp_k.clamp(0.1, 400.0),
        }
    }
}

/// Multi-physics evaluation metrics for non-Hermitian topological acoustic lasers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonHermitianLaserMetrics {
    /// Non-Hermitian skin mode boundary localization ratio $\eta_{\text{skin}}$ (target >= 92.0%).
    pub skin_mode_localization_ratio: f64,
    /// Laser single-mode suppression ratio SMSR in dB (target >= 35.0 dB).
    pub laser_smsr_db: f64,
    /// Dynamic corner acoustic laser output power $P_{\text{out}}$ in mW (target >= 15.0 mW).
    pub laser_output_power_mw: f64,
    /// Non-reciprocal acoustic forward/backward isolation in dB (target >= 30.0 dB).
    pub non_reciprocal_isolation_db: f64,
    /// Higher-order topological corner mode spatial purity / fidelity (target >= 0.950).
    pub corner_mode_fidelity: f64,
    /// Physical compliance verification flag.
    pub is_physically_compliant: bool,
}
