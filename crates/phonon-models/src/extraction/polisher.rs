#![deny(unsafe_code)]

//! Damped Gauss-Newton and Levenberg-Marquardt local parameter polishing engine.

use super::curve_data::{MeasuredCurve, MeasurementPoint};

/// Evaluation and convergence telemetry from a completed curve fitting or polishing execution.
#[derive(Debug, Clone, PartialEq)]
pub struct GenericFittingResult<T> {
    pub params: T,
    pub rmse: f64,
    pub r_squared: f64,
    pub iterations: usize,
    pub converged: bool,
}

/// Trait for compact device models optimizable via Levenberg-Marquardt local polishing.
pub trait PolishableModel: Clone + Copy {
    /// Number of continuous parameters in the model.
    fn num_params() -> usize;

    /// Converts physical parameters to normalized values in [0.0, 1.0].
    fn to_normalized(&self) -> Vec<f64>;

    /// Constructs physical parameters from normalized values in [0.0, 1.0].
    fn from_normalized(norm: &[f64]) -> Self;

    /// Evaluates predicted current at the given measurement bias point.
    fn evaluate_current(&self, pt: &MeasurementPoint, curve: &MeasuredCurve) -> f64;
}

/// Solves an N x N linear system A * x = b in-place using Gaussian elimination with partial pivoting.
/// Returns true if successful, or false if matrix is singular.
pub fn solve_linear_system(n: usize, a: &mut [f64], b: &mut [f64]) -> bool {
    for col in 0..n {
        let mut max_row = col;
        let mut max_val = a[col * n + col].abs();
        for row in (col + 1)..n {
            let val = a[row * n + col].abs();
            if val > max_val {
                max_val = val;
                max_row = row;
            }
        }
        if max_val < 1e-18 {
            return false;
        }
        if max_row != col {
            for c in 0..n {
                a.swap(col * n + c, max_row * n + c);
            }
            b.swap(col, max_row);
        }

        let pivot = a[col * n + col];
        for row in (col + 1)..n {
            let factor = a[row * n + col] / pivot;
            a[row * n + col] = 0.0;
            for c in (col + 1)..n {
                a[row * n + c] -= factor * a[col * n + c];
            }
            b[row] -= factor * b[col];
        }
    }

    for row in (0..n).rev() {
        let mut sum = b[row];
        for c in (row + 1)..n {
            sum -= a[row * n + c] * b[c];
        }
        let diag = a[row * n + row];
        if diag.abs() < 1e-18 {
            return false;
        }
        b[row] = sum / diag;
    }
    true
}

