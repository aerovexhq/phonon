//! Polynomial Chaos Expansion (PCE) surrogate engine for uncertainty quantification and sensitivity analysis.

/// 1D Legendre polynomial evaluation up to arbitrary degree.
pub fn eval_legendre(order: usize, x: f64) -> (f64, f64) {
    // Returns (P_n(x), P_n'(x))
    if order == 0 {
        return (1.0, 0.0);
    }
    if order == 1 {
        return (x, 1.0);
    }
    let mut p_prev2 = 1.0;
    let mut p_prev1 = x;
    let mut dp_prev2 = 0.0;
    let mut dp_prev1 = 1.0;

    let mut p_curr = x;
    let mut dp_curr = 1.0;

    for n in 1..order {
        let n_f = n as f64;
        // Bonnet's recurrence: (n+1) P_{n+1}(x) = (2n+1) x P_n(x) - n P_{n-1}(x)
        p_curr = ((2.0 * n_f + 1.0) * x * p_prev1 - n_f * p_prev2) / (n_f + 1.0);
        // Derivative recurrence: P_{n+1}'(x) = (n+1) P_n(x) + x P_n'(x) (or standard derivative relation)
        // Alternative: (1 - x^2) P_n'(x) = -n x P_n(x) + n P_{n-1}(x)
        dp_curr = ((2.0 * n_f + 1.0) * (p_prev1 + x * dp_prev1) - n_f * dp_prev2) / (n_f + 1.0);

        p_prev2 = p_prev1;
        p_prev1 = p_curr;
        dp_prev2 = dp_prev1;
        dp_prev1 = dp_curr;
    }

    (p_curr, dp_curr)
}

/// A multi-index term in a multi-dimensional polynomial chaos expansion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PceTerm {
    /// Exponents / polynomial degrees for each input variable: size [dimension].
    pub degrees: Vec<usize>,
}

/// Polynomial Chaos Expansion metamodel: Y(xi) = sum_k c_k * Psi_k(xi).
#[derive(Debug, Clone, PartialEq)]
pub struct PolynomialChaosExpansion {
    pub dimension: usize,
    /// Multi-index set defining the basis functions.
    pub terms: Vec<PceTerm>,
    /// Expansion coefficients c_k.
    pub coefficients: Vec<f64>,
    /// Minimum bounds for each input dimension [dimension].
    pub input_mins: Vec<f64>,
    /// Maximum bounds for each input dimension [dimension].
    pub input_maxs: Vec<f64>,
}

impl PolynomialChaosExpansion {
    pub fn new(
        dimension: usize,
        terms: Vec<PceTerm>,
        coefficients: Vec<f64>,
        input_mins: Vec<f64>,
        input_maxs: Vec<f64>,
    ) -> Self {
        assert_eq!(terms.len(), coefficients.len());
        assert_eq!(input_mins.len(), dimension);
        assert_eq!(input_maxs.len(), dimension);
        Self {
            dimension,
            terms,
            coefficients,
            input_mins,
            input_maxs,
        }
    }

    /// Normalizes input x in [min, max] to standard domain xi in [-1, 1].
    pub fn normalize_input(&self, x: &[f64]) -> Vec<f64> {
        let mut xi = vec![0.0; self.dimension];
        for i in 0..self.dimension {
            let span = self.input_maxs[i] - self.input_mins[i];
            if span.abs() > 1e-12 {
                xi[i] = 2.0 * (x[i] - self.input_mins[i]) / span - 1.0;
            } else {
                xi[i] = 0.0;
            }
        }
        xi
    }

