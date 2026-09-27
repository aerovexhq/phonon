//! Boundary conditions for spatial thermal diffusion and heat transfer.

/// Stefan-Boltzmann radiation constant $\sigma_{SB}$ in $\text{W} / (\text{m}^2 \cdot \text{K}^4)$.
pub const STEFAN_BOLTZMANN: f64 = 5.670_374_419e-8;

/// Physical boundary condition applied to surfaces or boundaries of a thermal domain.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThermalBoundary {
    /// Dirichlet boundary condition: fixed surface temperature (e.g., ideal infinite heatsink).
    Dirichlet { temperature_kelvin: f64 },

    /// Neumann boundary condition: prescribed heat flux normal to the surface in $\text{W} / \text{m}^2$.
    /// A value of `0.0` represents an ideal adiabatic / thermally insulated boundary.
    Neumann { heat_flux_w_m2: f64 },

    /// Robin boundary condition: combined convective and radiative cooling to ambient environment.
    Robin {
        ambient_kelvin: f64,
        /// Convective heat transfer coefficient $h_{conv}$ in $\text{W} / (\text{m}^2 \cdot \text{K})$.
        h_conv: f64,
        /// Surface emissivity $\epsilon \in [0.0, 1.0]$ for Stefan-Boltzmann radiation.
        emissivity: f64,
    },
}

impl Default for ThermalBoundary {
    fn default() -> Self {
        Self::Neumann {
            heat_flux_w_m2: 0.0,
        }
    }
}

impl ThermalBoundary {
    /// Computes the outward heat flux $q_{out}$ ($\text{W}/\text{m}^2$) and its temperature
    /// derivative $\frac{\partial q_{out}}{\partial T}$ ($\text{W}/(\text{m}^2 \cdot \text{K})$)
    /// at surface temperature $T$.
    #[inline]
    pub fn evaluate_flux(&self, surface_temp_k: f64) -> (f64, f64) {
        match *self {
            ThermalBoundary::Dirichlet { .. } => {
                // Dirichlet does not prescribe flux; it enforces temperature directly.
                (0.0, 0.0)
            }
            ThermalBoundary::Neumann { heat_flux_w_m2 } => (heat_flux_w_m2, 0.0),
            ThermalBoundary::Robin {
                ambient_kelvin,
                h_conv,
                emissivity,
            } => {
                let delta_t = surface_temp_k - ambient_kelvin;
                let flux_conv = h_conv * delta_t;
                let dflux_conv = h_conv;

                let flux_rad = if emissivity > 0.0 {
                    let t4 = surface_temp_k.powi(4);
                    let tamb4 = ambient_kelvin.powi(4);
                    emissivity * STEFAN_BOLTZMANN * (t4 - tamb4)
                } else {
                    0.0
                };

                let dflux_rad = if emissivity > 0.0 {
                    4.0 * emissivity * STEFAN_BOLTZMANN * surface_temp_k.powi(3)
                } else {
                    0.0
                };

                (flux_conv + flux_rad, dflux_conv + dflux_rad)
            }
        }
    }
}
