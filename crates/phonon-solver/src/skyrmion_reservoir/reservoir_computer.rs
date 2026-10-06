#![deny(unsafe_code)]

//! Spintronic Reservoir Computing & Neuromorphic Feature Extraction Engine.
//!
//! Implements a virtual-node spintronic reservoir based on non-linear skyrmion
//! gyrotropic/breathing dynamics and STTO magnetoresistance modulation.
//! Includes ridge regression readout training, NARMA-10 benchmark solver,
//! waveform transformation, and memory capacity evaluation.

use std::f64::consts::PI;

/// Configuration parameters for the spintronic reservoir.
#[derive(Debug, Clone)]
pub struct SpintronicReservoirParams {
    /// Number of virtual nodes N_virt (typically 20 - 50).
    pub num_virtual_nodes: usize,
    /// Time constant / feedback coupling between virtual nodes (0.0 to 1.0).
    pub feedback_coupling: f64,
    /// Input scaling factor gamma_in.
    pub input_scale: f64,
    /// Bias parameter for non-linear activation.
    pub bias: f64,
    /// Leaky integration rate alpha in [0.0, 1.0] (typically 0.4 - 0.7).
    pub leaking_rate: f64,
    /// Ridge regression regularization parameter lambda (e.g. 1e-5).
    pub ridge_lambda: f64,
    /// Transient washout steps to discard initial state (e.g. 25).
    pub washout_steps: usize,
}

impl Default for SpintronicReservoirParams {
    fn default() -> Self {
        Self {
            num_virtual_nodes: 30,
            feedback_coupling: 0.65,
            leaking_rate: 0.50,
            input_scale: 1.20,
            bias: 0.15,
            ridge_lambda: 1e-5,
            washout_steps: 25,
        }
    }
}

/// Spintronic virtual-node reservoir co-processor.
#[derive(Debug, Clone)]
pub struct SpintronicReservoir {
    pub params: SpintronicReservoirParams,
    /// Mask weights vector W_in of length N_virt.
    pub input_mask: Vec<f64>,
    /// Trained readout weights W_out of length N_virt + 1 (including constant bias).
    pub readout_weights: Vec<f64>,
}

impl SpintronicReservoir {
    /// Creates a new reservoir initialized with alternating / pseudo-random input mask.
    pub fn new(params: SpintronicReservoirParams) -> Self {
        let n = params.num_virtual_nodes;
        let mut mask = Vec::with_capacity(n);
        // Deterministic pseudo-random sequence for reproducibility without external crate:
        let mut seed = 0x5A17_C0DE_u64;
        for _ in 0..n {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let val = ((seed >> 33) as f64) / ((1u64 << 31) as f64) - 0.5; // [-0.5, 0.5]
            mask.push(val * 2.0); // [-1.0, 1.0]
        }

        Self {
            params,
            input_mask: mask,
            readout_weights: vec![0.0; n + 1],
        }
    }

    /// Evaluates the physical non-linear skyrmion/STTO activation function:
    ///
    /// f(z) = tanh(z) + 0.15 * sin(2.5 * z)
    /// (Capturing higher-order non-linear magnetic gyrotropic and magnetoresistive response).
    #[inline]
    pub fn spintronic_nonlinearity(&self, z: f64) -> f64 {
        z.tanh() + 0.15 * (2.5 * z).sin()
    }

    /// Feeds an input stream u[k] through the virtual-node reservoir.
    ///
    /// Returns the reservoir state trajectory matrix of dimension (N_samples, N_virt).
    pub fn process_input_stream(&self, inputs: &[f64]) -> Vec<Vec<f64>> {
        let n_virt = self.params.num_virtual_nodes;
        let n_steps = inputs.len();
        let mut states = Vec::with_capacity(n_steps);

        let mut current_state = vec![0.0; n_virt];

        for &u in inputs {
            let mut next_state = vec![0.0; n_virt];
            for i in 0..n_virt {
                let prev_idx = if i == 0 { n_virt - 1 } else { i - 1 };
                let drive = self.params.feedback_coupling * current_state[prev_idx]
                    + self.params.input_scale * self.input_mask[i] * u
                    + self.params.bias;

                let act = self.spintronic_nonlinearity(drive);
                next_state[i] = (1.0 - self.params.leaking_rate) * current_state[i]
                    + self.params.leaking_rate * act;
            }
            current_state = next_state;
            states.push(current_state.clone());
        }

        states
    }

