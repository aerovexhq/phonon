#![deny(unsafe_code)]

//! Non-Hermitian Jordan block decomposition, exceptional point (EP) eigensolver,
//! and topological Riemann surface trajectory tracker.
//!
//! Provides spectral decomposition and fractional sensitivity analysis for higher-order
//! exceptional points (EP2, EP3, EP4) in non-Hermitian phononic and photonic systems.

use std::f64::consts::PI;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

/// Double-precision complex number for non-Hermitian operator analysis.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub const ZERO: Self = Self { re: 0.0, im: 0.0 };
    pub const ONE: Self = Self { re: 1.0, im: 0.0 };
    pub const I: Self = Self { re: 0.0, im: 1.0 };

    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    pub const fn from_real(re: f64) -> Self {
        Self { re, im: 0.0 }
    }

    pub fn from_polar(r: f64, theta: f64) -> Self {
        Self {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }

    pub fn norm_sq(&self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    pub fn norm(&self) -> f64 {
        self.norm_sq().sqrt()
    }

    pub fn arg(&self) -> f64 {
        self.im.atan2(self.re)
    }

    pub fn conj(&self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    pub fn scale(&self, s: f64) -> Self {
        Self {
            re: self.re * s,
            im: self.im * s,
        }
    }

    pub fn inv(&self) -> Self {
        let nsq = self.norm_sq();
        if nsq == 0.0 {
            Self::ZERO
        } else {
            Self {
                re: self.re / nsq,
                im: -self.im / nsq,
            }
        }
    }

    pub fn sqrt(&self) -> Self {
        let r = self.norm();
        let theta = self.arg();
        Self::from_polar(r.sqrt(), theta * 0.5)
    }

    pub fn powf(&self, p: f64) -> Self {
        let r = self.norm();
        if r == 0.0 {
            Self::ZERO
        } else {
            let theta = self.arg();
            Self::from_polar(r.powf(p), theta * p)
        }
    }

    pub fn exp(&self) -> Self {
        let r = self.re.exp();
        Self {
            re: r * self.im.cos(),
            im: r * self.im.sin(),
        }
    }
}

impl Add for Complex {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl Sub for Complex {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}

impl Mul for Complex {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}

impl Div for Complex {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        let nsq = rhs.norm_sq();
        if nsq == 0.0 {
            Self::ZERO
        } else {
            Self {
                re: (self.re * rhs.re + self.im * rhs.im) / nsq,
                im: (self.im * rhs.re - self.re * rhs.im) / nsq,
            }
        }
    }
}

impl Neg for Complex {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            re: -self.re,
            im: -self.im,
        }
    }
}

impl fmt::Display for Complex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.im >= 0.0 {
            write!(f, "{:.4} + {:.4}i", self.re, self.im)
        } else {
            write!(f, "{:.4} - {:.4}i", self.re, -self.im)
        }
    }
}

/// Exceptional Point degeneracy order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EpOrder {
    /// Order-2 Exceptional Point (square-root Puiseux branch).
    EP2 = 2,
    /// Order-3 Exceptional Point (cube-root Puiseux branch).
    EP3 = 3,
    /// Order-4 Exceptional Point (quartic-root Puiseux branch).
    EP4 = 4,
}

impl EpOrder {
    /// Dimension of the Jordan block (2, 3, or 4).
    pub fn order(&self) -> usize {
        match self {
            Self::EP2 => 2,
            Self::EP3 => 3,
            Self::EP4 => 4,
        }
    }

    /// Puiseux scaling exponent 1/N.
    pub fn puiseux_exponent(&self) -> f64 {
        1.0 / (self.order() as f64)
    }
}

impl fmt::Display for EpOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EP2 => write!(f, "EP2 (Order 2)"),
            Self::EP3 => write!(f, "EP3 (Order 3)"),
            Self::EP4 => write!(f, "EP4 (Order 4)"),
        }
    }
}

/// Riemann surface trajectory point recording perturbation and complex eigenfrequencies.
#[derive(Debug, Clone, PartialEq)]
pub struct RiemannBranchPoint {
    /// Trajectory phase angle around exceptional point in radians.
    pub theta: f64,
    /// Applied complex perturbation.
    pub epsilon: Complex,
    /// Evaluated complex eigenvalues on all sheets.
    pub eigenvalues: Vec<Complex>,
}

/// Non-Hermitian Hamiltonian with Jordan block structure and perturbation response.
#[derive(Debug, Clone, PartialEq)]
pub struct NonHermitianHamiltonian {
    /// Exceptional point degeneracy order.
    pub order: EpOrder,
    /// Coalescence eigenvalue at the exceptional point singularity.
    pub lambda_0: Complex,
    /// Inter-element Jordan coupling parameter.
    pub coupling: f64,
}