    /// Evaluates the PCE surrogate Y(x) and its gradient dY/dx at an arbitrary point x.
    /// Returns (y, gradient: Vec<f64> of size [dimension]).
    pub fn evaluate_with_gradient(&self, x: &[f64]) -> (f64, Vec<f64>) {
        assert_eq!(x.len(), self.dimension);
        let xi = self.normalize_input(x);

        let mut y = 0.0;
        let mut grad_xi = vec![0.0; self.dimension];

        // Precompute 1D Legendre evaluations for each variable and degree
        let mut max_degrees = vec![0; self.dimension];
        for term in &self.terms {
            for (i, &deg) in term.degrees.iter().enumerate() {
                max_degrees[i] = max_degrees[i].max(deg);
            }
        }

        let mut legendre_vals = Vec::with_capacity(self.dimension);
        let mut legendre_derivs = Vec::with_capacity(self.dimension);
        for i in 0..self.dimension {
            let mut v = Vec::with_capacity(max_degrees[i] + 1);
            let mut d = Vec::with_capacity(max_degrees[i] + 1);
            for deg in 0..=max_degrees[i] {
                let (val, deriv) = eval_legendre(deg, xi[i]);
                v.push(val);
                d.push(deriv);
            }
            legendre_vals.push(v);
            legendre_derivs.push(d);
        }

        // Sum over all expansion terms
        for (k, term) in self.terms.iter().enumerate() {
            let c = self.coefficients[k];
            let mut basis_val = 1.0;
            for (i, &deg) in term.degrees.iter().enumerate() {
                basis_val *= legendre_vals[i][deg];
            }
            y += c * basis_val;

            // Gradient with respect to xi_j via product rule
            for j in 0..self.dimension {
                let deg_j = term.degrees[j];
                if deg_j == 0 {
                    continue; // d(P_0)/dx = 0
                }
                let mut d_basis = c * legendre_derivs[j][deg_j];
                for i in 0..self.dimension {
                    if i != j {
                        d_basis *= legendre_vals[i][term.degrees[i]];
                    }
                }
                grad_xi[j] += d_basis;
            }
        }

        // Scale gradient from dY/dxi back to dY/dx: dY/dx = dY/dxi * dxi/dx = dY/dxi * (2 / span)
        let mut grad_x = vec![0.0; self.dimension];
        for i in 0..self.dimension {
            let span = self.input_maxs[i] - self.input_mins[i];
            if span.abs() > 1e-12 {
                grad_x[i] = grad_xi[i] * (2.0 / span);
            }
        }

        (y, grad_x)
    }

    /// Evaluates the PCE surrogate Y(x).
    pub fn evaluate(&self, x: &[f64]) -> f64 {
        self.evaluate_with_gradient(x).0
    }

    /// Computes the total model output variance and first-order Sobol sensitivity indices
    /// S_i = Var_i(Y) / Var(Y) describing the percentage contribution of variable x_i.
    pub fn sobol_indices(&self) -> (f64, Vec<f64>) {
        let mut var_total = 0.0;
        let mut var_first_order = vec![0.0; self.dimension];

        for (k, term) in self.terms.iter().enumerate() {
            let sum_deg: usize = term.degrees.iter().sum();
            if sum_deg == 0 {
                // Constant term c_0 is mean E[Y], does not contribute to variance
                continue;
            }

            // Norm squared of Legendre polynomial product: <P_n^2> = 1 / (2n + 1)
            let mut norm_sq = 1.0;
            for &deg in &term.degrees {
                norm_sq /= 2.0 * deg as f64 + 1.0;
            }
            let term_var = self.coefficients[k] * self.coefficients[k] * norm_sq;
            var_total += term_var;

            // Check if this is a purely first-order term (only variable i has degree > 0)
            let non_zero_vars: Vec<usize> = term
                .degrees
                .iter()
                .enumerate()
                .filter(|(_, &deg)| deg > 0)
                .map(|(idx, _)| idx)
                .collect();
            if non_zero_vars.len() == 1 {
                let var_idx = non_zero_vars[0];
                var_first_order[var_idx] += term_var;
            }
        }

        let sobol: Vec<f64> = if var_total > 1e-15 {
            var_first_order.iter().map(|&v| v / var_total).collect()
        } else {
            vec![0.0; self.dimension]
        };

        (var_total, sobol)
    }
}
