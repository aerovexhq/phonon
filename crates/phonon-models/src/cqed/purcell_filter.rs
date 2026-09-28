//! Purcell effect filters: bandpass filtering suppressing spontaneous radiative qubit decay
//! while preserving fast dispersive readout cavity coupling.

use crate::cqed::DispersiveCqedSystem;

/// Lumped/distributed bandpass Purcell filter inserted between readout cavity and feedline.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PurcellFilter {
    /// Filter passband center frequency $f_P = \omega_P / (2\pi)$ in GHz (aligned to cavity $\omega_r$).
    pub filter_frequency_ghz: f64,
    /// 3-dB passband bandwidth $B_P / (2\pi)$ in MHz.
    pub bandwidth_mhz: f64,
    /// Characteristic impedance in Ohms (typically 50 Ohms).
    pub impedance_ohms: f64,
    /// Filter order / number of reactive poles $N$ (typically 2).
    pub filter_order: usize,
}

impl PurcellFilter {
    /// Constructs a new Purcell filter.
    pub fn new(
        filter_frequency_ghz: f64,
        bandwidth_mhz: f64,
        impedance_ohms: f64,
        filter_order: usize,
    ) -> Self {
        Self {
            filter_frequency_ghz,
            bandwidth_mhz,
            impedance_ohms,
            filter_order: filter_order.max(1),
        }
    }

    /// Standard 2nd-order Butterworth Purcell filter centered at 7.0 GHz with 40 MHz bandwidth.
    pub fn standard_7ghz() -> Self {
        Self::new(7.0, 40.0, 50.0, 2)
    }

    /// Filter suppression factor $S(f) \ge 1.0$ at probe frequency $f$ in GHz:
    /// $$S(f) = 1 + \left( \frac{2 |f - f_P|}{B_P} \right)^{2N}$$
    pub fn suppression_ratio_at_freq(&self, freq_ghz: f64) -> f64 {
        let delta_f_mhz = (freq_ghz - self.filter_frequency_ghz).abs() * 1.0e3;
        let x = (2.0 * delta_f_mhz) / self.bandwidth_mhz.max(1e-3);
        1.0 + x.powi((2 * self.filter_order) as i32)
    }

    /// Insertion loss in dB at cavity readout frequency $f_r$:
    /// $$\text{IL}(f) = 10 \log_{10}(S(f))$$
    pub fn insertion_loss_db_at_cavity(&self, cavity_freq_ghz: f64) -> f64 {
        let s = self.suppression_ratio_at_freq(cavity_freq_ghz);
        10.0 * s.log10()
    }

    /// Raw (unfiltered) spontaneous Purcell radiative decay rate $\gamma_{Purcell} / (2\pi)$ in kHz:
    /// $$\gamma_{Purcell} = \kappa_r \left( \frac{g}{\Delta} \right)^2$$
    pub fn unfiltered_purcell_rate_khz(&self, cqed: &DispersiveCqedSystem) -> f64 {
        let kappa_khz = cqed.cavity.kappa_total_mhz() * 1.0e3;
        let g_mhz = cqed.coupling_g_mhz;
        let delta_mhz = cqed.detuning_ghz().abs() * 1.0e3;
        let coupling_ratio = g_mhz / delta_mhz.max(1e-3);
        kappa_khz * coupling_ratio.powi(2)
    }

    /// Filtered Purcell radiative decay rate $\gamma_{filtered} / (2\pi)$ in kHz:
    /// $$\gamma_{filtered} = \frac{\gamma_{Purcell}}{S(f_{01})}$$
    pub fn filtered_purcell_rate_khz(&self, cqed: &DispersiveCqedSystem) -> f64 {
        let gamma_raw = self.unfiltered_purcell_rate_khz(cqed);
        let s_qubit = self.suppression_ratio_at_freq(cqed.transmon.omega_01_ghz());
        gamma_raw / s_qubit.max(1.0)
    }

    /// Unfiltered Purcell relaxation time limit $T_{1,Purcell}$ in microseconds ($\mu\text{s}$).
    pub fn unfiltered_purcell_t1_us(&self, cqed: &DispersiveCqedSystem) -> f64 {
        let rate_hz = self.unfiltered_purcell_rate_khz(cqed) * 1.0e3 * 2.0 * std::f64::consts::PI;
        1.0e6 / rate_hz.max(1e-3)
    }

    /// Filtered Purcell relaxation time limit $T_{1,filtered}$ in microseconds ($\mu\text{s}$).
    pub fn filtered_purcell_t1_us(&self, cqed: &DispersiveCqedSystem) -> f64 {
        let rate_hz = self.filtered_purcell_rate_khz(cqed) * 1.0e3 * 2.0 * std::f64::consts::PI;
        1.0e6 / rate_hz.max(1e-3)
    }
}
