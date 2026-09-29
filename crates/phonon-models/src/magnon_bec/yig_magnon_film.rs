//! YIG Magnon Film & Non-Equilibrium Bose-Einstein Condensation Models.
//!
//! Formulates dipolar-exchange spin-wave dispersion in YIG thin films,
//! four-magnon scattering thermalization, critical microwave pumping threshold,
//! and non-equilibrium chemical potential saturation at the bottom of the spectrum.

use std::f64::consts::PI;

/// Reduced Planck constant in J·s.
pub const HBAR: f64 = 1.054_571_817e-34;
/// Boltzmann constant in J/K.
pub const BOLTZMANN_K: f64 = 1.380_649e-23;
/// Vacuum permeability in H/m.
pub const MU_0: f64 = 4.0 * PI * 1.0e-7;
/// Electron gyromagnetic ratio in rad/(s·T).
pub const GAMMA_E: f64 = 1.760_859_630_23e11;

/// Physical parameters of a Yttrium Iron Garnet (YIG) thin film for magnon BEC.
#[derive(Debug, Clone, PartialEq)]
pub struct YigMagnonFilm {
    /// Saturation magnetization $M_s$ in A/m (default 1.4e5 A/m, ~1750 G).
    pub saturation_magnetization_a_per_m: f64,
    /// Exchange stiffness constant $A_{ex}$ in J/m (default 3.7e-12 J/m).
    pub exchange_stiffness_j_per_m: f64,
    /// Gilbert damping parameter $\alpha$ (dimensionless, default 3.0e-5).
    pub gilbert_damping: f64,
    /// Film thickness $d$ in meters (default 5.0 um).
    pub thickness_m: f64,
    /// Static bias magnetic field $B_0$ in Tesla (default 0.1 T = 100 mT).
    pub bias_field_tesla: f64,
    /// Operating temperature in Kelvin (default 300.0 K, room-temperature BEC).
    pub temperature_k: f64,
}

impl Default for YigMagnonFilm {
    fn default() -> Self {
        Self {
            saturation_magnetization_a_per_m: 1.4e5,
            exchange_stiffness_j_per_m: 3.7e-12,
            gilbert_damping: 3.0e-5,
            thickness_m: 5.0e-6,
            bias_field_tesla: 0.10,
            temperature_k: 300.0,
        }
    }
}

impl YigMagnonFilm {
    /// Exchange length $\ell_{ex} = \sqrt{\frac{2 A_{ex}}{\mu_0 M_s^2}}$ in meters.
    #[inline]
    pub fn exchange_length_m(&self) -> f64 {
        let ms = self.saturation_magnetization_a_per_m;
        (2.0 * self.exchange_stiffness_j_per_m / (MU_0 * ms.powi(2))).sqrt()
    }

    /// Larmor precession angular frequency $\omega_H = \gamma B_0$ in rad/s.
    #[inline]
    pub fn larmor_frequency_rad_per_s(&self) -> f64 {
        GAMMA_E * self.bias_field_tesla
    }

    /// Magnetization characteristic frequency $\omega_M = \gamma \mu_0 M_s$ in rad/s.
    #[inline]
    pub fn magnetization_frequency_rad_per_s(&self) -> f64 {
        GAMMA_E * MU_0 * self.saturation_magnetization_a_per_m
    }

    /// Wavevector $k_{min}$ in m$^{-1}$ at the dipolar-exchange energy minimum.
    /// In in-plane magnetized films, dipolar-exchange hybridization creates a minimum at finite k.
    #[inline]
    pub fn wavevector_minimum_per_m(&self) -> f64 {
        4.0e6 // ~ 4 x 10^4 cm^-1
    }

    /// Dipolar-exchange spin-wave dispersion angular frequency $\omega(k)$ in rad/s.
    pub fn spin_wave_frequency_rad_per_s(&self, k: f64) -> f64 {
        let wh = self.larmor_frequency_rad_per_s();
        let wm = self.magnetization_frequency_rad_per_s();
        let lex = self.exchange_length_m();
        let k_min = self.wavevector_minimum_per_m();

        // Parabolic expansion around finite k_min minimum:
        // omega(k) = omega_min + (hbar / (2 m*)) * (k - k_min)^2
        let w_min = wh.sqrt() * (wh + wm * 0.1).sqrt(); // Base minimum frequency
        let delta_k = k - k_min;
        let d2w_dk2 = 2.0 * wm * lex.powi(2) + 1.2e-4; // Curvature

        (w_min + 0.5 * d2w_dk2 * delta_k.powi(2)).max(wh)
    }

    /// Minimum spin-wave energy $E_{min} = \hbar \omega(k_{min})$ in Joules.
    #[inline]
    pub fn minimum_spin_wave_energy_joules(&self) -> f64 {
        HBAR * self.spin_wave_frequency_rad_per_s(self.wavevector_minimum_per_m())
    }

    /// Effective magnon mass $m^* = \frac{\hbar}{\partial^2 \omega / \partial k^2}$ in kg.
    pub fn effective_magnon_mass_kg(&self) -> f64 {
        let wm = self.magnetization_frequency_rad_per_s();
        let lex = self.exchange_length_m();
        let curvature = 2.0 * wm * lex.powi(2) + 1.2e-4;
        HBAR / curvature
    }

    /// Four-magnon scattering thermalization rate $\Gamma_{4m}$ in s$^{-1}$.
    /// Demonstrates rapid thermalization conserving total magnon number.
    pub fn four_magnon_scattering_rate_per_s(&self, magnon_density_per_m3: f64) -> f64 {
        let wm = self.magnetization_frequency_rad_per_s();
        let base_rate = 1.0e8; // 100 MHz base rate
        let density_factor = (magnon_density_per_m3 / 1.0e24).max(0.1);
        base_rate * density_factor * (wm / 3.0e10)
    }

    /// Critical microwave pumping threshold field $h_{crit}$ in A/m for BEC onset.
    pub fn critical_pumping_threshold_a_per_m(&self, pump_freq_hz: f64) -> f64 {
        let w_pump = 2.0 * PI * pump_freq_hz;
        let delta_h = (2.0 * self.gilbert_damping * w_pump) / GAMMA_E;
        let wm_field = MU_0 * self.saturation_magnetization_a_per_m;
        (delta_h * w_pump) / (GAMMA_E * wm_field)
    }

    /// Quasi-equilibrium magnon chemical potential $\mu_m$ in Joules.
    /// As pump power increases, $\mu_m$ saturates at $E_{min}$.
    pub fn chemical_potential_joules(&self, pump_power_ratio: f64) -> f64 {
        let e_min = self.minimum_spin_wave_energy_joules();
        if pump_power_ratio < 1.0 {
            e_min * (1.0 - (-pump_power_ratio).exp()) / (1.0 - (-1.0f64).exp())
        } else {
            e_min
        }
    }

    /// Condensate fraction $n_c / n_{total}$ as a function of pump power ratio $P / P_{crit}$.
    pub fn condensate_fraction(&self, pump_power_ratio: f64) -> f64 {
        if pump_power_ratio <= 1.0 {
            0.0
        } else {
            let excess = pump_power_ratio - 1.0;
            (excess / (excess + 1.0)).min(0.98)
        }
    }
}
