//! Pure safe Rust Multilayer Perceptron (MLP) with analytical Jacobian evaluation.
//!
//! Provides C^1/C^2 smooth activation functions (SiLU, Tanh, GELU) and exact backpropagation
//! Jacobians (dY/dX) for seamless Newton-Raphson Modified Nodal Analysis (MNA) stamping.

/// Smooth non-linear activation functions suitable for gradient-based circuit convergence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationFunction {
    /// Linear (identity): f(x) = x, f'(x) = 1.
    Linear,
    /// Hyperbolic tangent: f(x) = tanh(x), f'(x) = 1 - tanh^2(x).
    Tanh,
    /// Sigmoid Linear Unit (SiLU / Swish-1): f(x) = x / (1 + e^-x).
    SiLU,
    /// Gaussian Error Linear Unit (GELU approximation).
    GELU,
}

impl ActivationFunction {
    /// Evaluates the activation function f(x).
    #[inline(always)]
    pub fn evaluate(&self, x: f64) -> f64 {
        match self {
            Self::Linear => x,
            Self::Tanh => x.tanh(),
            Self::SiLU => {
                let sig = 1.0 / (1.0 + (-x).clamp(-80.0, 80.0).exp());
                x * sig
            }
            Self::GELU => {
                // CDF approximation: 0.5 * x * (1 + tanh(sqrt(2/pi) * (x + 0.044715 * x^3)))
                let c = (2.0 / std::f64::consts::PI).sqrt();
                let inner = c * (x + 0.044715 * x * x * x);
                0.5 * x * (1.0 + inner.tanh())
            }
        }
    }

    /// Evaluates the analytical first derivative f'(x) given the pre-activation input x.
    #[inline(always)]
    pub fn derivative(&self, x: f64) -> f64 {
        match self {
            Self::Linear => 1.0,
            Self::Tanh => {
                let t = x.tanh();
                1.0 - t * t
            }
            Self::SiLU => {
                let sig = 1.0 / (1.0 + (-x).clamp(-80.0, 80.0).exp());
                sig + x * sig * (1.0 - sig)
            }
            Self::GELU => {
                let c = (2.0 / std::f64::consts::PI).sqrt();
                let inner = c * (x + 0.044715 * x * x * x);
                let tanh_val = inner.tanh();
                let sech2 = 1.0 - tanh_val * tanh_val;
                let d_inner = c * (1.0 + 3.0 * 0.044715 * x * x);
                0.5 * (1.0 + tanh_val) + 0.5 * x * sech2 * d_inner
            }
        }
    }
}

/// A dense feedforward layer with contiguous weight storage.
#[derive(Debug, Clone, PartialEq)]
pub struct DenseLayer {
    pub in_features: usize,
    pub out_features: usize,
    /// Contiguous weights in row-major layout: size [out_features * in_features].
    /// Weight at [i, j] is weights[i * in_features + j].
    pub weights: Vec<f64>,
    /// Bias vector: size [out_features].
    pub biases: Vec<f64>,
    pub activation: ActivationFunction,
}

impl DenseLayer {
    /// Creates a new dense layer initialized with provided weights and biases.
    pub fn new(
        in_features: usize,
        out_features: usize,
        weights: Vec<f64>,
        biases: Vec<f64>,
        activation: ActivationFunction,
    ) -> Self {
        assert_eq!(
            weights.len(),
            in_features * out_features,
            "Weight vector length must match in_features * out_features"
        );
        assert_eq!(
            biases.len(),
            out_features,
            "Bias vector length must match out_features"
        );
        Self {
            in_features,
            out_features,
            weights,
            biases,
            activation,
        }
    }

