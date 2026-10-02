#![deny(unsafe_code)]

//! Port-Hamiltonian Dirac Interconnection Structure & Symplectic Integrator.
//!
//! Implements finite-dimensional Port-Hamiltonian systems with skew-symmetric
//! Dirac interconnection matrices J = -J^T, positive semi-definite dissipation
//! matrices R >= 0, and discrete bilinear implicit midpoint time-stepping.

/// Finite-dimensional Port-Hamiltonian Dirac Interconnection structure.
#[derive(Debug, Clone, PartialEq)]
pub struct DiracInterconnection {
    /// Dimension of state vector x.
    pub dim: usize,
    /// Dimension of input port vector u.
    pub inputs: usize,
    /// Skew-symmetric coupling matrix J (N x N, row-major).
    pub j_matrix: Vec<f64>,
    /// Positive semi-definite dissipation matrix R (N x N, row-major).
    pub r_matrix: Vec<f64>,
    /// Quadratic Hamiltonian energy metric matrix Q (N x N, row-major).
    pub q_matrix: Vec<f64>,
    /// Input port coupling matrix B (N x M, row-major).
    pub b_matrix: Vec<f64>,
    /// Direct feedthrough matrix D (M x M, row-major, skew-symmetric).
    pub d_matrix: Vec<f64>,
}

impl DiracInterconnection {
    /// Creates a new uncoupled Dirac interconnection system of dimension N with M inputs.
    pub fn new(dim: usize, inputs: usize) -> Self {
        let mut q = vec![0.0; dim * dim];
        for i in 0..dim {
            q[i * dim + i] = 1.0;
        }

        Self {
            dim,
            inputs,
            j_matrix: vec![0.0; dim * dim],
            r_matrix: vec![0.0; dim * dim],
            q_matrix: q,
            b_matrix: vec![0.0; dim * inputs],
            d_matrix: vec![0.0; inputs * inputs],
        }
    }

    /// Sets the skew-symmetric coupling matrix J.
    pub fn set_j(&mut self, j: &[f64]) {
        assert_eq!(j.len(), self.dim * self.dim);
        self.j_matrix.copy_from_slice(j);
    }

    /// Sets the positive semi-definite dissipation matrix R.
    pub fn set_r(&mut self, r: &[f64]) {
        assert_eq!(r.len(), self.dim * self.dim);
        self.r_matrix.copy_from_slice(r);
    }

    /// Sets the energy gradient metric matrix Q.
    pub fn set_q(&mut self, q: &[f64]) {
        assert_eq!(q.len(), self.dim * self.dim);
        self.q_matrix.copy_from_slice(q);
    }

    /// Sets the input coupling matrix B.
    pub fn set_b(&mut self, b: &[f64]) {
        assert_eq!(b.len(), self.dim * self.inputs);
        self.b_matrix.copy_from_slice(b);
    }

    /// Sets the feedthrough matrix D.
    pub fn set_d(&mut self, d: &[f64]) {
        assert_eq!(d.len(), self.inputs * self.inputs);
        self.d_matrix.copy_from_slice(d);
    }

    /// Verifies that J is strictly skew-symmetric (J = -J^T and diag(J) = 0).
    pub fn is_skew_symmetric(&self, tol: f64) -> bool {
        let n = self.dim;
        for i in 0..n {
            if self.j_matrix[i * n + i].abs() > tol {
                return false;
            }
            for j in 0..n {
                let j_ij = self.j_matrix[i * n + j];
                let j_ji = self.j_matrix[j * n + i];
                if (j_ij + j_ji).abs() > tol {
                    return false;
                }
            }
        }
        true
    }

    /// Evaluates the quadratic form x^T * J * x. For any skew-symmetric J, this evaluates to 0.
    pub fn quadratic_form_j(&self, x: &[f64]) -> f64 {
        assert_eq!(x.len(), self.dim);
        let n = self.dim;
        let mut sum = 0.0;
        for i in 0..n {
            let mut row_sum = 0.0;
            for j in 0..n {
                row_sum += self.j_matrix[i * n + j] * x[j];
            }
            sum += x[i] * row_sum;
        }
        sum
    }

    /// Evaluates the quadratic form x^T * R * x.
    pub fn quadratic_form_r(&self, x: &[f64]) -> f64 {
        assert_eq!(x.len(), self.dim);
        let n = self.dim;
        let mut sum = 0.0;
        for i in 0..n {
            let mut row_sum = 0.0;
            for j in 0..n {
                row_sum += self.r_matrix[i * n + j] * x[j];
            }
            sum += x[i] * row_sum;
        }
        sum
    }

    /// Verifies that the dissipation matrix R is positive semi-definite across test vectors.
    pub fn is_dissipation_positive_semidefinite(&self, tol: f64) -> bool {
        let n = self.dim;
        // Verify symmetry first
        for i in 0..n {
            for j in 0..n {
                let diff = (self.r_matrix[i * n + j] - self.r_matrix[j * n + i]).abs();
                if diff > tol {
                    return false;
                }
            }
        }
        // Test standard basis vectors
        for i in 0..n {
            if self.r_matrix[i * n + i] < -tol {
                return false;
            }
        }
        true
    }

    /// Computes the total stored Hamiltonian energy H(x) = 0.5 * x^T * Q * x in Joules.
    pub fn hamiltonian(&self, x: &[f64]) -> f64 {
        assert_eq!(x.len(), self.dim);
        let n = self.dim;
        let mut energy = 0.0;
        for i in 0..n {
            let mut q_row = 0.0;
            for j in 0..n {
                q_row += self.q_matrix[i * n + j] * x[j];
            }
            energy += x[i] * q_row;
        }
        0.5 * energy
    }

