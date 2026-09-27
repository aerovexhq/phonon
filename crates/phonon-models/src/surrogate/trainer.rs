//! Automated in-memory training and regression harness for neural MLP and PCE surrogates.

use super::mlp::{ActivationFunction, DenseLayer, MultilayerPerceptron};
use super::pce::{eval_legendre, PceTerm, PolynomialChaosExpansion};

/// Configuration options for training an MLP surrogate.
#[derive(Debug, Clone)]
pub struct MlpTrainingConfig {
    pub hidden_layers: Vec<usize>,
    pub activation: ActivationFunction,
    pub learning_rate: f64,
    pub epochs: usize,
    pub batch_size: usize,
}

impl Default for MlpTrainingConfig {
    fn default() -> Self {
        Self {
            hidden_layers: vec![16, 16],
            activation: ActivationFunction::SiLU,
            learning_rate: 0.01,
            epochs: 200,
            batch_size: 32,
        }
    }
}

/// Trains a Multilayer Perceptron surrogate on input-output pairs using Adam optimization.
#[allow(clippy::needless_range_loop)]
pub fn train_mlp_surrogate(
    inputs: &[Vec<f64>],
    targets: &[Vec<f64>],
    config: &MlpTrainingConfig,
) -> MultilayerPerceptron {
    assert_eq!(inputs.len(), targets.len());
    let n_samples = inputs.len();
    assert!(n_samples > 0);
    let in_dim = inputs[0].len();
    let out_dim = targets[0].len();

    // 1. Initialize network architecture
    let mut layer_dims = Vec::with_capacity(config.hidden_layers.len() + 2);
    layer_dims.push(in_dim);
    layer_dims.extend_from_slice(&config.hidden_layers);
    layer_dims.push(out_dim);

    let mut layers = Vec::new();
    let mut seed = 42u64;

    for i in 0..(layer_dims.len() - 1) {
        let n_in = layer_dims[i];
        let n_out = layer_dims[i + 1];
        let is_last = i == layer_dims.len() - 2;
        let act = if is_last {
            ActivationFunction::Linear
        } else {
            config.activation
        };

        // Xavier/Glorot uniform initialization
        let limit = (6.0 / (n_in + n_out) as f64).sqrt();
        let mut weights = Vec::with_capacity(n_in * n_out);
        for _ in 0..(n_in * n_out) {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let unit = (seed as f64) / (u64::MAX as f64);
            let w = -limit + 2.0 * limit * unit;
            weights.push(w);
        }
        let biases = vec![0.0; n_out];
        layers.push(DenseLayer::new(n_in, n_out, weights, biases, act));
    }

    let mut mlp = MultilayerPerceptron::new(layers);

    // 2. Adam optimizer state
    let mut m_weights = Vec::new();
    let mut v_weights = Vec::new();
    let mut m_biases = Vec::new();
    let mut v_biases = Vec::new();

    for l in &mlp.layers {
        m_weights.push(vec![0.0; l.weights.len()]);
        v_weights.push(vec![0.0; l.weights.len()]);
        m_biases.push(vec![0.0; l.biases.len()]);
        v_biases.push(vec![0.0; l.biases.len()]);
    }

    let beta1: f64 = 0.9;
    let beta2: f64 = 0.999;
    let eps: f64 = 1e-8;
    let mut t_step: i32 = 0;

    // 3. Mini-batch gradient descent loop
    let lr = config.learning_rate;
    let batch_sz = config.batch_size.min(n_samples);

    for _ in 0..config.epochs {
        let mut sample_idx = 0;
        while sample_idx < n_samples {
            t_step += 1;
            let current_batch_size = (n_samples - sample_idx).min(batch_sz);

            // Accumulate gradients across batch
            let mut grad_weights: Vec<Vec<f64>> = mlp
                .layers
                .iter()
                .map(|l| vec![0.0; l.weights.len()])
                .collect();
            let mut grad_biases: Vec<Vec<f64>> = mlp
                .layers
                .iter()
                .map(|l| vec![0.0; l.biases.len()])
                .collect();

            for b in 0..current_batch_size {
                let x = &inputs[sample_idx + b];
                let y_true = &targets[sample_idx + b];

                // Forward pass saving activations
                let mut activations = Vec::with_capacity(mlp.layers.len() + 1);
                let mut pre_acts = Vec::with_capacity(mlp.layers.len());
                activations.push(x.clone());

                for layer in &mlp.layers {
                    let (z, a) = layer.forward(activations.last().unwrap());
                    pre_acts.push(z);
                    activations.push(a);
                }

                let y_pred = activations.last().unwrap();

                // Compute loss gradient: dL/dy_pred = 2 * (y_pred - y_true) / batch_size
                let mut delta = vec![0.0; out_dim];
                for k in 0..out_dim {
                    delta[k] = (2.0 * (y_pred[k] - y_true[k])) / (current_batch_size as f64);
                }

                // Backpropagate deltas
                for l in (0..mlp.layers.len()).rev() {
                    let layer = &mlp.layers[l];
                    let z = &pre_acts[l];
                    let a_in = &activations[l];

                    // delta_l = delta * f'(z)
                    let mut delta_l = vec![0.0; layer.out_features];
                    for i in 0..layer.out_features {
                        delta_l[i] = delta[i] * layer.activation.derivative(z[i]);
                    }

                    // Weight gradients: dL/dW_ij = delta_l[i] * a_in[j]
                    for i in 0..layer.out_features {
                        let row_offset = i * layer.in_features;
                        for j in 0..layer.in_features {
                            grad_weights[l][row_offset + j] += delta_l[i] * a_in[j];
                        }
                        grad_biases[l][i] += delta_l[i];
                    }

                    // Propagate delta to previous layer
                    let mut delta_prev = vec![0.0; layer.in_features];
                    for j in 0..layer.in_features {
                        for i in 0..layer.out_features {
                            delta_prev[j] += layer.weights[i * layer.in_features + j] * delta_l[i];
                        }
                    }
                    delta = delta_prev;
                }
            }

            // Apply Adam updates to weights and biases
            let corr1 = 1.0 - beta1.powi(t_step);
            let corr2 = 1.0 - beta2.powi(t_step);

            for (l, layer) in mlp.layers.iter_mut().enumerate() {
                for (w_idx, w) in layer.weights.iter_mut().enumerate() {
                    let g = grad_weights[l][w_idx];
                    m_weights[l][w_idx] = beta1 * m_weights[l][w_idx] + (1.0 - beta1) * g;
                    v_weights[l][w_idx] = beta2 * v_weights[l][w_idx] + (1.0 - beta2) * g * g;

                    let m_hat = m_weights[l][w_idx] / corr1;
                    let v_hat = v_weights[l][w_idx] / corr2;
                    *w -= lr * m_hat / (v_hat.sqrt() + eps);
                }

                for (b_idx, b) in layer.biases.iter_mut().enumerate() {
                    let g = grad_biases[l][b_idx];
                    m_biases[l][b_idx] = beta1 * m_biases[l][b_idx] + (1.0 - beta1) * g;
                    v_biases[l][b_idx] = beta2 * v_biases[l][b_idx] + (1.0 - beta2) * g * g;

                    let m_hat = m_biases[l][b_idx] / corr1;
                    let v_hat = v_biases[l][b_idx] / corr2;
                    *b -= lr * m_hat / (v_hat.sqrt() + eps);
                }
            }

            sample_idx += current_batch_size;
        }
    }

    mlp
}

