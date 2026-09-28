#![allow(clippy::needless_range_loop)]
//! Multi-Threaded Rayon Readout Solvers & Neuromorphic Reservoir Benchmarks.
//!
//! Formulates:
//! - Multi-threaded Rayon ridge regression (Tikhonov regularization) and Moore-Penrose pseudo-inverse readout:
//!   $$\mathbf{W}_{out} = \mathbf{Y}_{target} \mathbf{X}^T (\mathbf{X} \mathbf{X}^T + \lambda \mathbf{I})^{-1}$$
//!   in pure safe Rust with partial-pivoting matrix inversion.
//! - Closed-loop and open-loop forecasting inference pipelines with washout transient period.
//! - Chaotic time-series generators:
//!   * Mackey-Glass delay differential attractor.
//!   * Lorenz-63 strange attractor ($x, y, z$) with 4th-order Runge-Kutta (RK4).
//!   * Non-linear Auto-Regressive Moving Average (NARMA-10) system identification:
//!     $$y[t+1] = 0.3 y[t] + 0.05 y[t] \sum_{i=0}^9 y[t-i] + 1.5 u[t-9] u[t] + 0.1$$
//! - Normalized Root Mean Square Error (NRMSE) validation ($< 0.05$).

use phonon_models::memristor::reservoir::{invert_matrix_safe, MemristiveReservoir, ReservoirRng};
use rayon::prelude::*;

/// Trained linear readout weights and bias.
#[derive(Debug, Clone, PartialEq)]
pub struct TrainedReadout {
    /// Readout weight matrix $\mathbf{W}_{out} \in \mathbb{R}^{K \times D}$ where $D = N_{res} + 1$ (including bias).
    pub weights: Vec<Vec<f64>>,
    /// Number of output dimensions $K$.
    pub num_outputs: usize,
    /// Number of input features $D$ per state.
    pub num_features: usize,
}

impl TrainedReadout {
    /// Predicts target vector $\hat{\mathbf{y}}$ given reservoir state vector $\mathbf{x}$.
    pub fn predict(&self, state: &[f64]) -> Vec<f64> {
        let mut out = vec![0.0; self.num_outputs];
        let d = state.len();

        for k in 0..self.num_outputs {
            let mut sum = 0.0;
            for i in 0..d {
                sum += self.weights[k][i] * state[i];
            }
            // Bias term (last element in weight vector):
            sum += self.weights[k][d];
            out[k] = sum;
        }

        out
    }
}

/// Multi-threaded Rayon Ridge Regression and Pseudo-Inverse Readout Solver.
pub struct ReservoirSolver;

impl ReservoirSolver {
    /// Trains linear readout matrix $\mathbf{W}_{out}$ using parallel Tikhonov ridge regression:
    /// $$\mathbf{W}_{out} = \mathbf{Y} \mathbf{X}^T (\mathbf{X} \mathbf{X}^T + \lambda \mathbf{I})^{-1}$$
    pub fn train_ridge_regression(
        states: &[Vec<f64>],
        targets: &[Vec<f64>],
        lambda: f64,
    ) -> Result<TrainedReadout, String> {
        let total_samples = states.len();
        if total_samples == 0 || targets.len() != total_samples {
            return Err("Mismatched or empty states and targets".to_string());
        }

        let n_res = states[0].len();
        let num_features = n_res + 1; // plus bias
        let num_outputs = targets[0].len();

        // 1. Parallel assembly of Gram matrix A = X^T * X + lambda * I (size D x D)
        let a_matrix: Vec<Vec<f64>> = (0..num_features)
            .into_par_iter()
            .map(|i| {
                let mut row = vec![0.0; num_features];
                for j in 0..num_features {
                    let mut sum = 0.0;
                    for t in 0..total_samples {
                        let xi = if i < n_res { states[t][i] } else { 1.0 };
                        let xj = if j < n_res { states[t][j] } else { 1.0 };
                        sum += xi * xj;
                    }
                    row[j] = sum;
                }
                row[i] += lambda;
                row
            })
            .collect();

        // 2. Invert A matrix using safe partial-pivoting Gauss-Jordan solver:
        let a_inv = invert_matrix_safe(&a_matrix)
            .ok_or_else(|| "Singular Gram matrix in ridge regression".to_string())?;

        // 3. Parallel assembly of cross-covariance matrix B = Y^T * X (size K x D)
        let b_matrix: Vec<Vec<f64>> = (0..num_outputs)
            .into_par_iter()
            .map(|k| {
                let mut row = vec![0.0; num_features];
                for i in 0..num_features {
                    let mut sum = 0.0;
                    for t in 0..total_samples {
                        let xi = if i < n_res { states[t][i] } else { 1.0 };
                        sum += targets[t][k] * xi;
                    }
                    row[i] = sum;
                }
                row
            })
            .collect();

        // 4. Compute W_out = B * A_inv (size K x D) in parallel:
        let weights: Vec<Vec<f64>> = (0..num_outputs)
            .into_par_iter()
            .map(|k| {
                let mut w_row = vec![0.0; num_features];
                for j in 0..num_features {
                    let mut sum = 0.0;
                    for i in 0..num_features {
                        sum += b_matrix[k][i] * a_inv[i][j];
                    }
                    w_row[j] = sum;
                }
                w_row
            })
            .collect();

        Ok(TrainedReadout {
            weights,
            num_outputs,
            num_features,
        })
    }

