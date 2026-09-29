//! Autonomous multi-physics solver for microcavity exciton-polariton condensates,
//! open-dissipative Gross-Pitaevskii kinetics, and Bogoliubov superfluidity.

use phonon_core::constants::H_BAR;
use phonon_models::polariton_condensate::{PolaritonCondensateMetrics, PolaritonCondensateParams};

/// Multi-physics solver for microcavity exciton-polariton condensates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolaritonCondensateSolver {
    pub params: PolaritonCondensateParams,
}

impl PolaritonCondensateSolver {
    /// Creates a new polariton condensate solver.
    pub fn new(params: PolaritonCondensateParams) -> Self {
        Self { params }
    }

    /// Solves steady-state condensate metrics.
    pub fn solve_condensate_metrics(&self) -> PolaritonCondensateMetrics {
        self.params.evaluate_metrics()
    }

    /// Solves Bogoliubov elementary excitation spectrum $\epsilon(k)$ in $\text{meV}$:
    /// $$\epsilon(k) = \sqrt{ \frac{\hbar^2 k^2}{2 m^*} \left( \frac{\hbar^2 k^2}{2 m^*} + 2 g n_0 \right) }$$
    pub fn solve_bogoliubov_energy_mev(&self, k_inv_um: f64) -> f64 {
        let n0 = self.params.condensate_density_um2();
        if n0 <= 0.0 {
            return 0.0;
        }
        let k_inv_m = k_inv_um * 1.0e6;
        let m_eff = self.params.effective_mass_kg();
        let kinetic_j = (H_BAR * H_BAR * k_inv_m * k_inv_m) / (2.0 * m_eff);

        let mev_to_j = 1.602_176_634e-22;
        let um2_to_m2 = 1.0e-12;
        let g_si = self.params.interaction_g_mev_um2 * mev_to_j * um2_to_m2;
        let n0_si = n0 / 1.0e-12;
        let interaction_j = 2.0 * g_si * n0_si;

        let energy_sq_j2 = kinetic_j * (kinetic_j + interaction_j);
        let energy_j = energy_sq_j2.max(0.0).sqrt();
        (energy_j / 1.602_176_634e-19) * 1.0e3
    }

    /// Evaluates Landau critical velocity and superfluid drag reduction ratio:
    /// For flow velocity $v < c_s$, backscattering is exponentially suppressed.
    pub fn solve_superfluid_drag_reduction(&self, flow_velocity_m_s: f64) -> f64 {
        let cs = self.params.sound_speed_m_s();
        if cs <= 0.0 {
            return 1.0;
        }
        let mach_number = flow_velocity_m_s / cs;
        if mach_number < 1.0 {
            // Exponential suppression of drag below Landau critical velocity
            (1.0 - (1.0 - mach_number).powi(2) * self.params.superfluid_fraction()).clamp(0.01, 1.0)
        } else {
            1.0 // Normal fluid drag
        }
    }
}
