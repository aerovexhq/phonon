//! Multi-physics solver for non-Hermitian skin effect, non-Bloch band theory,
//! Generalized Brillouin Zone (GBZ) contours, and higher-order exceptional points.

use phonon_models::non_hermitian_topo::{NonHermitianSkinMetrics, NonHermitianSkinParams};
use std::f64::consts::PI;

/// Multi-physics solver for non-Hermitian skin mode localization and non-Bloch topology.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonBlochSkinSolver {
    pub params: NonHermitianSkinParams,
}

impl NonBlochSkinSolver {
    /// Creates a new non-Bloch skin solver.
    pub fn new(params: NonHermitianSkinParams) -> Self {
        Self { params }
    }

    /// Evaluates non-Hermitian skin metrics and exceptional point characteristics.
    pub fn solve_skin_metrics(&self) -> NonHermitianSkinMetrics {
        self.params.evaluate_skin_metrics()
    }

    /// Solves the spatial probability density distribution $|\psi(j)|^2$ of the skin boundary mode:
    /// Normalized such that $\sum_{j=0}^{N-1} |\psi(j)|^2 = 1.0$.
    pub fn solve_skin_wavefunction(&self) -> Vec<(usize, f64)> {
        let n = self.params.lattice_sites_count;
        let metrics = self.solve_skin_metrics();
        let xi = metrics.skin_depth_unit_cells.max(0.1);

        let mut unnorm = Vec::with_capacity(n);
        let mut sum_prob = 0.0;
        for j in 0..n {
            // Localization toward right boundary j = n - 1:
            let dist_from_right = (n - 1 - j) as f64;
            let prob = (-2.0 * dist_from_right / xi).exp();
            sum_prob += prob;
            unnorm.push((j, prob));
        }

        let inv_sum = 1.0 / sum_prob.max(1e-12);
        unnorm.into_iter().map(|(j, p)| (j, p * inv_sum)).collect()
    }

    /// Solves the Generalized Brillouin Zone (GBZ) contour $beta(    heta) = r_{\mathrm{GBZ}} e^{i    heta}$:
    pub fn solve_gbz_contour(&self, num_points: usize) -> Vec<(f64, f64)> {
        let n = num_points.max(8);
        let metrics = self.solve_skin_metrics();
        let r = metrics.gbz_radius;

        let mut contour = Vec::with_capacity(n);
        for i in 0..n {
            let theta = 2.0 * PI * (i as f64 / n as f64);
            let re = r * theta.cos();
            let im = r * theta.sin();
            contour.push((re, im));
        }
        contour
    }

    /// Evaluates the complex eigenvalue energy splitting $\Delta E$ near an $N_{\mathrm{EP}}$-th order exceptional point:
    /// $$\Delta E = (\epsilon)^{1 / N_{\mathrm{EP}}}$$
    pub fn solve_exceptional_point_splitting(&self, perturbation: f64) -> f64 {
        let metrics = self.solve_skin_metrics();
        let order = metrics.exceptional_point_order as f64;
        let eps = perturbation.abs().max(1.0e-12);
        eps.powf(1.0 / order)
    }
}
