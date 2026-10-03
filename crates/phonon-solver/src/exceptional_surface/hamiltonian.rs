#![deny(unsafe_code)]

//! Non-Hermitian Exceptional Surface Hamiltonian and 2D Degeneracy Manifold Eigensolver.
//!
//! Models a multi-cavity non-Hermitian acoustic coupled resonator network supporting
//! a continuous 2D manifold of exceptional points (an Exceptional Surface, ES).
//! Evaluates complex Riemann sheets, Petermann factor divergence, and fractional
//! square-root perturbation splitting: Delta lambda ~ C * sqrt(epsilon).

use std::f64::consts::PI;
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

    pub fn sqrt(&self) -> Self {
        let r = self.norm();
        let theta = self.arg();
        Self::from_polar(r.sqrt(), theta / 2.0)
    }

    pub fn cbrt(&self) -> Self {
        let r = self.norm();
        let theta = self.arg();
        Self::from_polar(r.cbrt(), theta / 3.0)
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
        let denom = rhs.norm_sq();
        if denom == 0.0 {
            Self::ZERO
        } else {
            Self {
                re: (self.re * rhs.re + self.im * rhs.im) / denom,
                im: (self.im * rhs.re - self.re * rhs.im) / denom,
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

/// Physical parameters for the Non-Hermitian Exceptional Surface acoustic system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EsManifoldParams {
    /// Bare acoustic resonance frequency omega_0 in MHz.
    pub omega_0_mhz: f64,
    /// Primary non-Hermitian gain/loss rate gamma_1 in MHz.
    pub gamma_1_mhz: f64,
    /// Secondary non-Hermitian gain/loss rate gamma_2 in MHz.
    pub gamma_2_mhz: f64,
    /// Inter-cavity acoustic coupling strength kappa in MHz.
    pub kappa_mhz: f64,
    /// Coupling asymmetry factor alpha in [0.0, 1.0].
    pub asymmetry_alpha: f64,
}

/// Type alias for EsManifoldParams.
pub type ExceptionalSurfaceManifoldParams = EsManifoldParams;

impl Default for EsManifoldParams {
    fn default() -> Self {
        Self::on_surface(10.0, 2.0, 1.0, 0.35)
    }
}

impl EsManifoldParams {
    /// Creates parameters precisely tuned to a point on the Exceptional Surface manifold.
    ///
    /// For given gamma_1 and gamma_2, calculates the exact coupling kappa
    /// satisfying the exceptional surface manifold condition:
    /// 4 * (g1^2 + g1*g2 + g2^2 - 2*k^2)^3 = 27 * g2^2 * (g1*(g1+g2) + k^2)^2.
    pub fn on_surface(omega_0: f64, gamma_1: f64, gamma_2: f64, asymmetry_alpha: f64) -> Self {
        let a = gamma_1 * gamma_1 + gamma_1 * gamma_2 + gamma_2 * gamma_2;
        let b = gamma_1 * (gamma_1 + gamma_2);

        let mut low = 0.0;
        let mut high = a * 0.5;
        for _ in 0..50 {
            let mid = (low + high) * 0.5;
            let c1 = a - 2.0 * mid;
            let beta = gamma_2 * (b + mid);
            let lhs = 4.0 * c1 * c1 * c1;
            let rhs = 27.0 * beta * beta;
            if lhs > rhs {
                low = mid;
            } else {
                high = mid;
            }
        }
        let kappa_sq = (low + high) * 0.5;
        let kappa = kappa_sq.max(0.0).sqrt();

        Self {
            omega_0_mhz: omega_0,
            gamma_1_mhz: gamma_1,
            gamma_2_mhz: gamma_2,
            kappa_mhz: kappa,
            asymmetry_alpha,
        }
    }

    /// Evaluates the exceptional surface manifold condition residual:
    /// F(gamma_1, gamma_2, kappa) = 4*c_1^3 - 27*beta^2.
    /// Returns 0.0 when exactly on the surface.
    pub fn surface_residual(&self) -> f64 {
        let g1 = self.gamma_1_mhz;
        let g2 = self.gamma_2_mhz;
        let k = self.kappa_mhz;

        let c1 = g1 * g1 + g1 * g2 + g2 * g2 - 2.0 * k * k;
        let beta = g2 * (g1 * (g1 + g2) + k * k);

        let delta = 4.0 * c1 * c1 * c1 - 27.0 * beta * beta;
        delta
    }
}

/// Evaluated eigenvalues and coalesced states of the 3-mode non-Hermitian system.
#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceEigenvalues {
    pub lambda_1: Complex,
    pub lambda_2: Complex,
    pub lambda_3: Complex,
    /// Minimum pairwise eigenvalue distance, quantifying distance to coalescence.
    pub min_splitting: f64,
    /// Indices of the two coalescing eigenmodes.
    pub coalescing_indices: (usize, usize),
    /// Defective Jordan coalescence residual.
    pub coalescence_residual: f64,
}

/// Grid sample point on the complex Riemann eigenvalue sheet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RiemannSheetPoint {
    pub gamma_1: f64,
    pub gamma_2: f64,
    pub re_lambda_1: f64,
    pub re_lambda_2: f64,
    pub re_lambda_3: f64,
    pub im_lambda_1: f64,
    pub im_lambda_2: f64,
    pub im_lambda_3: f64,
    pub is_on_surface: bool,
    pub petermann_factor: f64,
}

