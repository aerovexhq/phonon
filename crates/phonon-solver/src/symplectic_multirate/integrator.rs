#![deny(unsafe_code)]

//! High-order symplectic integrators including 2nd-order Stormer-Verlet / Implicit Midpoint
//! and 4th-order Gauss-Legendre Runge-Kutta (GLRK4) with zero-heap-allocation Newton-Raphson solvers.

/// Maximum supported state dimension for zero-heap-allocation stack-based vector solvers.
pub const MAX_STATE_DIM: usize = 32;

/// Maximum dimension of stage equations in GLRK4 (2 stages * MAX_STATE_DIM).
pub const MAX_STAGE_DIM: usize = 2 * MAX_STATE_DIM;

/// Butcher tableau coefficients for 4th-order Gauss-Legendre Runge-Kutta (GLRK4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glrk4ButcherTableau {
    /// Stage nodes c_i in [0, 1].
    pub c: [f64; 2],
    /// Runge-Kutta matrix a_ij.
    pub a: [[f64; 2]; 2],
    /// Quadrature weights b_i.
    pub b: [f64; 2],
}

impl Default for Glrk4ButcherTableau {
    fn default() -> Self {
        Self::new()
    }
}

impl Glrk4ButcherTableau {
    /// Constructs the exact Butcher tableau for 2-stage GLRK4.
    pub fn new() -> Self {
        let sqrt3_over_6 = 3.0_f64.sqrt() / 6.0;
        Self {
            c: [0.5 - sqrt3_over_6, 0.5 + sqrt3_over_6],
            a: [
                [0.25, 0.25 - sqrt3_over_6],
                [0.25 + sqrt3_over_6, 0.25],
            ],
            b: [0.5, 0.5],
        }
    }

    /// Evaluates the symplectic condition matrix M_ij = b_i * a_ij + b_j * a_ji - b_i * b_j.
    ///
    /// For any symplectic Runge-Kutta method, M_ij must be zero to machine precision for all i, j.
    pub fn symplectic_condition_matrix(&self) -> [[f64; 2]; 2] {
        let mut m = [[0.0; 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                m[i][j] = self.b[i] * self.a[i][j] + self.b[j] * self.a[j][i] - self.b[i] * self.b[j];
            }
        }
        m
    }
}

/// High-performance symplectic integrator engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SymplecticIntegrator {
    /// Butcher tableau for GLRK4.
    pub tableau: Glrk4ButcherTableau,
    /// Maximum allowed Newton-Raphson iterations.
    pub max_newton_iters: usize,
    /// Absolute convergence tolerance for Newton-Raphson residual.
    pub newton_tolerance: f64,
}

impl Default for SymplecticIntegrator {
    fn default() -> Self {
        Self::new()
    }
}

impl SymplecticIntegrator {
    /// Constructs a new SymplecticIntegrator with standard precision settings.
    pub fn new() -> Self {
        Self {
            tableau: Glrk4ButcherTableau::new(),
            max_newton_iters: 25,
            newton_tolerance: 1.0e-11,
        }
    }

