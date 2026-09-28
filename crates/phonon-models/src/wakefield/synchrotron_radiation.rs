//! Betatron oscillations and synchrotron X-ray radiation emission:
//! critical photon energy, radiated power, and classical/QED radiation reaction.

use crate::wakefield::PlasmaChannelParams;
use std::f64::consts::PI;

/// Betatron oscillation dynamics and synchrotron radiation emission.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BetatronRadiation {
    /// Plasma channel parameters.
    pub plasma: PlasmaChannelParams,
}

impl BetatronRadiation {
    /// Constructs a betatron radiation model.
    pub fn new(plasma: PlasmaChannelParams) -> Self {
        Self { plasma }
    }

    /// Betatron oscillation frequency $\omega_\beta = \omega_p / \sqrt{2\gamma}$ in rad/s:
    pub fn betatron_frequency_rad_per_s(&self, gamma: f64) -> f64 {
        let omegap = self.plasma.plasma_frequency_rad_per_s();
        omegap / (2.0 * gamma.max(1.0)).sqrt()
    }

    /// Betatron strength parameter $K_\beta = \gamma k_\beta r_\beta$:
    pub fn betatron_strength_parameter(&self, gamma: f64, betatron_radius_m: f64) -> f64 {
        let c = 2.997_924_58e8;
        let omega_b = self.betatron_frequency_rad_per_s(gamma);
        let k_b = omega_b / c;
        gamma * k_b * betatron_radius_m
    }

    /// Critical synchrotron X-ray photon energy $\hbar \omega_c$ in keV:
    /// $$\hbar \omega_c = \frac{3}{2} \gamma^3 \hbar c \kappa_{curv} = \frac{3}{4} \hbar \gamma^2 \omega_p^2 r_\beta / c$$
    pub fn critical_photon_energy_kev(&self, gamma: f64, betatron_radius_m: f64) -> f64 {
        let hbar_ev = 6.582_119_569e-16; // eV * s
        let c = 2.997_924_58e8;
        let omegap = self.plasma.plasma_frequency_rad_per_s();
        let energy_ev = 0.75 * hbar_ev * gamma.powi(2) * omegap.powi(2) * betatron_radius_m / c;
        energy_ev * 1.0e-3 // keV
    }

    /// Classical synchrotron radiated power $P_{rad}$ in Watts (Larmor-Schwinger formula):
    /// $$P_{rad} = \frac{2}{3} \frac{e^2}{4\pi\epsilon_0 c} \gamma^4 \left( \frac{c}{\rho} \right)^2 = \frac{e^2 \gamma^2 \omega_p^4 r_\beta^2}{12 \pi \epsilon_0 c^3}$$
    pub fn radiated_power_watts(&self, gamma: f64, betatron_radius_m: f64) -> f64 {
        let eps0: f64 = 8.854_187_812_8e-12;
        let c: f64 = 2.997_924_58e8;
        let e: f64 = 1.602_176_634e-19;
        let omegap = self.plasma.plasma_frequency_rad_per_s();
        let num = e.powi(2) * gamma.powi(2) * omegap.powi(4) * betatron_radius_m.powi(2);
        let denom = 12.0 * PI * eps0 * c.powi(3);
        num / denom.max(1e-30)
    }

    /// Non-linear QED quantum parameter $\chi_e = \gamma E_\perp / E_{Schwinger}$:
    /// where $E_{Schwinger} = m_e^2 c^3 / (e \hbar) \approx 1.32 \times 10^{18}\,\text{V/m}$.
    pub fn qed_quantum_parameter(&self, gamma: f64, transverse_field_v_per_m: f64) -> f64 {
        let e_schwinger = 1.323_28e18; // V/m
        gamma * transverse_field_v_per_m.abs() / e_schwinger
    }

    /// Rate of electron Lorentz factor loss due to radiation reaction $d\gamma / dt$ in $s^{-1}$:
    pub fn radiation_damping_rate_per_s(&self, gamma: f64, betatron_radius_m: f64) -> f64 {
        let me: f64 = 9.109_383_7e-31;
        let c: f64 = 2.997_924_58e8;
        let p_rad = self.radiated_power_watts(gamma, betatron_radius_m);
        p_rad / (me * c.powi(2))
    }
}
