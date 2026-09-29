//! Autonomous solver for relativistic Néel vector domain wall dynamics,
//! sub-picosecond synaptic memristor plasticity, and non-reciprocal magnon diodes.

use phonon_models::afm_spintronics::{AfmMaterialParams, NeelDomainWallParams, NeelTransitMetrics};

/// Solver for relativistic Néel domain wall dynamics and memristive synapses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NeelDomainWallSolver {
    pub afm_params: AfmMaterialParams,
    pub dw_params: NeelDomainWallParams,
}

impl NeelDomainWallSolver {
    /// Creates a new Néel domain wall solver.
    pub fn new(afm_params: AfmMaterialParams, dw_params: NeelDomainWallParams) -> Self {
        Self {
            afm_params,
            dw_params,
        }
    }

    /// Solves steady-state relativistic domain wall velocity in $\text{m/s}$.
    pub fn solve_dw_velocity(&self, current_density_a_m2: f64) -> f64 {
        self.dw_params
            .evaluate_dw_velocity(&self.afm_params, current_density_a_m2)
    }

    /// Solves the domain wall transit time across the nanogap synaptic track in picoseconds ($\text{ps}$).
    pub fn solve_transit_time_ps(&self, current_density_a_m2: f64) -> f64 {
        let metrics = self
            .dw_params
            .evaluate_transit(&self.afm_params, current_density_a_m2, 0.5);
        metrics.transit_time_ps
    }

    /// Evaluates synaptic plastic weight update following a programming current pulse:
    /// Returns the updated normalized position $x \in [0, 1]$ and full transit metrics.
    pub fn apply_programming_pulse(
        &self,
        current_density_a_m2: f64,
        pulse_duration_ps: f64,
        initial_normalized_pos: f64,
    ) -> (f64, NeelTransitMetrics) {
        let v_dw = self.solve_dw_velocity(current_density_a_m2);
        let pulse_s = pulse_duration_ps * 1.0e-12;
        let delta_x_m = v_dw * pulse_s * current_density_a_m2.signum();

        let current_x_m = initial_normalized_pos * self.dw_params.track_length_m;
        let new_x_m = (current_x_m + delta_x_m).clamp(0.0, self.dw_params.track_length_m);
        let new_pos = new_x_m / self.dw_params.track_length_m;

        let metrics =
            self.dw_params
                .evaluate_transit(&self.afm_params, current_density_a_m2, new_pos);
        (new_pos, metrics)
    }

    /// Solves the non-reciprocal magnon diode rectification ratio in decibels.
    pub fn solve_magnon_diode_rectification(&self) -> f64 {
        let metrics = self
            .dw_params
            .evaluate_transit(&self.afm_params, 1.0e11, 0.5);
        metrics.diode_rectification_db
    }
}
