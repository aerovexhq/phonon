//! Antiferromagnetic material parameters, exchange-dominated spin waves,
//! and Terahertz resonance dynamics.

use crate::cavity_spintronics::GAMMA_ELECTRON;
use phonon_core::constants::PLANCK_CONSTANT;
use std::f64::consts::PI;

/// Parameters for an exchange-dominated antiferromagnetic material (e.g. NiO, MnF2, alpha-Fe2O3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AfmMaterialParams {
    /// Saturation magnetization per sublattice $M_s$ in $\text{A/m}$ (nominal $4.0 \times 10^5\text{ A/m}$).
    pub sublattice_ms_a_m: f64,
    /// Effective inter-sublattice exchange field $H_{\mathrm{ex}}$ in Tesla ($T$) (nominal $100 - 800\text{ T}$).
    pub exchange_field_tesla: f64,
    /// Uniaxial/easy-axis magnetic anisotropy field $H_A$ in Tesla ($T$) (nominal $0.1 - 5.0\text{ T}$).
    pub anisotropy_field_tesla: f64,
    /// Exchange stiffness constant $A_{\mathrm{ex}}$ in $\text{J/m}$ (nominal $1.0 \times 10^{-11}\text{ J/m}$).
    pub exchange_stiffness_j_m: f64,
    /// Dimensionless Gilbert damping coefficient $\alpha$ (nominal $1.0 \times 10^{-4} - 1.0 \times 10^{-3}$).
    pub gilbert_damping: f64,
    /// Dzyaloshinskii-Moriya Interaction (DMI) constant $D$ in $\text{J/m}^2$ (nominal $0.0 - 2.0\text{ mJ/m}^2$).
    pub dmi_constant_j_m2: f64,
}

impl Default for AfmMaterialParams {
    fn default() -> Self {
        Self {
            sublattice_ms_a_m: 4.0e5,
            exchange_field_tesla: 350.0,
            anisotropy_field_tesla: 1.2,
            exchange_stiffness_j_m: 1.0e-11,
            gilbert_damping: 5.0e-4,
            dmi_constant_j_m2: 1.5e-3,
        }
    }
}

impl AfmMaterialParams {
    /// Creates new antiferromagnetic material parameters.
    pub fn new(
        exchange_field_tesla: f64,
        anisotropy_field_tesla: f64,
        exchange_stiffness_j_m: f64,
        gilbert_damping: f64,
    ) -> Self {
        Self {
            sublattice_ms_a_m: 4.0e5,
            exchange_field_tesla: exchange_field_tesla.max(10.0),
            anisotropy_field_tesla: anisotropy_field_tesla.max(0.01),
            exchange_stiffness_j_m: exchange_stiffness_j_m.max(1e-13),
            gilbert_damping: gilbert_damping.clamp(1e-5, 0.1),
            dmi_constant_j_m2: 1.5e-3,
        }
    }

    /// Evaluates the Kittel Antiferromagnetic Resonance (AFMR) angular frequency $\omega_{\mathrm{afmr}}$ in $\text{rad/s}$:
    /// $$\omega_{\mathrm{afmr}} = \gamma \sqrt{2 H_{\mathrm{ex}} H_A + H_A^2}$$
    pub fn afmr_angular_frequency_rad_s(&self) -> f64 {
        let hex = self.exchange_field_tesla;
        let ha = self.anisotropy_field_tesla;
        let arg = (2.0 * hex * ha + ha * ha).max(0.0);
        GAMMA_ELECTRON * arg.sqrt()
    }

    /// Evaluates the AFMR frequency in Terahertz ($\text{THz}$):
    /// $$f_{\mathrm{afmr}} = \frac{\omega_{\mathrm{afmr}}}{2\pi} \in [0.1, 3.0\text{ THz}]$$
    pub fn afmr_frequency_thz(&self) -> f64 {
        self.afmr_angular_frequency_rad_s() / (2.0 * PI * 1.0e12)
    }

    /// Evaluates the exchange spin-wave group velocity $v_{\mathrm{ex}}$ in $\text{m/s}$:
    /// $$v_{\mathrm{ex}} = \gamma \sqrt{\frac{2 H_{\mathrm{ex}} A_{\mathrm{ex}}}{M_s}}$$
    pub fn exchange_velocity_m_s(&self) -> f64 {
        let hex = self.exchange_field_tesla;
        let a_ex = self.exchange_stiffness_j_m;
        let ms = self.sublattice_ms_a_m;
        let ratio = (2.0 * hex * a_ex) / ms;
        GAMMA_ELECTRON * ratio.max(0.0).sqrt()
    }

    /// Evaluates spin-wave dispersion frequency $\omega(k)$ at wavevector $k$ in $\text{rad/s}$:
    /// $$\omega(k) = \sqrt{\omega_{\mathrm{afmr}}^2 + v_{\mathrm{ex}}^2 k^2}$$
    pub fn spin_wave_frequency_rad_s(&self, k_inv_m: f64) -> f64 {
        let w0 = self.afmr_angular_frequency_rad_s();
        let vex = self.exchange_velocity_m_s();
        (w0 * w0 + vex * vex * k_inv_m * k_inv_m).sqrt()
    }

    /// Evaluates spin-wave energy $\hbar \omega(k)$ in $\text{meV}$.
    pub fn spin_wave_energy_mev(&self, k_inv_m: f64) -> f64 {
        let hbar = PLANCK_CONSTANT / (2.0 * PI);
        let ev_to_j = 1.602176634e-19;
        let energy_j = hbar * self.spin_wave_frequency_rad_s(k_inv_m);
        (energy_j / ev_to_j) * 1.0e3
    }
}