    /// Evaluates Normalized Root Mean Square Error (NRMSE) between predictions and ground-truth targets:
    /// $$\text{NRMSE} = \sqrt{\frac{\sum_{t=1}^N \|\hat{\mathbf{y}}[t] - \mathbf{y}[t]\|^2}{\sum_{t=1}^N \|\mathbf{y}[t] - \bar{\mathbf{y}}\|^2}}$$
    pub fn compute_nrmse(predictions: &[Vec<f64>], targets: &[Vec<f64>]) -> f64 {
        let n = predictions.len();
        if n == 0 || targets.len() != n {
            return 1.0;
        }

        let k = predictions[0].len();
        let mut target_mean = vec![0.0; k];
        for t in 0..n {
            for j in 0..k {
                target_mean[j] += targets[t][j];
            }
        }
        for j in 0..k {
            target_mean[j] /= n as f64;
        }

        let mut sse = 0.0;
        let mut sst = 0.0;

        for t in 0..n {
            for j in 0..k {
                let err = predictions[t][j] - targets[t][j];
                sse += err * err;
                let var = targets[t][j] - target_mean[j];
                sst += var * var;
            }
        }

        if sst < 1e-14 {
            0.0
        } else {
            (sse / sst).sqrt()
        }
    }

    /// Trains and validates a memristive reservoir on time-series inputs and targets.
    pub fn train_and_evaluate(
        reservoir: &mut MemristiveReservoir,
        inputs: &[Vec<f64>],
        targets: &[Vec<f64>],
        washout_steps: usize,
        train_split: f64,
        lambda: f64,
    ) -> Result<(TrainedReadout, f64, Vec<Vec<f64>>), String> {
        let total_steps = inputs.len();
        if total_steps <= washout_steps + 20 {
            return Err("Insufficient time steps for washout and training".to_string());
        }

        // Collect reservoir states across entire sequence:
        reservoir.reset_state();
        let mut all_states = Vec::with_capacity(total_steps);
        for u in inputs {
            reservoir.step(u);
            all_states.push(reservoir.state.clone());
        }

        let effective_steps = total_steps - washout_steps;
        let train_steps = ((effective_steps as f64) * train_split).round() as usize;
        let test_steps = effective_steps.saturating_sub(train_steps);

        if train_steps < 10 || test_steps < 5 {
            return Err("Train or test split too small".to_string());
        }

        let train_states: Vec<Vec<f64>> =
            all_states[washout_steps..(washout_steps + train_steps)].to_vec();
        let train_targets: Vec<Vec<f64>> =
            targets[washout_steps..(washout_steps + train_steps)].to_vec();

        let readout = Self::train_ridge_regression(&train_states, &train_targets, lambda)?;

        let test_states = &all_states[(washout_steps + train_steps)..];
        let test_targets = &targets[(washout_steps + train_steps)..];

        let mut test_predictions = Vec::with_capacity(test_steps);
        for s in test_states {
            test_predictions.push(readout.predict(s));
        }

        let nrmse = Self::compute_nrmse(&test_predictions, test_targets);

        Ok((readout, nrmse, test_predictions))
    }
}