/// Fits a Polynomial Chaos Expansion (PCE) surrogate to scalar output data using linear regression.
#[allow(clippy::needless_range_loop)]
pub fn fit_pce_surrogate(
    inputs: &[Vec<f64>],
    targets: &[f64],
    max_degree: usize,
) -> PolynomialChaosExpansion {
    let n_samples = inputs.len();
    assert!(n_samples > 0);
    assert_eq!(n_samples, targets.len());
    let dim = inputs[0].len();

    // 1. Determine input domain [min, max]
    let mut mins = vec![f64::INFINITY; dim];
    let mut maxs = vec![f64::NEG_INFINITY; dim];
    for x in inputs {
        for i in 0..dim {
            mins[i] = mins[i].min(x[i]);
            maxs[i] = maxs[i].max(x[i]);
        }
    }

    // 2. Generate multi-indices with total degree <= max_degree
    let mut terms = Vec::new();
    let mut current_term = vec![0; dim];
    fn gen_indices(
        dim_idx: usize,
        rem_degree: usize,
        current: &mut Vec<usize>,
        terms: &mut Vec<PceTerm>,
    ) {
        if dim_idx == current.len() - 1 {
            for deg in 0..=rem_degree {
                current[dim_idx] = deg;
                terms.push(PceTerm {
                    degrees: current.clone(),
                });
            }
        } else {
            for deg in 0..=rem_degree {
                current[dim_idx] = deg;
                gen_indices(dim_idx + 1, rem_degree - deg, current, terms);
            }
        }
    }
    gen_indices(0, max_degree, &mut current_term, &mut terms);
    let p_terms = terms.len();

    // 3. Assemble measurement matrix Phi [n_samples x p_terms]
    let mut phi = vec![vec![0.0; p_terms]; n_samples];
    for (s, x) in inputs.iter().enumerate() {
        let mut xi = vec![0.0; dim];
        for i in 0..dim {
            let span = maxs[i] - mins[i];
            xi[i] = if span.abs() > 1e-12 {
                2.0 * (x[i] - mins[i]) / span - 1.0
            } else {
                0.0
            };
        }

        for (k, term) in terms.iter().enumerate() {
            let mut val = 1.0;
            for (i, &deg) in term.degrees.iter().enumerate() {
                val *= eval_legendre(deg, xi[i]).0;
            }
            phi[s][k] = val;
        }
    }

    // 4. Solve normal equations: (Phi^T * Phi + lambda * I) * c = Phi^T * y
    let mut ata = vec![vec![0.0; p_terms]; p_terms];
    let mut aty = vec![0.0; p_terms];

    for s in 0..n_samples {
        let y_s = targets[s];
        for j in 0..p_terms {
            let phi_sj = phi[s][j];
            aty[j] += phi_sj * y_s;
            for k in 0..p_terms {
                ata[j][k] += phi_sj * phi[s][k];
            }
        }
    }

    // Tikhonov regularization for numerical stability
    let lambda = 1e-8;
    for i in 0..p_terms {
        ata[i][i] += lambda;
    }

    // Solve dense system ATA * c = ATY via Gaussian elimination with partial pivoting
    let mut a_mat = ata;
    let mut b_vec = aty;

    for i in 0..p_terms {
        // Pivot
        let mut max_row = i;
        let mut max_val = a_mat[i][i].abs();
        for r in (i + 1)..p_terms {
            if a_mat[r][i].abs() > max_val {
                max_val = a_mat[r][i].abs();
                max_row = r;
            }
        }
        if max_row != i {
            a_mat.swap(i, max_row);
            b_vec.swap(i, max_row);
        }

        let pivot = a_mat[i][i];
        if pivot.abs() < 1e-18 {
            continue;
        }

        for r in (i + 1)..p_terms {
            let factor = a_mat[r][i] / pivot;
            for c in i..p_terms {
                a_mat[r][c] -= factor * a_mat[i][c];
            }
            b_vec[r] -= factor * b_vec[i];
        }
    }

    // Back substitution
    let mut coeffs = vec![0.0; p_terms];
    for i in (0..p_terms).rev() {
        let mut sum = b_vec[i];
        for c in (i + 1)..p_terms {
            sum -= a_mat[i][c] * coeffs[c];
        }
        coeffs[i] = if a_mat[i][i].abs() > 1e-18 {
            sum / a_mat[i][i]
        } else {
            0.0
        };
    }

    PolynomialChaosExpansion::new(dim, terms, coeffs, mins, maxs)
}