    /// Advances state by one step dt using 4th-order Gauss-Legendre Runge-Kutta (GLRK4).
    ///
    /// The implicit stage vectors k_1 and k_2 are solved via Newton-Raphson iteration
    /// with zero heap allocations on stack buffers.
    pub fn step_glrk4<F>(&self, x: &[f64], t: f64, dt: f64, x_next: &mut [f64], mut f: F) -> bool
    where
        F: FnMut(&[f64], f64, &mut [f64]),
    {
        let d = x.len();
        if d == 0 || d > MAX_STATE_DIM {
            return false;
        }

        let n = 2 * d;
        let c1 = self.tableau.c[0];
        let c2 = self.tableau.c[1];
        let a11 = self.tableau.a[0][0];
        let a12 = self.tableau.a[0][1];
        let a21 = self.tableau.a[1][0];
        let a22 = self.tableau.a[1][1];
        let b1 = self.tableau.b[0];
        let b2 = self.tableau.b[1];

        // Initial stage guesses: evaluate vector field at current state
        let mut k1 = [0.0; MAX_STATE_DIM];
        let mut k2 = [0.0; MAX_STATE_DIM];
        f(x, t, &mut k1[..d]);
        for i in 0..d {
            k2[i] = k1[i];
        }

        let mut y1 = [0.0; MAX_STATE_DIM];
        let mut y2 = [0.0; MAX_STATE_DIM];
        let mut f_y1 = [0.0; MAX_STATE_DIM];
        let mut f_y2 = [0.0; MAX_STATE_DIM];
        let mut res = [0.0; MAX_STAGE_DIM];

        let mut j1 = [[0.0; MAX_STATE_DIM]; MAX_STATE_DIM];
        let mut j2 = [[0.0; MAX_STATE_DIM]; MAX_STATE_DIM];
        let mut jac = [[0.0; MAX_STAGE_DIM]; MAX_STAGE_DIM];
        let mut rhs = [0.0; MAX_STAGE_DIM];

        let eps = 1.0e-7;
        let inv_2eps = 1.0 / (2.0 * eps);

        let mut converged = false;

        for _iter in 0..self.max_newton_iters {
            // Compute stage state values Y_1, Y_2
            for i in 0..d {
                y1[i] = x[i] + dt * (a11 * k1[i] + a12 * k2[i]);
                y2[i] = x[i] + dt * (a21 * k1[i] + a22 * k2[i]);
            }

            f(&y1[..d], t + c1 * dt, &mut f_y1[..d]);
            f(&y2[..d], t + c2 * dt, &mut f_y2[..d]);

            // Residual: R_1 = k_1 - f(Y_1), R_2 = k_2 - f(Y_2)
            let mut res_norm_sq = 0.0;
            for i in 0..d {
                let r1 = k1[i] - f_y1[i];
                let r2 = k2[i] - f_y2[i];
                res[i] = r1;
                res[d + i] = r2;
                rhs[i] = -r1;
                rhs[d + i] = -r2;
                res_norm_sq += r1 * r1 + r2 * r2;
            }

            if res_norm_sq < self.newton_tolerance * self.newton_tolerance {
                converged = true;
                break;
            }

            // Evaluate system Jacobians J_1 = df/dx(Y_1) and J_2 = df/dx(Y_2)
            let mut pert_plus = [0.0; MAX_STATE_DIM];
            let mut pert_minus = [0.0; MAX_STATE_DIM];
            let mut f_plus = [0.0; MAX_STATE_DIM];
            let mut f_minus = [0.0; MAX_STATE_DIM];

            for j in 0..d {
                for i in 0..d {
                    pert_plus[i] = y1[i];
                    pert_minus[i] = y1[i];
                }
                pert_plus[j] += eps;
                pert_minus[j] -= eps;
                f(&pert_plus[..d], t + c1 * dt, &mut f_plus[..d]);
                f(&pert_minus[..d], t + c1 * dt, &mut f_minus[..d]);
                for i in 0..d {
                    j1[i][j] = (f_plus[i] - f_minus[i]) * inv_2eps;
                }
            }

            for j in 0..d {
                for i in 0..d {
                    pert_plus[i] = y2[i];
                    pert_minus[i] = y2[i];
                }
                pert_plus[j] += eps;
                pert_minus[j] -= eps;
                f(&pert_plus[..d], t + c2 * dt, &mut f_plus[..d]);
                f(&pert_minus[..d], t + c2 * dt, &mut f_minus[..d]);
                for i in 0..d {
                    j2[i][j] = (f_plus[i] - f_minus[i]) * inv_2eps;
                }
            }

            // Assemble 2D x 2D Jacobian of the residual with respect to [k_1, k_2]:
            // dR_1 / dk_1 = I_d - dt * a11 * J_1
            // dR_1 / dk_2 = - dt * a12 * J_1
            // dR_2 / dk_1 = - dt * a21 * J_2
            // dR_2 / dk_2 = I_d - dt * a22 * J_2
            for i in 0..d {
                for j in 0..d {
                    let eye = if i == j { 1.0 } else { 0.0 };
                    jac[i][j] = eye - dt * a11 * j1[i][j];
                    jac[i][d + j] = -dt * a12 * j1[i][j];
                    jac[d + i][j] = -dt * a21 * j2[i][j];
                    jac[d + i][d + j] = eye - dt * a22 * j2[i][j];
                }
            }

            // Solve linear system: jac * delta = rhs via stack LU/Gauss elimination
            if !solve_linear_system(&mut jac, &mut rhs, n) {
                return false;
            }

            // Update stage values: k_1 += delta_1, k_2 += delta_2
            let mut delta_norm_sq = 0.0;
            for i in 0..d {
                let dk1 = rhs[i];
                let dk2 = rhs[d + i];
                k1[i] += dk1;
                k2[i] += dk2;
                delta_norm_sq += dk1 * dk1 + dk2 * dk2;
            }

            if delta_norm_sq < self.newton_tolerance * self.newton_tolerance {
                converged = true;
                break;
            }
        }

        if !converged {
            return false;
        }

        // Final state update: x_{n+1} = x_n + dt * (b1 * k1 + b2 * k2)
        for i in 0..d {
            x_next[i] = x[i] + dt * (b1 * k1[i] + b2 * k2[i]);
        }
        true
    }

