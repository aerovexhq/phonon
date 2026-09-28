//! Parametric processes (3WM and 4WM) in traveling-wave amplifiers:
//! gain formulas, bandwidths, and quantum-limited noise performance.

/// Planck constant over 2*pi in J*s.
pub const HBAR: f64 = 1.054_571_817e-34;
/// Boltzmann constant in J/K.
pub const BOLTZMANN_K: f64 = 1.380_649e-23;

/// Parameters defining a parametric amplification process.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParametricProcessParams {
    /// Signal center frequency f_s in Hz.
    pub signal_freq_hz: f64,
    /// Pump frequency f_p in Hz (f_p = f_s + f_i for 3WM, 2 f_p = f_s + f_i for 4WM).
    pub pump_freq_hz: f64,
    /// Parametric gain factor g_0 in rad / meter (or rad / cell).
    pub gain_factor_per_meter: f64,
    /// Linear phase mismatch Delta k in rad / meter (or rad / cell).
    pub phase_mismatch_rad_per_m: f64,
    /// Total interaction length L in meters (or number of cells).
    pub total_length_m: f64,
    /// Physical bath temperature T_bath in Kelvin (typically 0.02 K = 20 mK).
    pub bath_temperature_k: f64,
}

impl ParametricProcessParams {
    /// Constructs parametric process parameters.
    pub fn new(
        signal_freq_hz: f64,
        pump_freq_hz: f64,
        gain_factor_per_meter: f64,
        phase_mismatch_rad_per_m: f64,
        total_length_m: f64,
        bath_temperature_k: f64,
    ) -> Self {
        Self {
            signal_freq_hz,
            pump_freq_hz,
            gain_factor_per_meter,
            phase_mismatch_rad_per_m,
            total_length_m,
            bath_temperature_k,
        }
    }

    /// Standard 3WM JTWPA configuration achieving ~20 dB gain at 6 GHz.
    pub fn standard_3wm_amplifier() -> Self {
        Self::new(6.0e9, 12.0e9, 60.0, 2.0, 0.05, 0.02)
    }

    /// Effective spatial exponential growth rate kappa_eff = sqrt(g^2 - (Delta k / 2)^2):
    pub fn effective_growth_rate(&self) -> f64 {
        let g_sq = self.gain_factor_per_meter.powi(2);
        let half_dk_sq = (0.5 * self.phase_mismatch_rad_per_m).powi(2);
        (g_sq - half_dk_sq).max(0.0).sqrt()
    }

    /// Power gain G_s(L) = 1 + (g / kappa)^2 * sinh^2(kappa * L) (linear power ratio):
    pub fn signal_power_gain_linear(&self) -> f64 {
        let kappa = self.effective_growth_rate();
        let g = self.gain_factor_per_meter;
        let l = self.total_length_m;

        if kappa < 1e-6 {
            1.0 + (g * l).powi(2)
        } else {
            1.0 + (g / kappa).powi(2) * (kappa * l).sinh().powi(2)
        }
    }

    /// Signal power gain in decibels G_dB = 10 * log10(G_s):
    pub fn signal_power_gain_db(&self) -> f64 {
        10.0 * self.signal_power_gain_linear().max(1.0).log10()
    }

    /// Caves quantum limit for added noise quanta N_add:
    /// $$N_{add} = \frac{1}{2} \left( 1 - \frac{1}{G} \right)$$
    pub fn added_noise_quanta(&self) -> f64 {
        let g = self.signal_power_gain_linear().max(1.0);
        0.5 * (1.0 - 1.0 / g)
    }

    /// Quantum noise temperature T_add = N_add * (hbar * omega_s / k_B) in Kelvin:
    pub fn added_noise_temperature_k(&self) -> f64 {
        let omega = 2.0 * std::f64::consts::PI * self.signal_freq_hz;
        let n_add = self.added_noise_quanta();
        (n_add * HBAR * omega) / BOLTZMANN_K
    }
}
