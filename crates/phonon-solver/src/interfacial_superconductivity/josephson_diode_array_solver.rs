//! Non-reciprocal Josephson diode array solver for finite-momentum supercurrents
//! and non-linear current-phase relations.

use phonon_models::interfacial_superconductivity::{JosephsonDiodeMetrics, JosephsonDiodeParams};
use std::f64::consts::PI;

/// Multi-junction non-reciprocal Josephson diode array solver.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JosephsonDiodeArraySolver {
    pub params: JosephsonDiodeParams,
}

impl JosephsonDiodeArraySolver {
    /// Creates a new Josephson diode array solver.
    pub fn new(params: JosephsonDiodeParams) -> Self {
        Self { params }
    }

    /// Solves the forward and reverse critical currents, diode efficiency, and rectification ratio.
    pub fn solve_diode_metrics(&self) -> JosephsonDiodeMetrics {
        self.params.evaluate_critical_currents()
    }

    /// Discretizes the non-linear current-phase relation $I_s(\phi)$ across $[0, 2\pi]$.
    pub fn solve_current_phase_relation(&self, num_points: usize) -> Vec<(f64, f64)> {
        let n = num_points.max(16);
        (0..n)
            .map(|i| {
                let phi = (i as f64) * 2.0 * PI / (n as f64);
                let is = self.params.supercurrent_at_phase(phi);
                (phi, is)
            })
            .collect()
    }
}
