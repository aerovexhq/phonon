//! Optomechanical cavity and resonator models: Fabry-Pérot cavities and photonic crystal nanobeams.
//!
//! Formulates:
//! - Radiation pressure interaction Hamiltonian:
//!   $$\hat{H}_{om} = -\hbar g_0 \hat{a}^\dagger \hat{a} (\hat{b} + \hat{b}^\dagger)$$
//! - Zero-point fluctuation amplitude:
//!   $$x_{zpf} = \sqrt{\frac{\hbar}{2 m_{eff} \Omega_m}}$$
//! - Frequency pull parameter:
//!   $$G = \frac{\partial \omega_c}{\partial x}, \quad g_0 = G x_{zpf}$$
//! - Multi-photon enhanced coupling:
//!   $$g = g_0 \sqrt{\bar{n}_{cav}}$$
//! - Optomechanical cooperativity:
//!   $$\mathcal{C} = \frac{4 g^2}{\kappa \gamma_m}$$

use phonon_core::constants::{BOLTZMANN_CONSTANT, H_BAR};

/// Physical type of the optomechanical cavity architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptomechanicalSystemType {
    /// Macroscopic or microscopic Fabry-Pérot cavity with moving end mirror.
    FabryPerot,
    /// Co-localized optical and phononic defect in a 1D photonic crystal nanobeam.
    PhotonicCrystalNanobeam,
    /// Whispering gallery mode microtoroid or microsphere.
    Microtoroid,
}

/// Specifications and parameters for an optomechanical cavity system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OptomechanicalParams {
    /// Architecture classification.
    pub system_type: OptomechanicalSystemType,
    /// Optical cavity resonance frequency $\omega_c / (2\pi)$ in Hz.
    pub optical_frequency_hz: f64,
    /// Total optical cavity linewidth / decay rate $\kappa / (2\pi)$ in Hz.
    pub cavity_linewidth_hz: f64,
    /// External optical coupling rate $\kappa_{ex} / (2\pi)$ in Hz.
    pub external_coupling_hz: f64,
    /// Mechanical resonator frequency $\Omega_m / (2\pi)$ in Hz.
    pub mechanical_frequency_hz: f64,
    /// Mechanical damping rate $\gamma_m / (2\pi)$ in Hz.
    pub mechanical_damping_hz: f64,
    /// Effective motional mass $m_{eff}$ in kg.
    pub effective_mass_kg: f64,
    /// Single-photon vacuum optomechanical coupling rate $g_0 / (2\pi)$ in Hz.
    pub vacuum_coupling_g0_hz: f64,
}

