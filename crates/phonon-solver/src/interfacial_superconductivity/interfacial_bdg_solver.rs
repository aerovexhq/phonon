//! Self-consistent Bogoliubov-de Gennes solver with forward-scattering electron-phonon coupling
//! and electronic nematic gap anisotropy in monolayer FeSe/STO.

use phonon_models::interfacial_superconductivity::{InterfacialScParams, NematicOrderParams};

/// Multi-physics Bogoliubov-de Gennes solver for interfacial high-Tc superconductivity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InterfacialBdgSolver {
    pub sc_params: InterfacialScParams,
    pub nematic_params: NematicOrderParams,
}

impl InterfacialBdgSolver {
    /// Creates a new interfacial BdG solver.
    pub fn new(sc_params: InterfacialScParams, nematic_params: NematicOrderParams) -> Self {
        Self {
            sc_params,
            nematic_params,
        }
    }

    /// Solves the enhanced critical temperature $T_c$ in Kelvin ($K$).
    pub fn solve_critical_temperature(&self) -> f64 {
        self.sc_params.critical_temperature_kelvin()
    }

    /// Solves the strong-coupling zero-temperature superconducting gap $\Delta_0$ in $\text{meV}$.
    pub fn solve_zero_temp_gap_mev(&self) -> f64 {
        self.sc_params.zero_temperature_gap_mev()
    }

    /// Solves the temperature-dependent superconducting gap $\Delta(T)$ in $\text{meV}$ using BCS interpolation:
    /// $$\Delta(T) = \Delta_0 \tanh\left( 1.74 \sqrt{\max(0, T_c / T - 1)} \right)$$
    pub fn solve_temp_dependent_gap_mev(&self, temperature_k: f64) -> f64 {
        let tc = self.solve_critical_temperature();
        if temperature_k >= tc {
            0.0
        } else {
            let delta_0 = self.solve_zero_temp_gap_mev();
            let reduced_arg = (tc / temperature_k.max(1e-6) - 1.0).max(0.0).sqrt();
            delta_0 * (1.74 * reduced_arg).tanh()
        }
    }

    /// Solves the angle-resolved anisotropic gap $\Delta(\theta)$ in $\text{meV}$.
    pub fn solve_anisotropic_gap(&self, angle_rad: f64) -> f64 {
        let delta_0 = self.solve_zero_temp_gap_mev();
        self.nematic_params.anisotropic_gap_mev(delta_0, angle_rad)
    }

    /// Solves the Curie-Weiss elastoresistance coefficient $2m_{66}(T)$.
    pub fn solve_elastoresistance(&self, temperature_k: f64) -> f64 {
        self.nematic_params.elastoresistance_2m66(temperature_k)
    }
}