impl NonHermitianHamiltonian {
    /// Creates a new non-Hermitian Hamiltonian with unit coupling.
    pub fn new(order: EpOrder, lambda_0: Complex) -> Self {
        Self {
            order,
            lambda_0,
            coupling: 1.0,
        }
    }

    /// Creates a new non-Hermitian Hamiltonian with custom coupling strength.
    pub fn new_with_coupling(order: EpOrder, lambda_0: Complex, coupling: f64) -> Self {
        Self {
            order,
            lambda_0,
            coupling: coupling.abs().max(1e-9),
        }
    }

    /// Assembles unperturbed EP matrix H_EP = lambda_0 * I + J_N.
    pub fn matrix_ep(&self) -> Vec<Vec<Complex>> {
        let n = self.order.order();
        let mut mat = vec![vec![Complex::ZERO; n]; n];
        for i in 0..n {
            mat[i][i] = self.lambda_0;
            if i + 1 < n {
                mat[i][i + 1] = Complex::from_real(self.coupling);
            }
        }
        mat
    }

    /// Assembles perturbed matrix H(epsilon) = H_EP + epsilon * |N-1><0|.
    pub fn matrix_perturbed(&self, epsilon: Complex) -> Vec<Vec<Complex>> {
        let mut mat = self.matrix_ep();
        let n = self.order.order();
        mat[n - 1][0] = epsilon;
        mat
    }

    /// Solves complex eigenvalues for perturbation epsilon.
    ///
    /// Characteristic equation:
    ///   det(lambda * I - H(epsilon)) = (lambda - lambda_0)^N - coupling^(N-1) * epsilon = 0
    /// Roots:
    ///   lambda_k = lambda_0 + (coupling^(N-1) * epsilon)^(1/N) * exp(i * 2*pi*k / N)
    pub fn solve_eigenvalues(&self, epsilon: Complex) -> Vec<Complex> {
        let n = self.order.order();
        if epsilon.norm_sq() == 0.0 {
            return vec![self.lambda_0; n];
        }

        let eff_eps = epsilon.scale(self.coupling.powi((n as i32) - 1));
        let r = eff_eps.norm();
        let theta = eff_eps.arg();
        let r_root = r.powf(1.0 / (n as f64));

        let mut roots = Vec::with_capacity(n);
        for k in 0..n {
            let angle = (theta + 2.0 * PI * (k as f64)) / (n as f64);
            let delta = Complex::from_polar(r_root, angle);
            roots.push(self.lambda_0 + delta);
        }
        roots
    }

    /// Solves eigenvalues, right eigenvectors, and left eigenvectors.
    ///
    /// Returns:
    /// (eigenvalues, right_eigenvectors, left_eigenvectors)
    /// where eigenvectors are normalized so that ||v_R||_2 = 1 and ||v_L||_2 = 1.
    pub fn solve_eigensystem(
        &self,
        epsilon: Complex,
    ) -> (Vec<Complex>, Vec<Vec<Complex>>, Vec<Vec<Complex>>) {
        let n = self.order.order();
        let evs = self.solve_eigenvalues(epsilon);

        if epsilon.norm_sq() == 0.0 {
            // Coalesced single geometric eigenvector at EP
            let mut vr = vec![Complex::ZERO; n];
            vr[0] = Complex::ONE;
            let mut vl = vec![Complex::ZERO; n];
            vl[n - 1] = Complex::ONE;
            return (evs.clone(), vec![vr; n], vec![vl; n]);
        }

        let mut right_vecs = Vec::with_capacity(n);
        let mut left_vecs = Vec::with_capacity(n);

        for &lambda in &evs {
            let mu = lambda - self.lambda_0;

            // Right eigenvector: v_R[j] = (mu / coupling)^j
            let mut vr = Vec::with_capacity(n);
            let mut current = Complex::ONE;
            let ratio = mu.scale(1.0 / self.coupling);
            let mut norm_r_sq = 0.0;
            for _ in 0..n {
                vr.push(current);
                norm_r_sq += current.norm_sq();
                current = current * ratio;
            }
            let norm_r = norm_r_sq.sqrt().max(1e-15);
            for v in vr.iter_mut() {
                *v = v.scale(1.0 / norm_r);
            }

            // Left eigenvector: H^dagger * v_L = lambda^* * v_L
            // v_L[j] = ((mu^* / coupling))^(N - 1 - j)
            let mut vl = vec![Complex::ZERO; n];
            let ratio_conj = mu.conj().scale(1.0 / self.coupling);
            let mut current_l = Complex::ONE;
            let mut norm_l_sq = 0.0;
            for j in 0..n {
                let idx = n - 1 - j;
                vl[idx] = current_l;
                norm_l_sq += current_l.norm_sq();
                current_l = current_l * ratio_conj;
            }
            let norm_l = norm_l_sq.sqrt().max(1e-15);
            for v in vl.iter_mut() {
                *v = v.scale(1.0 / norm_l);
            }

            right_vecs.push(vr);
            left_vecs.push(vl);
        }

        (evs, right_vecs, left_vecs)
    }