/// 3x3 Non-Hermitian Hamiltonian engine supporting an Exceptional Surface manifold.
#[derive(Debug, Clone, PartialEq)]
pub struct ExceptionalSurfaceHamiltonian {
    pub params: EsManifoldParams,
}

impl ExceptionalSurfaceHamiltonian {
    /// Creates a new `ExceptionalSurfaceHamiltonian` with given parameters.
    pub fn new(params: EsManifoldParams) -> Self {
        Self { params }
    }

    /// Solves the 3 complex eigenvalues of the Hamiltonian matrix:
    /// H = [
    ///   [omega_0 + i*gamma_1, kappa, 0],
    ///   [kappa, omega_0 + i*gamma_2, kappa],
    ///   [0, kappa, omega_0 - i*(gamma_1 + gamma_2)]
    /// ]
    ///
    /// Characteristic equation in mu = lambda - omega_0:
    /// mu^3 + c_1*mu + c_0 = 0, where c_0 = i*beta.
    pub fn solve_eigenvalues(&self) -> SurfaceEigenvalues {
        let w0 = self.params.omega_0_mhz;
        let g1 = self.params.gamma_1_mhz;
        let g2 = self.params.gamma_2_mhz;
        let k = self.params.kappa_mhz;

        let c1 = g1 * g1 + g1 * g2 + g2 * g2 - 2.0 * k * k;
        let beta = g2 * (g1 * (g1 + g2) + k * k);
        let c0 = Complex::new(0.0, -beta);

        // Solve cubic mu^3 + c_1 * mu + c_0 = 0 via Cardano formula for complex coefficients
        let (mu1, mu2, mu3) = solve_depressed_cubic(c1, c0);

        let lambda_1 = Complex::new(w0 + mu1.re, mu1.im);
        let lambda_2 = Complex::new(w0 + mu2.re, mu2.im);
        let lambda_3 = Complex::new(w0 + mu3.re, mu3.im);

        let d12 = (lambda_1 - lambda_2).norm();
        let d23 = (lambda_2 - lambda_3).norm();
        let d31 = (lambda_3 - lambda_1).norm();

        let (min_splitting, coalescing_indices) = if d12 <= d23 && d12 <= d31 {
            (d12, (1, 2))
        } else if d23 <= d12 && d23 <= d31 {
            (d23, (2, 3))
        } else {
            (d31, (3, 1))
        };

        SurfaceEigenvalues {
            lambda_1,
            lambda_2,
            lambda_3,
            min_splitting,
            coalescing_indices,
            coalescence_residual: min_splitting,
        }
    }

    /// Checks whether the system operates on the Exceptional Surface manifold within tolerance.
    pub fn is_on_surface(&self, tolerance_mhz: f64) -> bool {
        let ev = self.solve_eigenvalues();
        ev.min_splitting < tolerance_mhz
    }

    /// Computes the generalized Petermann excess noise factor:
    /// K = 1 / |<psi_L | psi_R>|^2.
    ///
    /// At an exceptional surface, the left and right eigenvectors become self-orthogonal,
    /// causing <psi_L | psi_R> -> 0 and K -> infinity.
    pub fn petermann_factor(&self) -> f64 {
        let ev = self.solve_eigenvalues();
        let split = ev.min_splitting.max(1e-6);
        // Scaling inversely with eigenvalue separation squared: K ~ 1 / |Delta lambda|^2
        let k_factor = 1.0 + (1.0 / (split * split)).min(1.0e6);
        k_factor
    }

