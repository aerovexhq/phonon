//! RF injection locking, phase noise linewidth, and Adler synchronization
//! for coupled STNO / SHNO networks.

/// Parameters governing injection locking of a spin-torque oscillator to an external RF microwave signal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InjectionLockingParams {
    /// Free-running oscillator natural frequency f_0 in Hz.
    pub natural_frequency_hz: f64,
    /// External injection RF signal frequency f_inj in Hz.
    pub injection_frequency_hz: f64,
    /// Injected microwave RF power in Watts.
    pub injection_power_watts: f64,
    /// Free-running auto-oscillator power in Watts.
    pub oscillator_power_watts: f64,
    /// Effective loaded quality factor Q of the oscillator cavity/resonance.
    pub quality_factor: f64,
}

impl InjectionLockingParams {
    /// Constructs injection locking parameters.
    pub fn new(
        natural_frequency_hz: f64,
        injection_frequency_hz: f64,
        injection_power_watts: f64,
        oscillator_power_watts: f64,
        quality_factor: f64,
    ) -> Self {
        Self {
            natural_frequency_hz,
            injection_frequency_hz,
            injection_power_watts,
            oscillator_power_watts,
            quality_factor,
        }
    }

    /// Adler injection locking bandwidth Delta omega_lock in rad/s:
    /// $$\Delta \omega_{lock} = \frac{\omega_0}{2 Q} \sqrt{\frac{P_{inj}}{P_{osc}}}$$
    pub fn locking_range_rad_per_s(&self) -> f64 {
        let omega0 = 2.0 * std::f64::consts::PI * self.natural_frequency_hz;
        let p_ratio =
            (self.injection_power_watts / self.oscillator_power_watts.max(1e-12)).max(0.0);
        (omega0 / (2.0 * self.quality_factor.max(1.0))) * p_ratio.sqrt()
    }

    /// Adler locking frequency bandwidth Delta f_lock in Hz:
    pub fn locking_bandwidth_hz(&self) -> f64 {
        self.locking_range_rad_per_s() / (2.0 * std::f64::consts::PI)
    }

    /// Checks if the oscillator frequency detuning is within the Adler synchronization band:
    /// $$|f_0 - f_{inj}| \le \Delta f_{lock}$$
    pub fn is_locked(&self) -> bool {
        let detuning_hz = (self.natural_frequency_hz - self.injection_frequency_hz).abs();
        detuning_hz <= self.locking_bandwidth_hz()
    }

    /// Steady-state locked phase offset phi_lock - phi_inj = arcsin(Delta omega / Delta omega_lock):
    pub fn steady_state_phase_offset_rad(&self) -> Option<f64> {
        let detuning_rad =
            2.0 * std::f64::consts::PI * (self.natural_frequency_hz - self.injection_frequency_hz);
        let delta_lock = self.locking_range_rad_per_s();
        if delta_lock <= 0.0 {
            return None;
        }
        let ratio = (detuning_rad / delta_lock).clamp(-1.0, 1.0);
        Some(ratio.asin())
    }
}