    /// Evaluates maximal eigenvalue splitting Delta lambda = max |lambda_i - lambda_j|.
    pub fn eigenvalue_splitting(&self, epsilon: Complex) -> f64 {
        let evs = self.solve_eigenvalues(epsilon);
        let mut max_diff = 0.0f64;
        for i in 0..evs.len() {
            for j in (i + 1)..evs.len() {
                let diff = (evs[i] - evs[j]).norm();
                if diff > max_diff {
                    max_diff = diff;
                }
            }
        }
        max_diff
    }

    /// Evaluates Petermann excess noise factor K = 1 / |<v_L | v_R>|^2.
    ///
    /// At the exceptional point (epsilon = 0), left and right eigenvectors are orthogonal,
    /// causing the Petermann noise factor K to diverge toward infinity.
    pub fn petermann_factor(&self, epsilon: Complex) -> f64 {
        if epsilon.norm_sq() == 0.0 {
            return f64::INFINITY;
        }

        let (_evs, right_vecs, left_vecs) = self.solve_eigensystem(epsilon);
        let mut max_k = 0.0f64;

        for (vr, vl) in right_vecs.iter().zip(left_vecs.iter()) {
            let mut overlap = Complex::ZERO;
            for (&r, &l) in vr.iter().zip(vl.iter()) {
                overlap = overlap + (l.conj() * r);
            }
            let denom = overlap.norm_sq();
            let k = if denom <= 1e-18 {
                1e12
            } else {
                1.0 / denom
            };
            if k > max_k {
                max_k = k;
            }
        }

        max_k
    }

    /// Evaluates theoretical scaling constant C_N in Delta lambda = C_N * |epsilon|^(1/N).
    pub fn theoretical_scaling_constant(&self) -> f64 {
        let n = self.order.order() as f64;
        let c_exp = (n - 1.0) / n;
        2.0 * (PI / n).sin() * self.coupling.powf(c_exp)
    }

    /// Evaluates fractional sensitivity Delta lambda / |epsilon|.
    pub fn fractional_sensitivity(&self, epsilon: Complex) -> f64 {
        let eps_norm = epsilon.norm();
        if eps_norm <= 1e-15 {
            0.0
        } else {
            self.eigenvalue_splitting(epsilon) / eps_norm
        }
    }

    /// Evaluates sensitivity enhancement factor relative to Hermitian linear sensor.
    ///
    /// For a standard Hermitian sensor, Delta lambda_Hermitian = |epsilon|.
    /// At the exceptional point, Delta lambda_EP = C_N * |epsilon|^(1/N).
    /// Ratio = Delta lambda_EP / |epsilon|.
    pub fn sensitivity_enhancement(&self, epsilon: Complex) -> f64 {
        let eps_norm = epsilon.norm().max(1e-15);
        let ep_split = self.eigenvalue_splitting(epsilon);
        ep_split / eps_norm
    }

    /// Computes Riemann surface trajectory tracing complex eigenvalue branch sheets.
    ///
    /// Evaluates eigenvalues along circular trajectory epsilon(theta) = r * exp(i * theta).
    pub fn riemann_surface_trajectory(
        &self,
        radius: f64,
        num_points_per_cycle: usize,
        cycles: usize,
    ) -> Vec<RiemannBranchPoint> {
        let n_points = num_points_per_cycle.max(8) * cycles.max(1);
        let total_angle = 2.0 * PI * (cycles as f64);
        let mut trajectory = Vec::with_capacity(n_points);

        for step in 0..=n_points {
            let theta = total_angle * (step as f64) / (n_points as f64);
            let eps = Complex::from_polar(radius, theta);
            let eigenvalues = self.solve_eigenvalues(eps);
            trajectory.push(RiemannBranchPoint {
                theta,
                epsilon: eps,
                eigenvalues,
            });
        }

        trajectory
    }

    /// Computes 2D points [Re(lambda), Im(lambda)] along branch sheets for UI visualization.
    pub fn riemann_sheet_points(&self, radius: f64, num_samples: usize) -> Vec<[f64; 2]> {
        let traj = self.riemann_surface_trajectory(radius, num_samples, self.order.order());
        let mut points = Vec::new();
        for pt in traj {
            for ev in pt.eigenvalues {
                points.push([ev.re, ev.im]);
            }
        }
        points
    }
}