    /// Advances state by one step dt using the 2nd-order Implicit Midpoint / Stormer-Verlet rule:
    /// (x_{n+1} - x_n) / dt = f((x_{n+1} + x_n)/2, t_n + dt/2).
    ///
    /// Preserves the phase-space symplectic 2-form d p_{n+1} wedge d q_{n+1} = d p_n wedge d q_n.
    pub fn step_implicit_midpoint<F>(
        &self,
        x: &[f64],
        t: f64,
        dt: f64,
        x_next: &mut [f64],
        mut f: F,
    ) -> bool
    where
        F: FnMut(&[f64], f64, &mut [f64]),
    {
        let d = x.len();
        if d == 0 || d > MAX_STATE_DIM {
            return false;
        }

        let half_dt = 0.5 * dt;
        let t_mid = t + half_dt;

        // Stage derivative k = f(x_mid, t_mid) where x_mid = x + half_dt * k
        let mut k = [0.0; MAX_STATE_DIM];
        f(x, t, &mut k[..d]);

        let mut x_mid = [0.0; MAX_STATE_DIM];
        let mut f_mid = [0.0; MAX_STATE_DIM];
        let mut j_f = [[0.0; MAX_STATE_DIM]; MAX_STATE_DIM];
        let mut jac = [[0.0; MAX_STATE_DIM]; MAX_STATE_DIM];
        let mut rhs = [0.0; MAX_STATE_DIM];

        let eps = 1.0e-7;
        let inv_2eps = 1.0 / (2.0 * eps);

        let mut converged = false;

        for _iter in 0..self.max_newton_iters {
            for i in 0..d {
                x_mid[i] = x[i] + half_dt * k[i];
            }

            f(&x_mid[..d], t_mid, &mut f_mid[..d]);

            // Residual R = k - f(x_mid)
            let mut res_norm_sq = 0.0;
            for i in 0..d {
                let r = k[i] - f_mid[i];
                rhs[i] = -r;
                res_norm_sq += r * r;
            }

            if res_norm_sq < self.newton_tolerance * self.newton_tolerance {
                converged = true;
                break;
            }

            // Jacobian J_f = df/dx at x_mid
            let mut pert_plus = [0.0; MAX_STATE_DIM];
            let mut pert_minus = [0.0; MAX_STATE_DIM];
            let mut f_plus = [0.0; MAX_STATE_DIM];
            let mut f_minus = [0.0; MAX_STATE_DIM];

            for j in 0..d {
                for i in 0..d {
                    pert_plus[i] = x_mid[i];
                    pert_minus[i] = x_mid[i];
                }
                pert_plus[j] += eps;
                pert_minus[j] -= eps;
                f(&pert_plus[..d], t_mid, &mut f_plus[..d]);
                f(&pert_minus[..d], t_mid, &mut f_minus[..d]);
                for i in 0..d {
                    j_f[i][j] = (f_plus[i] - f_minus[i]) * inv_2eps;
                }
            }

            // Jacobian of residual: J_res = I_d - half_dt * J_f
            for i in 0..d {
                for j in 0..d {
                    let eye = if i == j { 1.0 } else { 0.0 };
                    jac[i][j] = eye - half_dt * j_f[i][j];
                }
            }

            if !solve_linear_system(&mut jac, &mut rhs, d) {
                return false;
            }

            let mut delta_norm_sq = 0.0;
            for i in 0..d {
                let dk = rhs[i];
                k[i] += dk;
                delta_norm_sq += dk * dk;
            }

            if delta_norm_sq < self.newton_tolerance * self.newton_tolerance {
                converged = true;
                break;
            }
        }

        if !converged {
            return false;
        }

        for i in 0..d {
            x_next[i] = x[i] + dt * k[i];
        }
        true
    }

