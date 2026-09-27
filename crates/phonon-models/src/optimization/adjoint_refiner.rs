//! Adjoint gradient sensitivity solver and projected gradient local refiner.
//!
//! Provides fast local gradient-based fine-tuning of continuous design parameters
//! ($L_g, T_{ch}, W_{ch}, \text{EOT}, \Phi_m$) along the Pareto frontier.

use super::genome::{GeneBounds, TransistorGenome};
use super::physical_fitness::evaluate_transistor_fitness;

/// Direction / objective for local gradient refinement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptimizationTarget {
    /// Maximize on-off switching ratio $\ln(I_{on} / I_{off})$.
    MaximizeIonIoff,
    /// Minimize intrinsic switching delay $\tau$.
    MinimizeDelay,
    /// Maximize drive current $I_{on}$.
    MaximizeDriveCurrent,
    /// Minimize energy-delay product (EDP).
    MinimizeEdp,
}

/// Gradient sensitivity vector with respect to continuous geometric parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParameterSensitivities {
    pub d_j_d_lg: f64,
    pub d_j_d_tch: f64,
    pub d_j_d_wch: f64,
    pub d_j_d_eot: f64,
    pub d_j_d_phim: f64,
}

/// Local continuous parameter refiner using adjoint / finite-difference gradient projections.
#[derive(Debug, Clone)]
pub struct AdjointRefiner {
    pub bounds: GeneBounds,
    pub step_size: f64,
    pub max_steps: usize,
}

impl Default for AdjointRefiner {
    fn default() -> Self {
        Self {
            bounds: GeneBounds::default(),
            step_size: 0.05, // 5% normalized step size
            max_steps: 10,
        }
    }
}

impl AdjointRefiner {
    pub fn new(bounds: GeneBounds, max_steps: usize) -> Self {
        Self {
            bounds,
            step_size: 0.05,
            max_steps,
        }
    }

    /// Evaluates the scalar objective merit value for a given target.
    fn evaluate_target_merit(genome: &TransistorGenome, target: OptimizationTarget) -> f64 {
        let fit = evaluate_transistor_fitness(genome);
        match target {
            OptimizationTarget::MaximizeIonIoff => {
                if fit.ion_ioff_ratio > 1.0 {
                    fit.ion_ioff_ratio.ln()
                } else {
                    -10.0
                }
            }
            OptimizationTarget::MinimizeDelay => {
                -fit.intrinsic_delay_ps // Negative so maximizing merit minimizes delay
            }
            OptimizationTarget::MaximizeDriveCurrent => fit.i_on_a * 1.0e3, // In mA
            OptimizationTarget::MinimizeEdp => {
                -(fit.energy_delay_product_js * 1.0e28) // Negative so maximizing merit minimizes EDP
            }
        }
    }