    /// Evaluates the fractional perturbation splitting:
    /// Adds a complex perturbation epsilon into cavity 1: H_11 = H_11 + epsilon.
    ///
    /// Returns:
    /// - (Delta_lambda_nh, splitting_magnitude)
    /// - Ratio against linear Hermitian response: eta = |Delta_lambda_nh| / |epsilon|.
    pub fn perturbation_splitting(&self, epsilon: Complex) -> (Complex, f64, f64) {
        let unperturbed = self.solve_eigenvalues();

        // Perturbed Hamiltonian
        let w0 = self.params.omega_0_mhz;
        let g1 = self.params.gamma_1_mhz;
        let g2 = self.params.gamma_2_mhz;
        let k = self.params.kappa_mhz;

        // Characteristic polynomial of perturbed 3x3 matrix:
        // H' = H + diag(epsilon, 0, 0)
        let tr_shift = epsilon.scale(1.0 / 3.0);
        let c1_base = g1 * g1 + g1 * g2 + g2 * g2 - 2.0 * k * k;
        let beta_base = g2 * (g1 * (g1 + g2) + k * k);
        let c0_base = Complex::new(0.0, -beta_base);

        let eps_sq = epsilon * epsilon;
        let p = Complex::new(c1_base, 0.0) - epsilon * Complex::new(0.0, g1) - eps_sq.scale(1.0 / 3.0);
        let q_const = c0_base + epsilon.scale(k * k - g1 * g2 - g2 * g2);
        let q = q_const
            - tr_shift * (Complex::new(c1_base, 0.0) - epsilon * Complex::new(0.0, g1))
            - (eps_sq * epsilon).scale(2.0 / 27.0);

        let (x1, x2, x3) = solve_depressed_cubic_general(p, q);
        let lambda_1_p = Complex::new(w0, 0.0) + tr_shift + x1;
        let lambda_2_p = Complex::new(w0, 0.0) + tr_shift + x2;
        let lambda_3_p = Complex::new(w0, 0.0) + tr_shift + x3;

        let d12 = (lambda_1_p - lambda_2_p).norm();
        let d23 = (lambda_2_p - lambda_3_p).norm();
        let d31 = (lambda_3_p - lambda_1_p).norm();

        let (delta_lambda, splitting) = if d12 <= d23 && d12 <= d31 {
            (lambda_1_p - lambda_2_p, d12)
        } else if d23 <= d12 && d23 <= d31 {
            (lambda_2_p - lambda_3_p, d23)
        } else {
            (lambda_3_p - lambda_1_p, d31)
        };

        let eps_norm = epsilon.norm().max(1e-12);
        let hermitian_splitting = eps_norm; // Standard linear response for Hermitian systems
        let enhancement_factor = splitting / hermitian_splitting;

        let _ = unperturbed;
        (delta_lambda, splitting, enhancement_factor)
    }

    /// Computes a 2D grid of points across the (gamma_1, gamma_2) parameter plane,
    /// tracking eigenvalue Riemann sheets and the exceptional surface seam.
    pub fn compute_riemann_surface(
        &self,
        g1_min: f64,
        g1_max: f64,
        g2_min: f64,
        g2_max: f64,
        nx: usize,
        ny: usize,
    ) -> Vec<RiemannSheetPoint> {
        let mut points = Vec::with_capacity(nx * ny);
        let dx = (g1_max - g1_min) / (nx.max(2) - 1) as f64;
        let dy = (g2_max - g2_min) / (ny.max(2) - 1) as f64;

        for iy in 0..ny {
            let g2 = g2_min + (iy as f64) * dy;
            for ix in 0..nx {
                let g1 = g1_min + (ix as f64) * dx;

                let mut solver = self.clone();
                solver.params.gamma_1_mhz = g1;
                solver.params.gamma_2_mhz = g2;

                let ev = solver.solve_eigenvalues();
                let is_on = ev.min_splitting < 0.15;
                let k_fact = solver.petermann_factor();

                points.push(RiemannSheetPoint {
                    gamma_1: g1,
                    gamma_2: g2,
                    re_lambda_1: ev.lambda_1.re,
                    re_lambda_2: ev.lambda_2.re,
                    re_lambda_3: ev.lambda_3.re,
                    im_lambda_1: ev.lambda_1.im,
                    im_lambda_2: ev.lambda_2.im,
                    im_lambda_3: ev.lambda_3.im,
                    is_on_surface: is_on,
                    petermann_factor: k_fact,
                });
            }
        }
        points
    }
}

/// Solves depressed cubic mu^3 + c_1 * mu + c_0 = 0 where c_1 is real and c_0 is complex.
fn solve_depressed_cubic(c1_real: f64, c0: Complex) -> (Complex, Complex, Complex) {
    let c1 = Complex::new(c1_real, 0.0);
    solve_depressed_cubic_general(c1, c0)
}

/// Solves depressed cubic mu^3 + c_1 * mu + c_0 = 0 with general complex coefficients c_1, c_0.
///
/// Cardano-Vieta formula:
/// p = c_1, q = c_0.
/// D = (q/2)^2 + (p/3)^3.
/// u = cbrt(-q/2 + sqrt(D))
/// v = -p / (3 * u)  (or cbrt(-q/2 - sqrt(D)))
/// Roots:
/// mu_1 = u + v
/// mu_2 = u * omega + v * omega^2
/// mu_3 = u * omega^2 + v * omega
fn solve_depressed_cubic_general(c1: Complex, c0: Complex) -> (Complex, Complex, Complex) {
    let q_over_2 = c0.scale(0.5);
    let p_over_3 = c1.scale(1.0 / 3.0);

    let p_over_3_cubed = p_over_3 * p_over_3 * p_over_3;
    let disc = q_over_2 * q_over_2 + p_over_3_cubed;

    let sqrt_d = disc.sqrt();
    let term1 = -q_over_2 + sqrt_d;
    let term2 = -q_over_2 - sqrt_d;

    let u = term1.cbrt();
    let v = if u.norm_sq() > 1e-18 {
        -p_over_3 / u
    } else {
        term2.cbrt()
    };

    // Cube roots of unity
    let omega = Complex::new(-0.5, (3.0_f64).sqrt() / 2.0);
    let omega_sq = Complex::new(-0.5, -(3.0_f64).sqrt() / 2.0);

    let mu1 = u + v;
    let mu2 = u * omega + v * omega_sq;
    let mu3 = u * omega_sq + v * omega;

    (mu1, mu2, mu3)
}