/// Polishes candidate parameters using damped Gauss-Newton / Levenberg-Marquardt local refinement.
pub fn polish_model_parameters<M: PolishableModel>(
    initial_params: &M,
    curve: &MeasuredCurve,
    max_iterations: usize,
) -> (M, f64, f64, usize, bool) {
    let n_points = curve.points.len();
    if n_points == 0 {
        return (*initial_params, 0.0, 1.0, 0, true);
    }

    let n_f64 = n_points as f64;
    let mut sum_meas = 0.0;
    for pt in &curve.points {
        sum_meas += pt.i_ds;
    }
    let mean_meas = sum_meas / n_f64;

    let mut ss_tot = 0.0;
    for pt in &curve.points {
        let dev = pt.i_ds - mean_meas;
        ss_tot += dev * dev;
    }

    let p = M::num_params();
    let mut curr_u = initial_params.to_normalized();
    if curr_u.len() != p {
        curr_u.resize(p, 0.5);
    }

    let eval_loss = |u: &[f64]| -> (f64, f64, f64) {
        let model = M::from_normalized(u);
        let mut ss_res = 0.0;
        for pt in &curve.points {
            let i_pred = model.evaluate_current(pt, curve);
            let err = pt.i_ds - i_pred;
            ss_res += err * err;
        }
        let rmse = (ss_res / n_f64).sqrt();
        let r2 = if ss_tot > 1e-30 {
            (1.0 - ss_res / ss_tot).clamp(-1.0, 1.0)
        } else {
            1.0
        };
        (ss_res, rmse, r2)
    };

    let (mut curr_ss_res, mut curr_rmse, mut curr_r2) = eval_loss(&curr_u);
    let mut lambda = 1e-3;
    let mut iters_done = 0;
    let delta_h = 1e-5;

    for iter in 0..max_iterations.max(15) {
        iters_done = iter + 1;

        if curr_rmse < 1e-12 || curr_r2 > 0.999999 {
            break;
        }

        let curr_model = M::from_normalized(&curr_u);
        let mut residuals = Vec::with_capacity(n_points);
        for pt in &curve.points {
            let i_pred = curr_model.evaluate_current(pt, curve);
            residuals.push(pt.i_ds - i_pred);
        }

        let mut j_mat = vec![0.0; n_points * p];
        for j in 0..p {
            let mut u_plus = curr_u.clone();
            let mut u_minus = curr_u.clone();
            u_plus[j] = (u_plus[j] + delta_h).clamp(0.0, 1.0);
            u_minus[j] = (u_minus[j] - delta_h).clamp(0.0, 1.0);
            let actual_2h = u_plus[j] - u_minus[j];

            if actual_2h.abs() > 1e-12 {
                let m_plus = M::from_normalized(&u_plus);
                let m_minus = M::from_normalized(&u_minus);
                for i in 0..n_points {
                    let f_plus = m_plus.evaluate_current(&curve.points[i], curve);
                    let f_minus = m_minus.evaluate_current(&curve.points[i], curve);
                    j_mat[i * p + j] = (f_plus - f_minus) / actual_2h;
                }
            }
        }

        let mut a_mat = vec![0.0; p * p];
        let mut b_vec = vec![0.0; p];
        for j in 0..p {
            let mut sum_b = 0.0;
            for i in 0..n_points {
                sum_b += j_mat[i * p + j] * residuals[i];
            }
            b_vec[j] = sum_b;

            for k in 0..p {
                let mut sum_a = 0.0;
                for i in 0..n_points {
                    sum_a += j_mat[i * p + j] * j_mat[i * p + k];
                }
                a_mat[j * p + k] = sum_a;
            }
        }

        let mut a_damped = a_mat.clone();
        for j in 0..p {
            let diag = a_mat[j * p + j];
            a_damped[j * p + j] = diag * (1.0 + lambda) + lambda * 1e-7 + 1e-12;
        }

        let mut delta_u = b_vec.clone();
        let solved = solve_linear_system(p, &mut a_damped, &mut delta_u);

        if solved {
            let mut cand_u = curr_u.clone();
            for j in 0..p {
                cand_u[j] = (cand_u[j] + delta_u[j]).clamp(0.0, 1.0);
            }

            let (cand_ss_res, cand_rmse, cand_r2) = eval_loss(&cand_u);

            if cand_ss_res < curr_ss_res {
                curr_u = cand_u;
                curr_ss_res = cand_ss_res;
                curr_rmse = cand_rmse;
                curr_r2 = cand_r2;
                lambda = (lambda * 0.2).max(1e-9);
                continue;
            }
        }

        lambda = (lambda * 5.0).min(1e9);

        // Fallback: gradient descent step
        let norm_b = b_vec.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm_b > 1e-15 {
            for step_scale in [0.01, 0.002, 0.0005] {
                let mut grad_u = curr_u.clone();
                for j in 0..p {
                    grad_u[j] = (grad_u[j] + step_scale * (b_vec[j] / norm_b)).clamp(0.0, 1.0);
                }
                let (grad_ss, grad_rmse, grad_r2) = eval_loss(&grad_u);
                if grad_ss < curr_ss_res {
                    curr_u = grad_u;
                    curr_ss_res = grad_ss;
                    curr_rmse = grad_rmse;
                    curr_r2 = grad_r2;
                    break;
                }
            }
        }
    }

    let final_model = M::from_normalized(&curr_u);
    let converged = curr_r2 >= 0.98 || curr_rmse < 1e-4;
    (final_model, curr_rmse, curr_r2, iters_done, converged)
}

/// Generic Levenberg-Marquardt local refinement for any model implementing `PolishableModel`.
pub fn polish_parameters<M: PolishableModel>(
    params: &M,
    curve: &MeasuredCurve,
) -> GenericFittingResult<M> {
    let (refined_params, rmse, r_squared, iters, converged) =
        polish_model_parameters(params, curve, 30);
    GenericFittingResult {
        params: refined_params,
        rmse,
        r_squared,
        iterations: iters,
        converged,
    }
}