    /// Solves linear ridge regression system (A * W = b) using Gaussian elimination with partial pivoting.
    ///
    /// A is an (N x N) matrix, b is an (N) vector.
    pub fn solve_linear_system(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Result<Vec<f64>, String> {
        let n = b.len();
        if a.len() != n || a.iter().any(|row| row.len() != n) {
            return Err("Matrix dimensions mismatch in linear solve".to_string());
        }

        for col in 0..n {
            // Find pivot
            let mut max_row = col;
            let mut max_val = a[col][col].abs();
            for row in col + 1..n {
                let val = a[row][col].abs();
                if val > max_val {
                    max_val = val;
                    max_row = row;
                }
            }

            if max_val < 1e-18 {
                return Err("Singular or ill-conditioned matrix in ridge regression".to_string());
            }

            if max_row != col {
                a.swap(col, max_row);
                b.swap(col, max_row);
            }

            // Normalize pivot row
            let pivot = a[col][col];
            for j in col..n {
                a[col][j] /= pivot;
            }
            b[col] /= pivot;

            // Eliminate column in other rows
            for row in 0..n {
                if row != col {
                    let factor = a[row][col];
                    if factor.abs() > 1e-20 {
                        for j in col..n {
                            a[row][j] -= factor * a[col][j];
                        }
                        b[row] -= factor * b[col];
                    }
                }
            }
        }

        Ok(b)
    }

    /// Extracts linear and spintronic magnetoresistance quadratic features [x_i, x_i^2, 1.0].
    pub fn extract_features(&self, state: &[f64]) -> Vec<f64> {
        let mut phi = Vec::with_capacity(2 * state.len() + 1);
        phi.extend_from_slice(state);
        for &val in state {
            phi.push(val * val);
        }
        phi.push(1.0); // Bias term
        phi
    }

    /// Trains the readout weights W_out using Ridge Regression on the provided training pairs:
    ///
    /// W_out = Y * X^T * (X * X^T + lambda * I)^(-1)
    pub fn train_readout(
        &mut self,
        inputs: &[f64],
        targets: &[f64],
    ) -> Result<f64, String> {
        let n_samples = inputs.len();
        if n_samples != targets.len() {
            return Err("Input and target length mismatch".to_string());
        }
        let washout = self.params.washout_steps;
        if n_samples <= washout {
            return Err("Insufficient samples for reservoir washout".to_string());
        }

        let all_states = self.process_input_stream(inputs);
        let eff_samples = n_samples - washout;
        let n_features = 2 * self.params.num_virtual_nodes + 1; // linear + quadratic + constant bias

        // Form correlation matrix A = X * X^T + lambda * I
        // and target correlation vector b = X * Y^T
        let mut a_mat = vec![vec![0.0; n_features]; n_features];
        let mut b_vec = vec![0.0; n_features];

        for t in 0..eff_samples {
            let sample_idx = t + washout;
            let state = &all_states[sample_idx];
            let target = targets[sample_idx];

            let phi = self.extract_features(state);

            for i in 0..n_features {
                b_vec[i] += phi[i] * target;
                for j in 0..n_features {
                    a_mat[i][j] += phi[i] * phi[j];
                }
            }
        }

        // Add Tikhonov regularization lambda * I to diagonal
        for i in 0..n_features {
            a_mat[i][i] += self.params.ridge_lambda * eff_samples as f64;
        }

        let weights = Self::solve_linear_system(a_mat, b_vec)?;
        self.readout_weights = weights;

        // Evaluate train NMSE
        let preds = self.predict_from_states(&all_states[washout..]);
        let actual = &targets[washout..];
        let nmse = Self::calculate_nmse(actual, &preds);

        Ok(nmse)
    }

    /// Evaluates predictions from precomputed reservoir states.
    pub fn predict_from_states(&self, states: &[Vec<f64>]) -> Vec<f64> {
        states
            .iter()
            .map(|st| {
                let phi = self.extract_features(st);
                let mut sum = 0.0;
                for (w, f) in self.readout_weights.iter().zip(phi.iter()) {
                    sum += w * f;
                }
                sum
            })
            .collect()
    }

    /// Evaluates predictions directly on an unseen test input stream.
    pub fn predict(&self, inputs: &[f64]) -> Vec<f64> {
        let states = self.process_input_stream(inputs);
        self.predict_from_states(&states)
    }

    /// Calculates Normalized Mean Square Error (NMSE):
    ///
    /// NMSE = sum((y_actual - y_pred)^2) / sum((y_actual - mean(y_actual))^2)
    pub fn calculate_nmse(actual: &[f64], predicted: &[f64]) -> f64 {
        if actual.is_empty() || actual.len() != predicted.len() {
            return 1.0;
        }

        let mean = actual.iter().sum::<f64>() / actual.len() as f64;
        let mut num = 0.0;
        let mut den = 0.0;

        for (a, p) in actual.iter().zip(predicted.iter()) {
            let err = a - p;
            num += err * err;
            let dev = a - mean;
            den += dev * dev;
        }

        if den > 1e-20 {
            num / den
        } else {
            num
        }
    }

    /// Generates standard Non-linear Autoregressive Moving Average (NARMA-10) dataset:
    ///
    /// y[k+1] = 0.3 * y[k] + 0.05 * y[k] * sum_{i=0..9}(y[k-i]) + 1.5 * u[k-9] * u[k] + 0.1
    pub fn generate_narma10_dataset(num_points: usize) -> (Vec<f64>, Vec<f64>) {
        let mut inputs = Vec::with_capacity(num_points);
        let mut targets = vec![0.0; num_points];

        // Pseudo-random inputs uniformly distributed in [0.0, 0.5]:
        let mut seed = 0x481F_82B3_u64;
        for _ in 0..num_points {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let u = (((seed >> 33) as f64) / ((1u64 << 31) as f64)) * 0.5;
            inputs.push(u);
        }

        for k in 9..num_points - 1 {
            let sum_y: f64 = (0..10).map(|i| targets[k - i]).sum();
            targets[k + 1] = 0.3 * targets[k]
                + 0.05 * targets[k] * sum_y
                + 1.5 * inputs[k - 9] * inputs[k]
                + 0.1;
        }

        (inputs, targets)
    }

    /// Generates non-linear sine-to-square wave transformation dataset:
    pub fn generate_sine_to_square_dataset(num_points: usize) -> (Vec<f64>, Vec<f64>) {
        let mut inputs = Vec::with_capacity(num_points);
        let mut targets = Vec::with_capacity(num_points);

        for k in 0..num_points {
            let phase = 2.0 * PI * (k as f64) / 20.0; // Period of 20 samples
            let s = phase.sin();
            inputs.push(s);
            targets.push(if s >= 0.0 { 1.0 } else { -1.0 });
        }

        (inputs, targets)
    }

    /// Evaluates the Short-Term Memory (STM) capacity over delays d in 1..=max_delay:
    ///
    /// C_STM = sum_{d=1..max_delay} r^2(u[k - d], y_pred[k])
    pub fn evaluate_memory_capacity(&mut self, test_len: usize, max_delay: usize) -> f64 {
        let (inputs, _) = Self::generate_narma10_dataset(test_len + max_delay);
        let mut total_capacity = 0.0;

        for delay in 1..=max_delay {
            let mut delayed_targets = vec![0.0; test_len];
            for t in 0..test_len {
                delayed_targets[t] = inputs[t];
            }
            let sub_inputs = &inputs[delay..delay + test_len];

            if self.train_readout(sub_inputs, &delayed_targets).is_ok() {
                let preds = self.predict(sub_inputs);
                let actual = &delayed_targets[self.params.washout_steps..];
                let predicted = &preds[self.params.washout_steps..];

                // Correlation coefficient r^2
                let r2 = Self::calculate_r_squared(actual, predicted);
                total_capacity += r2.clamp(0.0, 1.0);
            }
        }

        total_capacity
    }

    /// Helper computing squared Pearson correlation coefficient r^2:
    pub fn calculate_r_squared(actual: &[f64], predicted: &[f64]) -> f64 {
        if actual.is_empty() || actual.len() != predicted.len() {
            return 0.0;
        }
        let mean_a = actual.iter().sum::<f64>() / actual.len() as f64;
        let mean_p = predicted.iter().sum::<f64>() / predicted.len() as f64;

        let mut cov = 0.0;
        let mut var_a = 0.0;
        let mut var_p = 0.0;

        for (a, p) in actual.iter().zip(predicted.iter()) {
            let da = a - mean_a;
            let dp = p - mean_p;
            cov += da * dp;
            var_a += da * da;
            var_p += dp * dp;
        }

        let denom = var_a * var_p;
        if denom > 1e-20 {
            (cov * cov) / denom
        } else {
            0.0
        }
    }
}
