//! Electronic nematic fluctuations, elastoresistance coefficients,
//! and anisotropic superconducting gap structure.

/// Parameters for electronic nematic order parameter fluctuations in FeSe.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NematicOrderParams {
    /// Nematic order parameter $\psi_{\mathrm{nem}} = \frac{n_{d_{xz}} - n_{d_{yz}}}{n_{d_{xz}} + n_{d_{yz}}} \in [-1.0, 1.0]$.
    pub nematic_order: f64,
    /// Nematic susceptibility prefactor $\chi_0$ in Kelvin.
    pub susceptibility_chi0_k: f64,
    /// Electronic nematic transition temperature $T_{\mathrm{nem}}$ in Kelvin.
    pub nematic_transition_temp_k: f64,
    /// Fluctuation cutoff temperature $\theta_{\mathrm{fluc}}$ in Kelvin.
    pub fluctuation_cutoff_k: f64,
    /// Gap anisotropy coupling coefficient $\eta_{\mathrm{nem}}$ (nominal $0.35$).
    pub gap_anisotropy_eta: f64,
}

impl Default for NematicOrderParams {
    fn default() -> Self {
        Self {
            nematic_order: 0.40,
            susceptibility_chi0_k: 2500.0,
            nematic_transition_temp_k: 70.0,
            fluctuation_cutoff_k: 15.0,
            gap_anisotropy_eta: 0.35,
        }
    }
}

impl NematicOrderParams {
    /// Creates new nematic fluctuation parameters.
    pub fn new(
        nematic_order: f64,
        nematic_transition_temp_k: f64,
        gap_anisotropy_eta: f64,
    ) -> Self {
        Self {
            nematic_order: nematic_order.clamp(-1.0, 1.0),
            susceptibility_chi0_k: 2500.0,
            nematic_transition_temp_k: nematic_transition_temp_k.max(1.0),
            fluctuation_cutoff_k: 15.0,
            gap_anisotropy_eta: gap_anisotropy_eta.clamp(0.0, 0.9),
        }
    }

    /// Evaluates the elastoresistance coefficient $2m_{66}(T) = \frac{d(\Delta R / R)}{d \epsilon_{[110]}}$:
    /// Follows Curie-Weiss divergence:
    /// $$2m_{66}(T) = \frac{\chi_0}{T - T_{\mathrm{nem}} + \theta_{\mathrm{fluc}}}$$
    pub fn elastoresistance_2m66(&self, temperature_k: f64) -> f64 {
        let denom =
            (temperature_k - self.nematic_transition_temp_k).abs() + self.fluctuation_cutoff_k;
        self.susceptibility_chi0_k / denom
    }

    /// Evaluates the angle-dependent anisotropic superconducting gap $\Delta(\theta)$ in $\text{meV}$:
    /// $$\Delta(\theta) = \Delta_0 \left[ 1 + \eta_{\mathrm{nem}} \psi_{\mathrm{nem}} \cos(2\theta) \right]$$
    pub fn anisotropic_gap_mev(&self, base_gap_mev: f64, angle_rad: f64) -> f64 {
        let modulation =
            1.0 + self.gap_anisotropy_eta * self.nematic_order * (2.0 * angle_rad).cos();
        base_gap_mev * modulation.max(0.01)
    }

    /// Gap anisotropy ratio $\Delta_{\mathrm{max}} / \Delta_{\mathrm{min}}$:
    /// $$\frac{\Delta_{\mathrm{max}}}{\Delta_{\mathrm{min}}} = \frac{1 + |\eta_{\mathrm{nem}} \psi_{\mathrm{nem}}|}{1 - |\eta_{\mathrm{nem}} \psi_{\mathrm{nem}}|}$$
    pub fn gap_anisotropy_ratio(&self) -> f64 {
        let val = (self.gap_anisotropy_eta * self.nematic_order).abs();
        (1.0 + val) / (1.0 - val).max(1e-4)
    }
}
