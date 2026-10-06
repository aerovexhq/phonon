#![deny(unsafe_code)]

//! Gaussian Process Surrogate Model & Bayesian Optimization Engine.
//!
//! Formulates:
//! - Squared Exponential / Radial Basis Function (RBF) covariance kernel
//! - Cholesky-based predictive mean mu(x*) and uncertainty variance sigma^2(x*)
//! - Expected Improvement (EI) and Upper Confidence Bound (UCB) acquisition functions
//! - Pure safe Rust linear algebra without external BLAS/LAPACK C dependencies

use std::f64::consts::PI;

/// Hyperparameters for Gaussian Process regression.
#[derive(Debug, Clone)]
pub struct GaussianProcessParams {
    /// Signal variance (amplitude scale sigma_f^2).
    pub signal_variance: f64,
    /// Characteristic lengthscale parameter ell.
    pub lengthscale: f64,
    /// Observation noise variance sigma_n^2 (nugget / jitter for numerical stability).
    pub noise_variance: f64,
}

impl Default for GaussianProcessParams {
    fn default() -> Self {
        Self {
            signal_variance: 1.0,
            lengthscale: 0.35,
            noise_variance: 1e-4,
        }
    }
}

/// Gaussian Process surrogate model storing observed design points and target responses.
#[derive(Debug, Clone)]
pub struct GaussianProcessRegressor {
    pub params: GaussianProcessParams,
    pub train_x: Vec<Vec<f64>>,
    pub train_y: Vec<f64>,
    weights: Vec<f64>,
    l_factor: Vec<Vec<f64>>,
}

impl GaussianProcessRegressor {
    pub fn new(params: GaussianProcessParams) -> Self {
        Self {
            params,
            train_x: Vec::new(),
            train_y: Vec::new(),
            weights: Vec::new(),
            l_factor: Vec::new(),
        }
    }

    /// Evaluates RBF kernel between two multi-dimensional feature vectors.
    pub fn kernel(&self, x1: &[f64], x2: &[f64]) -> f64 {
        let mut sq_dist = 0.0;
        for (a, b) in x1.iter().zip(x2.iter()) {
            let diff = a - b;
            sq_dist += diff * diff;
        }
        let ell_sq = self.params.lengthscale * self.params.lengthscale;
        self.params.signal_variance * (-0.5 * sq_dist / ell_sq.max(1e-6)).exp()
    }

    /// Fits the Gaussian Process to observed training points via Cholesky decomposition.
    pub fn fit(&mut self, x: Vec<Vec<f64>>, y: Vec<f64>) -> bool {
        let n = x.len();
        if n == 0 || n != y.len() {
            return false;
        }

        // Construct covariance matrix K with diagonal noise jitter
        let mut k = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..n {
                let cov = self.kernel(&x[i], &x[j]);
                k[i][j] = if i == j {
                    cov + self.params.noise_variance
                } else {
                    cov
                };
            }
        }

