#![deny(unsafe_code)]

//! Real-time Milne device estimating local truncation error and adapting integration step sizes
//! via high-order / low-order symplectic pairs.

use crate::symplectic_multirate::integrator::{SymplecticIntegrator, MAX_STATE_DIM};

/// Decision outcome for an adaptive step attempt.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StepDecision {
    /// True if the error is within bounds and the step is accepted.
    pub accepted: bool,
    /// Next proposed step size in seconds.
    pub next_dt: f64,
    /// Multiplicative adaptation factor s = (0.9 / err)^{1/5} clamped to [0.5, 2.0].
    pub adaptation_factor: f64,
    /// Normalized local truncation error.
    pub error_norm: f64,
}

/// Real-time Milne adaptive step size controller.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MilneAdaptiveController {
    /// Desired local error tolerance.
    pub tolerance: f64,
    /// Lower bound on integration time step in seconds.
    pub min_dt: f64,
    /// Upper bound on integration time step in seconds.
    pub max_dt: f64,
    /// Currently active integration time step in seconds.
    pub current_dt: f64,
    /// Cumulative count of accepted steps.
    pub accepted_steps: u64,
    /// Cumulative count of rejected steps requiring sub-stepping retry.
    pub rejected_steps: u64,
    /// Most recently evaluated error norm.
    pub last_error_norm: f64,
    /// Peak observed error norm across accepted/rejected steps.
    pub max_error_norm: f64,
}

impl Default for MilneAdaptiveController {
    fn default() -> Self {
        Self::new(1.0e-6, 1.0e-9, 1.0e-4, 1.0e-6)
    }
}

impl MilneAdaptiveController {
    /// Constructs a new Milne adaptive controller with user-specified bounds.
    pub fn new(tolerance: f64, min_dt: f64, max_dt: f64, initial_dt: f64) -> Self {
        let tol = tolerance.clamp(1.0e-12, 1.0e-2);
        let min_s = min_dt.max(1.0e-12);
        let max_s = max_dt.max(min_s);
        let init_s = initial_dt.clamp(min_s, max_s);
        Self {
            tolerance: tol,
            min_dt: min_s,
            max_dt: max_s,
            current_dt: init_s,
            accepted_steps: 0,
            rejected_steps: 0,
            last_error_norm: 0.0,
            max_error_norm: 0.0,
        }
    }

    /// Evaluates the normalized Milne error:
    /// err = ||x_{GLRK4} - x_{Verlet}|| / (tol * (1 + ||x||)).
    pub fn compute_error_norm(&self, x_glrk4: &[f64], x_verlet: &[f64]) -> f64 {
        let d = x_glrk4.len();
        let mut diff_sq = 0.0;
        let mut x_sq = 0.0;
        for i in 0..d {
            let diff = x_glrk4[i] - x_verlet[i];
            diff_sq += diff * diff;
            x_sq += x_glrk4[i] * x_glrk4[i];
        }
        let diff_norm = diff_sq.sqrt();
        let x_norm = x_sq.sqrt();
        diff_norm / (self.tolerance * (1.0 + x_norm))
    }

    /// Computes the adaptation scale factor s = (0.9 / err)^{1/5}, clamped to [0.5, 2.0].
    pub fn adaptation_factor(&self, err: f64) -> f64 {
        if err < 1.0e-15 {
            2.0
        } else {
            (0.9 / err).powf(0.2).clamp(0.5, 2.0)
        }
    }

    /// Evaluates a candidate step decision based on the Milne error norm.
    pub fn evaluate_step(&mut self, err: f64) -> StepDecision {
        self.last_error_norm = err;
        self.max_error_norm = self.max_error_norm.max(err);

        if err <= 1.0 {
            // Step accepted
            self.accepted_steps += 1;
            let s = self.adaptation_factor(err);
            let next_dt = (self.current_dt * s).clamp(self.min_dt, self.max_dt);
            let dec = StepDecision {
                accepted: true,
                next_dt,
                adaptation_factor: s,
                error_norm: err,
            };
            self.current_dt = next_dt;
            dec
        } else {
            // Step rejected, halving step size
            self.rejected_steps += 1;
            let s = 0.5;
            let next_dt = (self.current_dt * 0.5).clamp(self.min_dt, self.max_dt);
            let dec = StepDecision {
                accepted: false,
                next_dt,
                adaptation_factor: s,
                error_norm: err,
            };
            self.current_dt = next_dt;
            dec
        }
    }

    /// Executes an adaptive step on state vector x at time t.
    /// Retries with halved step size until accepted or min_dt is reached.
    pub fn step<F>(
        &mut self,
        x: &mut [f64],
        t: &mut f64,
        integrator: &SymplecticIntegrator,
        mut f: F,
    ) -> bool
    where
        F: FnMut(&[f64], f64, &mut [f64]),
    {
        let d = x.len();
        if d == 0 || d > MAX_STATE_DIM {
            return false;
        }

        let mut x_glrk = [0.0; MAX_STATE_DIM];
        let mut x_verlet = [0.0; MAX_STATE_DIM];

        const MAX_RETRIES: usize = 16;
        for _ in 0..MAX_RETRIES {
            let dt = self.current_dt;

            // Attempt GLRK4 4th-order solution
            let ok_glrk = integrator.step_glrk4(x, *t, dt, &mut x_glrk[..d], &mut f);
            if !ok_glrk {
                self.current_dt = (self.current_dt * 0.5).clamp(self.min_dt, self.max_dt);
                self.rejected_steps += 1;
                continue;
            }

            // Attempt Implicit Midpoint 2nd-order predictor
            let ok_verlet = integrator.step_implicit_midpoint(x, *t, dt, &mut x_verlet[..d], &mut f);
            if !ok_verlet {
                self.current_dt = (self.current_dt * 0.5).clamp(self.min_dt, self.max_dt);
                self.rejected_steps += 1;
                continue;
            }

            let err = self.compute_error_norm(&x_glrk[..d], &x_verlet[..d]);
            let decision = self.evaluate_step(err);

            if decision.accepted {
                // Apply update
                for i in 0..d {
                    x[i] = x_glrk[i];
                }
                *t += dt;
                return true;
            }

            if dt <= self.min_dt {
                // Cannot reduce further, accept at minimal step
                for i in 0..d {
                    x[i] = x_glrk[i];
                }
                *t += dt;
                return true;
            }
        }

        false
    }
}
