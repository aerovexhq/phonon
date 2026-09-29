//! Autonomous solver for topological polariton vortices,
//! quantized circulation, and acoustic black hole horizons.

use phonon_models::polariton_condensate::{
    PolaritonCondensateParams, PolaritonVortexMetrics, PolaritonVortexParams,
};

/// Solver for topological polariton vortices and sonic horizons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolaritonVortexSolver {
    pub condensate_params: PolaritonCondensateParams,
    pub vortex_params: PolaritonVortexParams,
}

impl PolaritonVortexSolver {
    /// Creates a new polariton vortex solver.
    pub fn new(
        condensate_params: PolaritonCondensateParams,
        vortex_params: PolaritonVortexParams,
    ) -> Self {
        Self {
            condensate_params,
            vortex_params,
        }
    }

    /// Solves quantized circulation and vortex metrics.
    pub fn solve_vortex_metrics(&self) -> PolaritonVortexMetrics {
        self.vortex_params
            .evaluate_vortex_metrics(&self.condensate_params)
    }

    /// Solves radial density profile $n(r)$ across given radii in $\mu\text{m}$.
    pub fn solve_radial_profile(&self, radii_um: &[f64]) -> Vec<(f64, f64)> {
        let n0 = self.condensate_params.condensate_density_um2();
        let xi = self.condensate_params.healing_length_um();
        radii_um
            .iter()
            .map(|&r| (r, self.vortex_params.density_at_radius(r, n0, xi)))
            .collect()
    }
}
