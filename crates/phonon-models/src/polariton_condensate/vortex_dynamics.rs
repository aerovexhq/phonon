//! Topological polariton vortices, quantized circulation,
//! and acoustic black hole event horizons.

use super::condensate_params::PolaritonCondensateParams;
use phonon_core::constants::H_BAR;
use std::f64::consts::PI;

/// Parameters for a quantized topological polariton vortex.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolaritonVortexParams {
    /// Topological winding number (quantized circulation charge) $\ell \in \{-3, -2, -1, 1, 2, 3\}$.
    pub winding_number: i32,
    /// Core radius $r_0$ in micrometers (nominal $1.0 - 3.0\,\mu\text{m}$).
    pub core_radius_um: f64,
    /// Fluid flow radial velocity gradient parameter for sonic horizon in $\text{s}^{-1}$.
    pub horizon_flow_gradient_s_inv: f64,
}

impl Default for PolaritonVortexParams {
    fn default() -> Self {
        Self {
            winding_number: 1,
            core_radius_um: 1.5,
            horizon_flow_gradient_s_inv: 5.0e10,
        }
    }
}

/// Evaluated metrics for topological polariton vortices and sonic horizons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolaritonVortexMetrics {
    /// Quantized velocity circulation $\Gamma = \oint \mathbf{v} \cdot d\mathbf{r}$ in $\text{m}^2/\text{s}$.
    pub circulation_m2_s: f64,
    /// Dimensionless circulation quantum $\Gamma / (h / m^*) \approx \ell$.
    pub circulation_quantum_ratio: f64,
    /// Vortex core density depletion at $r = 0$ (relative to background $n_0$).
    pub core_depletion_fraction: f64,
    /// Acoustic sonic horizon radius in micrometers where $|v| = c_s$.
    pub sonic_horizon_radius_um: f64,
    /// Sonic Hawking temperature $T_H$ in Kelvin ($K$).
    pub hawking_temperature_k: f64,
}

impl PolaritonVortexParams {
    /// Creates new vortex parameters.
    pub fn new(winding_number: i32, core_radius_um: f64) -> Self {
        Self {
            winding_number: if winding_number == 0 {
                1
            } else {
                winding_number
            },
            core_radius_um: core_radius_um.max(0.1),
            horizon_flow_gradient_s_inv: 5.0e10,
        }
    }

    /// Evaluates radial density profile $n(r) = n_0 \frac{(r/\xi)^2}{1 + (r/\xi)^2}$.
    pub fn density_at_radius(&self, radius_um: f64, n0_um2: f64, xi_um: f64) -> f64 {
        let r_norm = radius_um / xi_um.max(1e-3);
        let factor = (r_norm * r_norm) / (1.0 + r_norm * r_norm);
        n0_um2 * factor
    }

    /// Evaluates tangential azimuthal velocity $v_\theta(r) = \ell \frac{\hbar}{m^* r}$ in $\text{m/s}$.
    pub fn tangential_velocity_m_s(&self, radius_um: f64, m_eff_kg: f64) -> f64 {
        let r_m = radius_um.max(0.01) * 1.0e-6;
        (self.winding_number as f64 * H_BAR) / (m_eff_kg * r_m)
    }

    /// Computes full vortex and acoustic horizon metrics.
    pub fn evaluate_vortex_metrics(
        &self,
        condensate_params: &PolaritonCondensateParams,
    ) -> PolaritonVortexMetrics {
        let m_eff = condensate_params.effective_mass_kg();
        let h_bar = H_BAR;
        let h_planck = 2.0 * PI * h_bar;

        // Circulation Gamma = 2*pi*r * v_theta = 2*pi * ell * hbar / m* = ell * h / m*
        let gamma = (self.winding_number as f64 * h_planck) / m_eff;
        let quantum_ratio = gamma / (h_planck / m_eff);

        let cs = condensate_params.sound_speed_m_s();
        // Sonic horizon where v_theta = cs => r_H = ell * hbar / (m* * cs)
        let r_horizon_m = (self.winding_number.abs() as f64 * h_bar) / (m_eff * cs.max(1.0));
        let r_horizon_um = r_horizon_m * 1.0e6;

        // Sonic Hawking temperature: T_H = hbar / (2*pi*k_B) * |dv/dr|_H
        let boltzmann_k = 1.380_649e-23;
        let surface_gravity = self.horizon_flow_gradient_s_inv;
        let t_hawking = (h_bar * surface_gravity) / (2.0 * PI * boltzmann_k);

        PolaritonVortexMetrics {
            circulation_m2_s: gamma,
            circulation_quantum_ratio: quantum_ratio,
            core_depletion_fraction: 1.0, // 100% density depletion at r=0
            sonic_horizon_radius_um: r_horizon_um,
            hawking_temperature_k: t_hawking,
        }
    }
}