impl OptomechanicalParams {
    /// Creates a new optomechanical parameter set.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        system_type: OptomechanicalSystemType,
        optical_frequency_hz: f64,
        cavity_linewidth_hz: f64,
        external_coupling_hz: f64,
        mechanical_frequency_hz: f64,
        mechanical_damping_hz: f64,
        effective_mass_kg: f64,
        vacuum_coupling_g0_hz: f64,
    ) -> Self {
        Self {
            system_type,
            optical_frequency_hz,
            cavity_linewidth_hz,
            external_coupling_hz,
            mechanical_frequency_hz,
            mechanical_damping_hz,
            effective_mass_kg,
            vacuum_coupling_g0_hz,
        }
    }

    /// Standard high-Q silicon photonic crystal nanobeam:
    /// - $\lambda \approx 1550\text{ nm}$ ($193.4\text{ THz}$)
    /// - $\Omega_m / (2\pi) = 5.0\text{ GHz}$
    /// - $m_{eff} \approx 300\text{ fg} = 3.0 \times 10^{-16}\text{ kg}$
    /// - $g_0 / (2\pi) = 1.1\text{ MHz}$
    /// - $\kappa / (2\pi) = 15.0\text{ MHz}$ ($Q_c \approx 1.3 \times 10^7$)
    /// - $\gamma_m / (2\pi) = 5.0\text{ kHz}$ ($Q_m = 10^6$)
    pub fn standard_nanobeam() -> Self {
        Self::new(
            OptomechanicalSystemType::PhotonicCrystalNanobeam,
            193.4e12,
            15.0e6,
            7.5e6,
            5.0e9,
            5.0e3,
            3.0e-16,
            1.1e6,
        )
    }

    /// Standard high-finesse Fabry-Pérot cavity with micro-mirror:
    /// - $\lambda \approx 1064\text{ nm}$ ($281.7\text{ THz}$)
    /// - $\Omega_m / (2\pi) = 10.0\text{ MHz}$
    /// - $m_{eff} = 10.0\text{ ng} = 1.0 \times 10^{-11}\text{ kg}$
    /// - $g_0 / (2\pi) = 2.5\text{ kHz}$
    /// - $\kappa / (2\pi) = 500.0\text{ kHz}$
    /// - $\gamma_m / (2\pi) = 10.0\text{ Hz}$ ($Q_m = 10^6$)
    pub fn standard_fabry_perot() -> Self {
        Self::new(
            OptomechanicalSystemType::FabryPerot,
            281.7e12,
            500.0e3,
            250.0e3,
            10.0e6,
            10.0,
            1.0e-11,
            2.5e3,
        )
    }

    /// Mechanical zero-point fluctuation amplitude $x_{zpf} = \sqrt{\frac{\hbar}{2 m_{eff} \Omega_m}}$ in meters.
    pub fn zero_point_fluctuation_m(&self) -> f64 {
        let omega_m_rad = 2.0 * std::f64::consts::PI * self.mechanical_frequency_hz;
        (H_BAR / (2.0 * self.effective_mass_kg * omega_m_rad.max(1.0))).sqrt()
    }

    /// Optical cavity quality factor $Q_c = \omega_c / \kappa$.
    pub fn optical_quality_factor(&self) -> f64 {
        self.optical_frequency_hz / self.cavity_linewidth_hz.max(1.0)
    }

    /// Mechanical resonator quality factor $Q_m = \Omega_m / \gamma_m$.
    pub fn mechanical_quality_factor(&self) -> f64 {
        self.mechanical_frequency_hz / self.mechanical_damping_hz.max(1e-6)
    }

    /// Frequency pull parameter $G = \partial \omega_c / \partial x = g_0 / x_{zpf}$ in rad/(s*m).
    pub fn frequency_pull_parameter_rad_per_s_m(&self) -> f64 {
        let g0_rad = 2.0 * std::f64::consts::PI * self.vacuum_coupling_g0_hz;
        let x_zpf = self.zero_point_fluctuation_m();
        g0_rad / x_zpf.max(1e-25)
    }

    /// Resolved sideband regime criterion: $\Omega_m / \kappa > 1.0$.
    pub fn is_resolved_sideband(&self) -> bool {
        self.sideband_resolution_parameter() > 1.0
    }

    /// Sideband resolution ratio $\Omega_m / \kappa$.
    pub fn sideband_resolution_parameter(&self) -> f64 {
        self.mechanical_frequency_hz / self.cavity_linewidth_hz.max(1.0)
    }

    /// Effective linearized coupling rate $g / (2\pi) = g_0 \sqrt{\bar{n}_{cav}}$ in Hz.
    pub fn effective_coupling_hz(&self, intracavity_photons: f64) -> f64 {
        self.vacuum_coupling_g0_hz * intracavity_photons.max(0.0).sqrt()
    }

    /// Multi-photon optomechanical cooperativity $\mathcal{C} = \frac{4 g^2}{\kappa \gamma_m}$.
    pub fn cooperativity(&self, intracavity_photons: f64) -> f64 {
        let g = self.effective_coupling_hz(intracavity_photons);
        (4.0 * g * g) / (self.cavity_linewidth_hz * self.mechanical_damping_hz).max(1e-12)
    }

    /// Thermal phonon occupancy $n_{th} = \frac{k_B T}{\hbar \Omega_m}$ at bath temperature $T_{bath}$ in Kelvin.
    pub fn thermal_phonon_occupancy(&self, bath_temperature_k: f64) -> f64 {
        let omega_m_rad = 2.0 * std::f64::consts::PI * self.mechanical_frequency_hz;
        let exponent = (H_BAR * omega_m_rad) / (BOLTZMANN_CONSTANT * bath_temperature_k.max(1e-6));
        if exponent > 80.0 {
            0.0
        } else if exponent < 1e-4 {
            // Classical equipartition limit
            (BOLTZMANN_CONSTANT * bath_temperature_k) / (H_BAR * omega_m_rad)
        } else {
            1.0 / (exponent.exp() - 1.0)
        }
    }
}
