//! Adaptive step-size controller and Local Truncation Error (LTE) estimator for TR-BDF2.

use super::integrator::TR_BDF2_GAMMA;

/// Configuration options for the adaptive time-step controller.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StepControlOptions {
    /// Relative tolerance for local truncation error (typically 1e-3).
    pub reltol: f64,
    /// Absolute tolerance for voltages (typically 1e-6 V).
    pub vntol: f64,
    /// Absolute tolerance for currents (typically 1e-12 A).
    pub abstol: f64,
    /// Minimum allowed time step (seconds).
    pub min_step: f64,
    /// Maximum allowed time step (seconds).
    pub max_step: f64,
    /// Maximum step expansion ratio (typically 2.0).
    pub max_growth: f64,
    /// Maximum step shrinkage ratio on rejection (typically 0.25).
    pub min_shrink: f64,
    /// Safety factor for step prediction (typically 0.85 to 0.9).
    pub safety_factor: f64,
}

impl Default for StepControlOptions {
    fn default() -> Self {
        Self {
            reltol: 1e-3,
            vntol: 1e-6,
            abstol: 1e-12,
            min_step: 1e-15,
            max_step: 1e-3,
            max_growth: 2.0,
            min_shrink: 0.25,
            safety_factor: 0.9,
        }
    }
}

/// Evaluates the normalized Local Truncation Error (LTE) and computes the recommended next step size.
pub fn evaluate_tr_bdf2_lte(
    x_n: &[f64],
    x_gamma: &[f64],
    x_next: &[f64],
    h: f64,
    options: &StepControlOptions,
) -> (f64, bool, f64) {
    let gamma = TR_BDF2_GAMMA;
    let factor = 2.0 * (1.0 - 2.0 * gamma + gamma * gamma) / ((2.0 - gamma) * (1.0 - gamma));

    let mut sum_sq = 0.0;
    let n = x_next.len();

    if n == 0 {
        return (0.0, true, (h * options.max_growth).min(options.max_step));
    }

    for i in 0..n {
        // Difference between 2nd-order predictor and BDF2 corrector
        let d2 = x_next[i] - (1.0 / gamma) * x_gamma[i] + ((1.0 - gamma) / gamma) * x_n[i];
        let lte_i = (factor * d2).abs();

        let tol_i = options.reltol * x_next[i].abs() + options.vntol;
        let ratio = lte_i / tol_i.max(1e-18);
        sum_sq += ratio * ratio;
    }

    let error_norm = (sum_sq / n as f64).sqrt();
    let accepted = error_norm <= 1.0 || h <= options.min_step;

    let growth = if error_norm > 1e-12 {
        options.safety_factor * (1.0 / error_norm).powf(1.0 / 3.0)
    } else {
        options.max_growth
    };

    let next_h = if accepted {
        let clamped_growth = growth.clamp(0.5, options.max_growth);
        (h * clamped_growth).clamp(options.min_step, options.max_step)
    } else {
        let clamped_shrink = growth.clamp(options.min_shrink, 0.9);
        (h * clamped_shrink).max(options.min_step)
    };

    (error_norm, accepted, next_h)
}