        // Cholesky decomposition L * L^T = K
        let mut l = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..=i {
                let mut sum = k[i][j];
                for p in 0..j {
                    sum -= l[i][p] * l[j][p];
                }
                if i == j {
                    if sum <= 0.0 {
                        // Regularize if non-positive definite
                        sum = self.params.noise_variance.max(1e-6);
                    }
                    l[i][j] = sum.sqrt();
                } else {
                    l[i][j] = sum / l[j][j].max(1e-12);
                }
            }
        }

        // Forward solve L * v = y
        let mut v = vec![0.0; n];
        for i in 0..n {
            let mut sum = y[i];
            for j in 0..i {
                sum -= l[i][j] * v[j];
            }
            v[i] = sum / l[i][i].max(1e-12);
        }

        // Backward solve L^T * alpha = v
        let mut alpha = vec![0.0; n];
        for i in (0..n).rev() {
            let mut sum = v[i];
            for j in (i + 1)..n {
                sum -= l[j][i] * alpha[j];
            }
            alpha[i] = sum / l[i][i].max(1e-12);
        }

        self.train_x = x;
        self.train_y = y;
        self.weights = alpha;
        self.l_factor = l;
        true
    }

    /// Evaluates predictive mean mu(x*) and variance sigma^2(x*) at a query point.
    pub fn predict(&self, query: &[f64]) -> (f64, f64) {
        let n = self.train_x.len();
        if n == 0 {
            return (0.0, self.params.signal_variance);
        }

        // Cross-covariance vector k_*
        let mut k_star = vec![0.0; n];
        for i in 0..n {
            k_star[i] = self.kernel(query, &self.train_x[i]);
        }

        // Predictive mean: mu = k_*^T * alpha
        let mut mean = 0.0;
        for i in 0..n {
            mean += k_star[i] * self.weights[i];
        }

        // Forward solve L * z = k_*
        let mut z = vec![0.0; n];
        for i in 0..n {
            let mut sum = k_star[i];
            for j in 0..i {
                sum -= self.l_factor[i][j] * z[j];
            }
            z[i] = sum / self.l_factor[i][i].max(1e-12);
        }

        // Variance: sigma^2 = k(x*, x*) - z^T * z
        let mut z_sq_sum = 0.0;
        for &val in &z {
            z_sq_sum += val * val;
        }

        let k_self = self.kernel(query, query) + self.params.noise_variance;
        let variance = (k_self - z_sq_sum).max(1e-9);

        (mean, variance)
    }

    /// Evaluates Expected Improvement (EI) acquisition function for maximization.
    pub fn expected_improvement(&self, query: &[f64], best_y: f64, xi: f64) -> f64 {
        let (mu, var) = self.predict(query);
        let sigma = var.sqrt();
        if sigma <= 1e-9 {
            return 0.0;
        }

        let delta = mu - best_y - xi;
        let z = delta / sigma;

        let pdf = (-0.5 * z * z).exp() / (2.0 * PI).sqrt();
        let cdf = 0.5 * (1.0 + erf_approx(z / std::f64::consts::SQRT_2));

        delta * cdf + sigma * pdf
    }

    /// Evaluates Upper Confidence Bound (UCB) acquisition function: UCB = mu + beta * sigma.
    pub fn upper_confidence_bound(&self, query: &[f64], beta: f64) -> f64 {
        let (mu, var) = self.predict(query);
        mu + beta * var.sqrt()
    }
}

/// Numerical approximation of standard error function erf(x).
fn erf_approx(x: f64) -> f64 {
    // Abramowitz and Stegun formula 7.1.26 (maximum error: 1.5e-7)
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let t = 1.0 / (1.0 + 0.3275911 * x.abs());
    let y = 1.0 - (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t - 0.284496736) * t + 0.254829592) * t * (-x * x).exp();
    sign * y
}

/// Visual candidate sample along 1D design parameter slice for CAD visualizer.
#[derive(Debug, Clone)]
pub struct BayesianSlicePoint {
    pub param_value: f64,
    pub gp_mean: f64,
    pub gp_uncertainty_upper: f64,
    pub gp_uncertainty_lower: f64,
    pub expected_improvement: f64,
}

/// Generates 1D slice of GP mean, uncertainty corridor (+/- 1.96 sigma, 95% CI), and EI acquisition curve.
pub fn generate_gp_slice(
    gp: &GaussianProcessRegressor,
    min_x: f64,
    max_x: f64,
    steps: usize,
    best_y: f64,
) -> Vec<BayesianSlicePoint> {
    let mut points = Vec::with_capacity(steps);
    let step_size = (max_x - min_x) / (steps.max(2) - 1) as f64;

    for i in 0..steps {
        let x_val = min_x + i as f64 * step_size;
        let (mu, var) = gp.predict(&[x_val]);
        let sigma = var.sqrt();
        let ei = gp.expected_improvement(&[x_val], best_y, 0.01);

        points.push(BayesianSlicePoint {
            param_value: x_val,
            gp_mean: mu,
            gp_uncertainty_upper: mu + 1.96 * sigma,
            gp_uncertainty_lower: mu - 1.96 * sigma,
            expected_improvement: ei,
        });
    }

    points
}