    /// Canonical separable Stormer-Verlet integrator for Hamiltonian H(q, p) = 0.5 * p^T * M^-1 * p + V(q).
    ///
    /// Executes explicit kick-drift-kick step:
    /// p_{n+1/2} = p_n - 0.5 * dt * grad_V(q_n)
    /// q_{n+1} = q_n + dt * M^-1 * p_{n+1/2}
    /// p_{n+1} = p_{n+1/2} - 0.5 * dt * grad_V(q_{n+1}).
    #[inline(always)]
    pub fn step_stormer_verlet_separable<G>(
        &self,
        q: &mut [f64],
        p: &mut [f64],
        dt: f64,
        inv_mass: &[f64],
        mut grad_v: G,
    ) where
        G: FnMut(&[f64], &mut [f64]),
    {
        let d = q.len();
        let half_dt = 0.5 * dt;

        let mut g = [0.0; MAX_STATE_DIM];
        grad_v(q, &mut g[..d]);

        // Half kick
        for i in 0..d {
            p[i] -= half_dt * g[i];
        }

        // Full drift
        for i in 0..d {
            q[i] += dt * inv_mass[i] * p[i];
        }

        // Second half kick
        grad_v(q, &mut g[..d]);
        for i in 0..d {
            p[i] -= half_dt * g[i];
        }
    }

    /// Evaluates the determinant of the phase space Jacobian |det(J)| = |det(d x_{n+1} / d x_n)|
    /// for the Implicit Midpoint / Stormer-Verlet operator.
    pub fn stormer_verlet_phase_space_jacobian_det<F>(&self, x: &[f64], t: f64, dt: f64, mut f: F) -> f64
    where
        F: FnMut(&[f64], f64, &mut [f64]),
    {
        let d = x.len();
        let mut jac = [[0.0; MAX_STATE_DIM]; MAX_STATE_DIM];
        let eps = 1.0e-6;
        let inv_2eps = 1.0 / (2.0 * eps);

        let mut x_plus = [0.0; MAX_STATE_DIM];
        let mut x_minus = [0.0; MAX_STATE_DIM];
        let mut out_plus = [0.0; MAX_STATE_DIM];
        let mut out_minus = [0.0; MAX_STATE_DIM];

        for j in 0..d {
            for i in 0..d {
                x_plus[i] = x[i];
                x_minus[i] = x[i];
            }
            x_plus[j] += eps;
            x_minus[j] -= eps;

            self.step_implicit_midpoint(&x_plus[..d], t, dt, &mut out_plus[..d], &mut f);
            self.step_implicit_midpoint(&x_minus[..d], t, dt, &mut out_minus[..d], &mut f);

            for i in 0..d {
                jac[i][j] = (out_plus[i] - out_minus[i]) * inv_2eps;
            }
        }

        compute_determinant(&mut jac, d)
    }
}

/// Solves linear system A * x = b in place via Gaussian elimination with partial pivoting.
/// Matrix A and vector b are modified in place; solution is written to b.
fn solve_linear_system<const N: usize>(a: &mut [[f64; N]; N], b: &mut [f64; N], n: usize) -> bool {
    for i in 0..n {
        let mut pivot = i;
        let mut max_val = a[i][i].abs();
        for k in (i + 1)..n {
            let v = a[k][i].abs();
            if v > max_val {
                max_val = v;
                pivot = k;
            }
        }
        if max_val < 1.0e-14 {
            return false;
        }
        if pivot != i {
            for j in 0..n {
                let tmp = a[i][j];
                a[i][j] = a[pivot][j];
                a[pivot][j] = tmp;
            }
            let tmp = b[i];
            b[i] = b[pivot];
            b[pivot] = tmp;
        }

        let diag = a[i][i];
        for k in (i + 1)..n {
            let factor = a[k][i] / diag;
            for j in (i + 1)..n {
                a[k][j] -= factor * a[i][j];
            }
            b[k] -= factor * b[i];
        }
    }

    // Back substitution
    for i in (0..n).rev() {
        let mut sum = b[i];
        for j in (i + 1)..n {
            sum -= a[i][j] * b[j];
        }
        b[i] = sum / a[i][i];
    }
    true
}

/// Computes determinant of n x n matrix via LU decomposition with partial pivoting.
fn compute_determinant<const N: usize>(a: &mut [[f64; N]; N], n: usize) -> f64 {
    let mut det = 1.0;
    for i in 0..n {
        let mut pivot = i;
        let mut max_val = a[i][i].abs();
        for k in (i + 1)..n {
            let v = a[k][i].abs();
            if v > max_val {
                max_val = v;
                pivot = k;
            }
        }
        if max_val < 1.0e-14 {
            return 0.0;
        }
        if pivot != i {
            for j in 0..n {
                let tmp = a[i][j];
                a[i][j] = a[pivot][j];
                a[pivot][j] = tmp;
            }
            det = -det;
        }

        det *= a[i][i];
        let diag = a[i][i];
        for k in (i + 1)..n {
            let factor = a[k][i] / diag;
            for j in (i + 1)..n {
                a[k][j] -= factor * a[i][j];
            }
        }
    }
    det
}