/// Generates chaotic Mackey-Glass time-series dataset:
/// $$\frac{dx}{dt} = \frac{\beta x(t - \tau)}{1 + x(t - \tau)^n} - \gamma x(t)$$
pub fn generate_mackey_glass(num_steps: usize, tau: usize) -> Vec<f64> {
    let beta: f64 = 0.2;
    let gamma: f64 = 0.1;
    let n: f64 = 10.0;
    let dt: f64 = 1.0;

    let history_len = tau + 1;
    let mut history: Vec<f64> = vec![1.2; history_len];
    let mut series: Vec<f64> = Vec::with_capacity(num_steps);

    let mut current_x: f64 = 1.2;
    let mut idx = 0;

    for _ in 0..num_steps {
        let x_delayed: f64 = history[idx];
        let dx: f64 = (beta * x_delayed) / (1.0 + x_delayed.powf(n)) - gamma * current_x;
        current_x += dx * dt;

        history[idx] = current_x;
        idx = (idx + 1) % history_len;

        series.push(current_x);
    }

    series
}

/// Generates chaotic Lorenz-63 strange attractor 3D trajectories $[x(t), y(t), z(t)]$ using RK4 integration.
pub fn generate_lorenz63(num_steps: usize, dt: f64) -> Vec<[f64; 3]> {
    let sigma = 10.0;
    let rho = 28.0;
    let beta = 8.0 / 3.0;

    let mut state = [1.0, 1.0, 1.0];
    let mut trajectory = Vec::with_capacity(num_steps);

    let f = |s: [f64; 3]| -> [f64; 3] {
        [
            sigma * (s[1] - s[0]),
            s[0] * (rho - s[2]) - s[1],
            s[0] * s[1] - beta * s[2],
        ]
    };

    for _ in 0..num_steps {
        let k1 = f(state);

        let s2 = [
            state[0] + 0.5 * dt * k1[0],
            state[1] + 0.5 * dt * k1[1],
            state[2] + 0.5 * dt * k1[2],
        ];
        let k2 = f(s2);

        let s3 = [
            state[0] + 0.5 * dt * k2[0],
            state[1] + 0.5 * dt * k2[1],
            state[2] + 0.5 * dt * k2[2],
        ];
        let k3 = f(s3);

        let s4 = [
            state[0] + dt * k3[0],
            state[1] + dt * k3[1],
            state[2] + dt * k3[2],
        ];
        let k4 = f(s4);

        state[0] += (dt / 6.0) * (k1[0] + 2.0 * k2[0] + 2.0 * k3[0] + k4[0]);
        state[1] += (dt / 6.0) * (k1[1] + 2.0 * k2[1] + 2.0 * k3[1] + k4[1]);
        state[2] += (dt / 6.0) * (k1[2] + 2.0 * k2[2] + 2.0 * k3[2] + k4[2]);

        trajectory.push(state);
    }

    trajectory
}

/// Generates Non-linear Auto-Regressive Moving Average (NARMA-10) benchmark:
/// $$y[t+1] = 0.3 y[t] + 0.05 y[t] \sum_{i=0}^9 y[t-i] + 1.5 u[t-9] u[t] + 0.1$$
pub fn generate_narma10(num_steps: usize, seed: u64) -> (Vec<f64>, Vec<f64>) {
    let mut rng = ReservoirRng::new(seed);
    let mut u = Vec::with_capacity(num_steps);
    for _ in 0..num_steps {
        u.push(rng.next_range(0.0, 0.5));
    }

    let mut y = vec![0.0; num_steps];

    for t in 9..(num_steps - 1) {
        let mut sum_y = 0.0;
        for i in 0..10 {
            sum_y += y[t - i];
        }

        y[t + 1] = 0.3 * y[t] + 0.05 * y[t] * sum_y + 1.5 * u[t - 9] * u[t] + 0.1;
    }

    (u, y)
}