    /// Computes the sensitivity gradient $\nabla \mathcal{J}$ via central finite differences.
    pub fn compute_sensitivities(
        &self,
        genome: &TransistorGenome,
        target: OptimizationTarget,
    ) -> ParameterSensitivities {
        let delta = 1.0e-3;

        // dJ / dLg
        let mut g_plus = genome.clone();
        g_plus.gate_length_nm += delta;
        let mut g_minus = genome.clone();
        g_minus.gate_length_nm -= delta;
        let d_lg = (Self::evaluate_target_merit(&g_plus, target)
            - Self::evaluate_target_merit(&g_minus, target))
            / (2.0 * delta);

        // dJ / dTch
        let mut g_plus = genome.clone();
        g_plus.channel_thickness_nm += delta;
        let mut g_minus = genome.clone();
        g_minus.channel_thickness_nm -= delta;
        let d_tch = if genome.channel_material.is_2d_material() {
            0.0 // Monolayer thickness is fixed
        } else {
            (Self::evaluate_target_merit(&g_plus, target)
                - Self::evaluate_target_merit(&g_minus, target))
                / (2.0 * delta)
        };

        // dJ / dWch
        let mut g_plus = genome.clone();
        g_plus.channel_width_nm += delta;
        let mut g_minus = genome.clone();
        g_minus.channel_width_nm -= delta;
        let d_wch = (Self::evaluate_target_merit(&g_plus, target)
            - Self::evaluate_target_merit(&g_minus, target))
            / (2.0 * delta);

        // dJ / dEOT
        let mut g_plus = genome.clone();
        g_plus.eot_nm += delta;
        let mut g_minus = genome.clone();
        g_minus.eot_nm -= delta;
        let d_eot = (Self::evaluate_target_merit(&g_plus, target)
            - Self::evaluate_target_merit(&g_minus, target))
            / (2.0 * delta);

        // dJ / dPhiM
        let mut g_plus = genome.clone();
        g_plus.workfunction_ev += delta;
        let mut g_minus = genome.clone();
        g_minus.workfunction_ev -= delta;
        let d_phim = (Self::evaluate_target_merit(&g_plus, target)
            - Self::evaluate_target_merit(&g_minus, target))
            / (2.0 * delta);

        ParameterSensitivities {
            d_j_d_lg: d_lg,
            d_j_d_tch: d_tch,
            d_j_d_wch: d_wch,
            d_j_d_eot: d_eot,
            d_j_d_phim: d_phim,
        }
    }

    /// Refines a candidate genome using projected gradient ascent towards the target merit.
    pub fn refine_candidate(
        &self,
        candidate: &TransistorGenome,
        target: OptimizationTarget,
    ) -> TransistorGenome {
        let mut current = candidate.clone();
        let mut best_merit = Self::evaluate_target_merit(&current, target);

        for _ in 0..self.max_steps {
            let grad = self.compute_sensitivities(&current, target);

            // Compute gradient magnitude for normalization
            let norm = (grad.d_j_d_lg.powi(2)
                + grad.d_j_d_tch.powi(2)
                + grad.d_j_d_wch.powi(2)
                + grad.d_j_d_eot.powi(2)
                + grad.d_j_d_phim.powi(2))
            .sqrt();

            if norm < 1e-9 {
                break; // Local extremum reached
            }

            let mut step_candidate = current.clone();
            let step = self.step_size;

            step_candidate.gate_length_nm += (grad.d_j_d_lg / norm) * step * 2.0;
            step_candidate.channel_thickness_nm += (grad.d_j_d_tch / norm) * step * 0.5;
            step_candidate.channel_width_nm += (grad.d_j_d_wch / norm) * step * 2.0;
            step_candidate.eot_nm += (grad.d_j_d_eot / norm) * step * 0.2;
            step_candidate.workfunction_ev += (grad.d_j_d_phim / norm) * step * 0.1;

            step_candidate.clamp_to_bounds(&self.bounds);
            let candidate_merit = Self::evaluate_target_merit(&step_candidate, target);

            if candidate_merit > best_merit {
                best_merit = candidate_merit;
                current = step_candidate;
            } else {
                // Diminish step size or terminate
                break;
            }
        }

        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adjoint_gradient_sensitivities() {
        let genome = TransistorGenome::n2_gaa_nanosheet_preset();
        let refiner = AdjointRefiner::default();

        let sens = refiner.compute_sensitivities(&genome, OptimizationTarget::MaximizeIonIoff);
        // Increasing EOT degrades electrostatic control, so dJ/dEOT must be negative
        assert!(sens.d_j_d_eot < 0.0);
    }

    #[test]
    fn test_local_refinement_improves_merit() {
        let genome = TransistorGenome::n2_gaa_nanosheet_preset();
        let refiner = AdjointRefiner::new(GeneBounds::default(), 5);

        let initial_fit = evaluate_transistor_fitness(&genome);
        let refined = refiner.refine_candidate(&genome, OptimizationTarget::MaximizeDriveCurrent);
        let refined_fit = evaluate_transistor_fitness(&refined);

        // Drive current should be equal or improved after local refinement
        assert!(refined_fit.i_on_a >= initial_fit.i_on_a);
    }
}
