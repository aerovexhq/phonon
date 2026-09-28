//! Dynamical backaction, resolved-sideband cooling, and quantum ground-state synthesis.
//!
//! Formulates:
//! - Optical damping rate:
//!   $$\Gamma_{opt}(\Delta) = 4 g^2 \left( \frac{\kappa}{\kappa^2 + 4(\Delta + \Omega_m)^2} - \frac{\kappa}{\kappa^2 + 4(\Delta - \Omega_m)^2} \right)$$
//! - Optical spring frequency shift:
//!   $$\delta\Omega_m(\Delta) = 2 g^2 \left( \frac{\Delta + \Omega_m}{\kappa^2 + 4(\Delta + \Omega_m)^2} + \frac{\Delta - \Omega_m}{\kappa^2 + 4(\Delta - \Omega_m)^2} \right)$$
//! - Total effective mechanical linewidth:
//!   $$\gamma_{eff} = \gamma_m + \Gamma_{opt}$$
//! - Quantum backaction cooling limit:
//!   $$n_{min} = \frac{\kappa^2}{16 \Omega_m^2}$$
//! - Effective cooled phonon occupancy:
//!   $$\bar{n}_m = \frac{\gamma_m n_{th} + \Gamma_{opt} n_{min}}{\gamma_m + \Gamma_{opt}} = \frac{n_{th}}{1 + \mathcal{C}} + n_{min} \frac{\mathcal{C}}{1 + \mathcal{C}}$$

use super::fabry_perot_nanobeam::OptomechanicalParams;
use phonon_core::constants::{BOLTZMANN_CONSTANT, H_BAR};

/// Parameters for dynamical backaction sideband cooling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SidebandCoolingParams {
    /// Underlying optomechanical cavity system.
    pub system: OptomechanicalParams,
    /// Laser detuning $\Delta = \omega_L - \omega_c$ in Hz (e.g. $-\Omega_m$ for red sideband cooling).
    pub detuning_hz: f64,
    /// Mean intracavity photon number $\bar{n}_{cav}$.
    pub intracavity_photons: f64,
    /// Cryogenic bath temperature $T_{bath}$ in Kelvin.
    pub bath_temperature_k: f64,
}

impl SidebandCoolingParams {
    /// Constructs sideband cooling parameters.
    pub fn new(
        system: OptomechanicalParams,
        detuning_hz: f64,
        intracavity_photons: f64,
        bath_temperature_k: f64,
    ) -> Self {
        Self {
            system,
            detuning_hz,
            intracavity_photons,
            bath_temperature_k,
        }
    }

    /// Standard red-sideband configuration for ground-state cooling of a nanobeam at 4 Kelvin:
    /// $\Delta = -\Omega_m = -5.0\text{ GHz}$, $\bar{n}_{cav} = 3000$ photons.
    pub fn standard_ground_state_nanobeam() -> Self {
        let system = OptomechanicalParams::standard_nanobeam();
        let detuning = -system.mechanical_frequency_hz;
        Self::new(system, detuning, 3_000.0, 4.0)
    }

    /// Optomechanical dynamical backaction damping rate $\Gamma_{opt}(\Delta)$ in Hz:
    /// $$\Gamma_{opt}(\Delta) = 4 g^2 \left( \frac{\kappa}{\kappa^2 + 4(\Delta + \Omega_m)^2} - \frac{\kappa}{\kappa^2 + 4(\Delta - \Omega_m)^2} \right)$$
    pub fn optical_damping_hz(&self) -> f64 {
        let g = self.system.effective_coupling_hz(self.intracavity_photons);
        let kappa = self.system.cavity_linewidth_hz;
        let omega_m = self.system.mechanical_frequency_hz;
        let delta = self.detuning_hz;

        let term_plus = kappa / (kappa * kappa + 4.0 * (delta + omega_m).powi(2));
        let term_minus = kappa / (kappa * kappa + 4.0 * (delta - omega_m).powi(2));

        4.0 * g * g * (term_plus - term_minus)
    }

    /// Optical spring effect frequency shift $\delta\Omega_m(\Delta)$ in Hz:
    /// $$\delta\Omega_m(\Delta) = 2 g^2 \left( \frac{\Delta + \Omega_m}{\kappa^2 + 4(\Delta + \Omega_m)^2} + \frac{\Delta - \Omega_m}{\kappa^2 + 4(\Delta - \Omega_m)^2} \right)$$
    pub fn optical_spring_shift_hz(&self) -> f64 {
        let g = self.system.effective_coupling_hz(self.intracavity_photons);
        let kappa = self.system.cavity_linewidth_hz;
        let omega_m = self.system.mechanical_frequency_hz;
        let delta = self.detuning_hz;

        let term_plus = (delta + omega_m) / (kappa * kappa + 4.0 * (delta + omega_m).powi(2));
        let term_minus = (delta - omega_m) / (kappa * kappa + 4.0 * (delta - omega_m).powi(2));

        2.0 * g * g * (term_plus + term_minus)
    }

    /// Total effective mechanical linewidth $\gamma_{eff} = \gamma_m + \Gamma_{opt}$ in Hz.
    pub fn effective_mechanical_damping_hz(&self) -> f64 {
        self.system.mechanical_damping_hz + self.optical_damping_hz()
    }

    /// Effective mechanical resonance frequency $\Omega_{eff} = \Omega_m + \delta\Omega_m$ in Hz.
    pub fn effective_mechanical_frequency_hz(&self) -> f64 {
        self.system.mechanical_frequency_hz + self.optical_spring_shift_hz()
    }

    /// Quantum backaction cooling limit $n_{min} = \frac{\kappa^2}{16 \Omega_m^2}$.
    pub fn quantum_backaction_limit(&self) -> f64 {
        let kappa = self.system.cavity_linewidth_hz;
        let omega_m = self.system.mechanical_frequency_hz;
        (kappa * kappa) / (16.0 * omega_m * omega_m).max(1e-12)
    }

    /// Effective cooled mean phonon occupancy $\bar{n}_m$:
    /// $$\bar{n}_m = \frac{\gamma_m n_{th} + \Gamma_{opt} n_{min}}{\gamma_m + \Gamma_{opt}}$$
    pub fn cooled_phonon_occupancy(&self) -> f64 {
        let n_th = self
            .system
            .thermal_phonon_occupancy(self.bath_temperature_k);
        let gamma_m = self.system.mechanical_damping_hz;
        let gamma_opt = self.optical_damping_hz().max(0.0);
        let n_min = self.quantum_backaction_limit();

        (gamma_m * n_th + gamma_opt * n_min) / (gamma_m + gamma_opt).max(1e-12)
    }

    /// Verifies if the mechanical mode is cooled into the quantum ground state ($\bar{n}_m < 1.0$, and strictly $< 0.1$).
    pub fn is_ground_state_cooled(&self) -> bool {
        self.cooled_phonon_occupancy() < 0.1
    }

    /// Effective mode temperature $T_{eff} = \frac{\hbar \Omega_m}{k_B \ln(1 + 1/\bar{n}_m)}$ in Kelvin.
    pub fn effective_temperature_k(&self) -> f64 {
        let n_bar = self.cooled_phonon_occupancy();
        if n_bar < 1e-12 {
            return 0.0;
        }
        let omega_m_rad = 2.0 * std::f64::consts::PI * self.system.mechanical_frequency_hz;
        (H_BAR * omega_m_rad) / (BOLTZMANN_CONSTANT * (1.0 + 1.0 / n_bar).ln())
    }
}
