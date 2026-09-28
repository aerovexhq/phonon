//! 3D microwave cavity and Coplanar Waveguide (CPW) transmission line resonators.

use std::f64::consts::PI;

/// Physical parameters of a microwave readout cavity / CPW resonator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MicrowaveCavity {
    /// Bare resonance frequency $f_r = \omega_r / (2\pi)$ in GHz.
    pub resonance_frequency_ghz: f64,
    /// Internal / intrinsic quality factor $Q_{int}$ (material and dielectric losses).
    pub internal_q: f64,
    /// External / coupling quality factor $Q_{ext}$ (feedline coupling).
    pub coupling_q: f64,
    /// Characteristic impedance $Z_0$ in Ohms (typically 50 Ohms).
    pub characteristic_impedance_ohms: f64,
}

impl MicrowaveCavity {
    /// Constructs a new microwave cavity resonator.
    pub fn new(
        resonance_frequency_ghz: f64,
        internal_q: f64,
        coupling_q: f64,
        characteristic_impedance_ohms: f64,
    ) -> Self {
        Self {
            resonance_frequency_ghz,
            internal_q,
            coupling_q,
            characteristic_impedance_ohms,
        }
    }

    /// Standard 7.0 GHz CPW readout resonator with $Q_{ext} \approx 10^4$ and $Q_{int} \approx 5 \times 10^5$.
    pub fn standard_7ghz() -> Self {
        Self::new(7.0, 500_000.0, 10_000.0, 50.0)
    }

    /// Total loaded quality factor $Q_L$:
    /// $$1 / Q_L = 1 / Q_{int} + 1 / Q_{ext}$$
    pub fn loaded_q(&self) -> f64 {
        let inv_q = (1.0 / self.internal_q.max(1.0)) + (1.0 / self.coupling_q.max(1.0));
        1.0 / inv_q
    }

    /// Total photon loss rate $\kappa / (2\pi) = f_r / Q_L$ in MHz.
    pub fn kappa_total_mhz(&self) -> f64 {
        (self.resonance_frequency_ghz * 1.0e3) / self.loaded_q()
    }

    /// External photon exit rate $\kappa_{ext} / (2\pi) = f_r / Q_{ext}$ in MHz.
    pub fn kappa_ext_mhz(&self) -> f64 {
        (self.resonance_frequency_ghz * 1.0e3) / self.coupling_q.max(1.0)
    }

    /// Internal loss rate $\kappa_{int} / (2\pi) = f_r / Q_{int}$ in MHz.
    pub fn kappa_int_mhz(&self) -> f64 {
        (self.resonance_frequency_ghz * 1.0e3) / self.internal_q.max(1.0)
    }

    /// Cavity photon lifetime $\tau_{cav} = 1 / \kappa$ in nanoseconds.
    pub fn photon_lifetime_ns(&self) -> f64 {
        let kappa_hz = self.kappa_total_mhz() * 1.0e6 * 2.0 * PI;
        1.0e9 / kappa_hz.max(1.0)
    }

    /// Equivalent Foster lumped-element series inductance $L_r$ in nanohenries:
    /// $$L_r = \frac{Z_0}{\omega_r}$$
    pub fn foster_inductance_nh(&self) -> f64 {
        let omega = 2.0 * PI * self.resonance_frequency_ghz * 1.0e9;
        (self.characteristic_impedance_ohms / omega) * 1.0e9
    }

    /// Equivalent Foster lumped-element parallel capacitance $C_r$ in picofarads:
    /// $$C_r = \frac{1}{\omega_r Z_0}$$
    pub fn foster_capacitance_pf(&self) -> f64 {
        let omega = 2.0 * PI * self.resonance_frequency_ghz * 1.0e9;
        (1.0 / (omega * self.characteristic_impedance_ohms)) * 1.0e12
    }

    /// Generates harmonic resonance poles for a quarter-wave distributed CPW resonator:
    /// $$f_k = (2k - 1) f_r$$
    pub fn harmonic_frequencies_ghz(&self, num_harmonics: usize) -> Vec<f64> {
        (1..=num_harmonics)
            .map(|k| (2 * k - 1) as f64 * self.resonance_frequency_ghz)
            .collect()
    }
}