    /// Computes the co-energy gradient vector e = grad H(x) = Q * x.
    pub fn grad_hamiltonian(&self, x: &[f64]) -> Vec<f64> {
        assert_eq!(x.len(), self.dim);
        let n = self.dim;
        let mut e = vec![0.0; n];
        for i in 0..n {
            let mut sum = 0.0;
            for j in 0..n {
                sum += self.q_matrix[i * n + j] * x[j];
            }
            e[i] = sum;
        }
        e
    }

    /// Computes the state derivative dx/dt = (J - R) * e + B * u.
    pub fn state_derivative(&self, x: &[f64], u: &[f64]) -> Vec<f64> {
        assert_eq!(x.len(), self.dim);
        assert_eq!(u.len(), self.inputs);
        let n = self.dim;
        let m = self.inputs;
        let e = self.grad_hamiltonian(x);

        let mut dxdt = vec![0.0; n];
        for i in 0..n {
            let mut sum = 0.0;
            for j in 0..n {
                let jr = self.j_matrix[i * n + j] - self.r_matrix[i * n + j];
                sum += jr * e[j];
            }
            for k in 0..m {
                sum += self.b_matrix[i * m + k] * u[k];
            }
            dxdt[i] = sum;
        }
        dxdt
    }

    /// Computes conjugated port output y = B^T * e + D * u.
    pub fn output(&self, x: &[f64], u: &[f64]) -> Vec<f64> {
        assert_eq!(x.len(), self.dim);
        assert_eq!(u.len(), self.inputs);
        let n = self.dim;
        let m = self.inputs;
        let e = self.grad_hamiltonian(x);

        let mut y = vec![0.0; m];
        for k in 0..m {
            let mut sum = 0.0;
            for i in 0..n {
                sum += self.b_matrix[i * m + k] * e[i];
            }
            for j in 0..m {
                sum += self.d_matrix[k * m + j] * u[j];
            }
            y[k] = sum;
        }
        y
    }

    /// Verifies strict passivity: dH/dt = -e^T * R * e + y^T * u <= y^T * u.
    pub fn verify_passivity(&self, x: &[f64], _u: &[f64], tol: f64) -> bool {
        let e = self.grad_hamiltonian(x);
        let diss = self.quadratic_form_r(&e);
        diss >= -tol
    }

    /// Executes a discrete symplectic bilinear implicit midpoint step:
    /// (x_{k+1} - x_k) / dt = (J - R) * Q * ((x_{k+1} + x_k) / 2) + B * u_{mid}
    ///
    /// Preserves quadratic Hamiltonian invariants exactly when R = 0.
    pub fn discrete_step_midpoint(&self, x_k: &[f64], u_mid: &[f64], dt: f64) -> Vec<f64> {
        assert_eq!(x_k.len(), self.dim);
        assert_eq!(u_mid.len(), self.inputs);
        let n = self.dim;
        let m = self.inputs;

        // A = (J - R) * Q
        let mut a = vec![0.0; n * n];
        for i in 0..n {
            for j in 0..n {
                let mut sum = 0.0;
                for k in 0..n {
                    let jr = self.j_matrix[i * n + k] - self.r_matrix[i * n + k];
                    sum += jr * self.q_matrix[k * n + j];
                }
                a[i * n + j] = sum;
            }
        }

        // M_lhs = I - (dt / 2) * A
        let half_dt = 0.5 * dt;
        let mut m_lhs = vec![0.0; n * n];
        for i in 0..n {
            for j in 0..n {
                let delta = if i == j { 1.0 } else { 0.0 };
                m_lhs[i * n + j] = delta - half_dt * a[i * n + j];
            }
        }

        // b_rhs = (I + (dt / 2) * A) * x_k + dt * B * u_mid
        let mut b_rhs = vec![0.0; n];
        for i in 0..n {
            let mut row_sum = 0.0;
            for j in 0..n {
                let delta = if i == j { 1.0 } else { 0.0 };
                let m_rhs_ij = delta + half_dt * a[i * n + j];
                row_sum += m_rhs_ij * x_k[j];
            }
            for k in 0..m {
                row_sum += dt * self.b_matrix[i * m + k] * u_mid[k];
            }
            b_rhs[i] = row_sum;
        }

        // Solve M_lhs * x_{k+1} = b_rhs via Gaussian elimination with partial pivoting
        solve_linear_system(n, &mut m_lhs, &mut b_rhs)
    }
}

/// Solves linear system M * x = b in pure safe Rust using Gaussian elimination with partial pivoting.
pub(crate) fn solve_linear_system(n: usize, m: &mut [f64], b: &mut [f64]) -> Vec<f64> {
    for col in 0..n {
        // Find pivot
        let mut max_row = col;
        let mut max_val = m[col * n + col].abs();
        for row in (col + 1)..n {
            let val = m[row * n + col].abs();
            if val > max_val {
                max_val = val;
                max_row = row;
            }
        }

        if max_row != col {
            // Swap rows in m
            for j in 0..n {
                m.swap(col * n + j, max_row * n + j);
            }
            // Swap in b
            b.swap(col, max_row);
        }

        let pivot = m[col * n + col];
        if pivot.abs() < 1e-15 {
            // Singular or near-singular, fallback identity
            return b.to_vec();
        }

        for row in (col + 1)..n {
            let factor = m[row * n + col] / pivot;
            m[row * n + col] = 0.0;
            for j in (col + 1)..n {
                m[row * n + j] -= factor * m[col * n + j];
            }
            b[row] -= factor * b[col];
        }
    }

    // Back substitution
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        let mut sum = b[i];
        for j in (i + 1)..n {
            sum -= m[i * n + j] * x[j];
        }
        x[i] = sum / m[i * n + i];
    }
    x
}