    /// Forward pass through the layer: z = W * x + b, a = f(z).
    /// Returns (pre_activation z, activated output a).
    #[allow(clippy::needless_range_loop)]
    pub fn forward(&self, input: &[f64]) -> (Vec<f64>, Vec<f64>) {
        assert_eq!(input.len(), self.in_features);
        let mut z = self.biases.clone();
        for i in 0..self.out_features {
            let row_offset = i * self.in_features;
            for j in 0..self.in_features {
                z[i] += self.weights[row_offset + j] * input[j];
            }
        }
        let a: Vec<f64> = z.iter().map(|&val| self.activation.evaluate(val)).collect();
        (z, a)
    }
}

/// Multilayer Perceptron surrogate network with analytical Jacobian calculation.
#[derive(Debug, Clone, PartialEq)]
pub struct MultilayerPerceptron {
    pub layers: Vec<DenseLayer>,
}

impl MultilayerPerceptron {
    pub fn new(layers: Vec<DenseLayer>) -> Self {
        for i in 0..layers.len().saturating_sub(1) {
            assert_eq!(
                layers[i].out_features,
                layers[i + 1].in_features,
                "Layer output dimension must match subsequent layer input dimension"
            );
        }
        Self { layers }
    }

    /// Number of inputs to the first layer.
    pub fn in_features(&self) -> usize {
        self.layers.first().map(|l| l.in_features).unwrap_or(0)
    }

    /// Number of outputs from the final layer.
    pub fn out_features(&self) -> usize {
        self.layers.last().map(|l| l.out_features).unwrap_or(0)
    }

    /// Standard forward inference returning output vector y.
    pub fn forward(&self, input: &[f64]) -> Vec<f64> {
        let mut current = input.to_vec();
        for layer in &self.layers {
            let (_, a) = layer.forward(&current);
            current = a;
        }
        current
    }

    /// Evaluates forward inference and simultaneously calculates the exact analytical
    /// Jacobian matrix J_{ij} = dy_i / dx_j via the backward chain rule.
    ///
    /// Returns: (outputs: Vec<f64>, jacobian: Vec<Vec<f64>>) where jacobian has
    /// dimension [out_features x in_features].
    #[allow(clippy::needless_range_loop)]
    pub fn forward_with_jacobian(&self, input: &[f64]) -> (Vec<f64>, Vec<Vec<f64>>) {
        let n_layers = self.layers.len();
        if n_layers == 0 {
            return (input.to_vec(), Vec::new());
        }

        // 1. Forward pass storing activations and pre-activations
        let mut pre_activations = Vec::with_capacity(n_layers);
        let mut activations = Vec::with_capacity(n_layers + 1);
        activations.push(input.to_vec());

        for layer in &self.layers {
            let (z, a) = layer.forward(activations.last().unwrap());
            pre_activations.push(z);
            activations.push(a);
        }

        let output = activations.last().unwrap().clone();
        let in_dim = self.in_features();
        let out_dim = self.out_features();

        // 2. Analytical Jacobian calculation via vector-Jacobian products
        // For each output component k in 0..out_dim, backpropagate one-hot gradient e_k
        let mut jacobian = vec![vec![0.0; in_dim]; out_dim];

        for k in 0..out_dim {
            let mut grad = vec![0.0; out_dim];
            grad[k] = 1.0;

            for l in (0..n_layers).rev() {
                let layer = &self.layers[l];
                let z = &pre_activations[l];

                // Element-wise derivative of activation: d_l = grad * f'(z)
                let mut d_l = vec![0.0; layer.out_features];
                for i in 0..layer.out_features {
                    d_l[i] = grad[i] * layer.activation.derivative(z[i]);
                }

                // Backpropagate to layer input: grad_prev = W^T * d_l
                let mut grad_prev = vec![0.0; layer.in_features];
                for j in 0..layer.in_features {
                    for i in 0..layer.out_features {
                        grad_prev[j] += layer.weights[i * layer.in_features + j] * d_l[i];
                    }
                }
                grad = grad_prev;
            }

            jacobian[k] = grad;
        }

        (output, jacobian)
    }
}
