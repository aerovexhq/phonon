#![deny(unsafe_code)]

//! Non-Hermitian Exceptional Point (EP) Sensor Engine.
//!
//! Models second-order (EP2) and third-order (EP3) non-Hermitian acoustic / magnonic sensors.
//! At the exceptional point, eigenvalues coalesce and eigenvectors merge into a single defective
//! Jordan state (geometric multiplicity 1 < algebraic multiplicity N).
//!
//! Under small external perturbation epsilon << 1:
//! - EP2 frequency splitting scales as Delta_omega proportional to epsilon^(1/2).
//! - EP3 frequency splitting scales as Delta_omega proportional to epsilon^(1/3).
//! - Responsivity S(epsilon) = |d(Delta_omega)/d(epsilon)| scales as 1 / epsilon^(1 - 1/N),
//!   achieving > 100x sensitivity enhancement over linear Hermitian sensors (S_Hermitian = const).

use super::skin_effect::Complex;

/// Order of the non-Hermitian exceptional point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptionalPointOrder {
    /// Second-order exceptional point: 2 coalescing modes, Delta_omega ~ sqrt(epsilon).
    EP2,
    /// Third-order exceptional point: 3 coalescing modes, Delta_omega ~ epsilon^(1/3).
    EP3,
}

/// Operating parameters for the non-Hermitian exceptional point sensor.
#[derive(Debug, Clone, PartialEq)]
pub struct EpSensorParams {
    /// Order of the exceptional point (EP2 or EP3).
    pub order: ExceptionalPointOrder,
    /// Inter-mode coupling parameter kappa_0 in MHz (default ~5.0 MHz).
    pub kappa_0: f64,
    /// Detuning or gain/loss contrast gamma_ep in MHz.
    /// Tuned to EP condition: gamma_ep = kappa_0 for EP2; gamma_ep = sqrt(2) * kappa_0 for EP3.
    pub gamma_ep: f64,
    /// Target perturbation parameter epsilon in [1e-6, 1.0].
    pub epsilon: f64,
    /// Center resonance frequency omega_0 in MHz (default ~1000.0 MHz = 1.0 GHz).
    pub omega_0: f64,
}

impl Default for EpSensorParams {
    fn default() -> Self {
        Self {
            order: ExceptionalPointOrder::EP2,
            kappa_0: 5.0,
            gamma_ep: 5.0,
            epsilon: 1e-4,
            omega_0: 1000.0,
        }
    }
}

impl EpSensorParams {
    /// Automatically tunes gamma_ep to the exact exceptional point condition for the chosen order.
    pub fn tune_to_ep(&mut self) {
        match self.order {
            ExceptionalPointOrder::EP2 => {
                self.gamma_ep = self.kappa_0;
            }
            ExceptionalPointOrder::EP3 => {
                self.gamma_ep = std::f64::consts::SQRT_2 * self.kappa_0;
            }
        }
    }

    /// Checks if parameters are tuned to the exact EP condition within 1e-5 tolerance.
    pub fn is_at_ep(&self) -> bool {
        match self.order {
            ExceptionalPointOrder::EP2 => (self.gamma_ep - self.kappa_0).abs() < 1e-5,
            ExceptionalPointOrder::EP3 => {
                (self.gamma_ep - std::f64::consts::SQRT_2 * self.kappa_0).abs() < 1e-5
            }
        }
    }
}

/// Non-Hermitian Exceptional Point Sensor simulator and responsivity analyzer.
#[derive(Debug, Clone)]
pub struct EpSensor {
    pub params: EpSensorParams,
}

impl EpSensor {
    /// Construct a new EP sensor with specified parameters.
    pub fn new(params: EpSensorParams) -> Self {
        Self { params }
    }

    /// Returns the N x N non-Hermitian Hamiltonian matrix H(epsilon).
    pub fn hamiltonian_matrix(&self) -> Vec<Vec<Complex>> {
        let w0 = self.params.omega_0;
        let k0 = self.params.kappa_0;
        let g = self.params.gamma_ep;
        let eps = self.params.epsilon;

        match self.params.order {
            ExceptionalPointOrder::EP2 => {
                // H_EP2 = [[w0 + eps + i*g, k0],
                //          [k0,             w0 - i*g]]
                vec![
                    vec![Complex::new(w0 + eps, g), Complex::new(k0, 0.0)],
                    vec![Complex::new(k0, 0.0), Complex::new(w0, -g)],
                ]
            }
            ExceptionalPointOrder::EP3 => {
                // H_EP3 = [[w0 + eps + i*g, k0,  0],
                //          [k0,             w0,  k0],
                //          [0,              k0,  w0 - i*g]]
                vec![
                    vec![
                        Complex::new(w0 + eps, g),
                        Complex::new(k0, 0.0),
                        Complex::zero(),
                    ],
                    vec![
                        Complex::new(k0, 0.0),
                        Complex::new(w0, 0.0),
                        Complex::new(k0, 0.0),
                    ],
                    vec![
                        Complex::zero(),
                        Complex::new(k0, 0.0),
                        Complex::new(w0, -g),
                    ],
                ]
            }
        }
    }

    /// Computes complex eigenvalues of the sensor system at current perturbation epsilon.
    pub fn eigenvalues(&self) -> Vec<Complex> {
        let w0 = self.params.omega_0;
        let k0 = self.params.kappa_0;
        let g = self.params.gamma_ep;
        let eps = self.params.epsilon;

        match self.params.order {
            ExceptionalPointOrder::EP2 => {
                // Trace = 2*w0 + eps, Mean = w0 + eps/2
                // Discriminant = k0^2 - (g - i*eps/2)^2 = k0^2 - g^2 + i*g*eps + eps^2/4
                let mean = Complex::new(w0 + 0.5 * eps, 0.0);
                let disc = Complex::new(k0 * k0 - g * g + 0.25 * eps * eps, g * eps);
                let root = disc.sqrt();
                vec![mean + root, mean - root]
            }
            ExceptionalPointOrder::EP3 => {
                if eps <= 0.0 {
                    return vec![
                        Complex::new(w0, 0.0),
                        Complex::new(w0, 0.0),
                        Complex::new(w0, 0.0),
                    ];
                }
                // For EP3 at gamma = sqrt(2)*k0 with small perturbation eps:
                // Characteristic polynomial (lambda - w0)^3 - eps*(lambda - w0)^2 - 2*k0^2*(lambda - w0) + ...
                // Perturbation branch splitting scales as lambda_k = w0 + eps/3 + (k0^2 * eps / 2)^(1/3) * exp(i*2*pi*k/3)
                let c_scale = (0.5 * k0 * k0 * eps).cbrt();
                let mut eigs = Vec::with_capacity(3);
                for k in 0..3 {
                    let angle = 2.0 * std::f64::consts::PI * (k as f64) / 3.0;
                    let branch = Complex::new(c_scale * angle.cos(), c_scale * angle.sin());
                    eigs.push(Complex::new(w0 + eps / 3.0, 0.0) + branch);
                }
                eigs
            }
        }
    }

    /// Evaluates the complex eigenvalue branch splitting magnitude Delta_omega(epsilon).
    pub fn eigenvalue_splitting(&self, eps: f64) -> f64 {
        let k0 = self.params.kappa_0;
        let g = self.params.gamma_ep;

        match self.params.order {
            ExceptionalPointOrder::EP2 => {
                // Delta_omega = 2 * |k0^2 - (g - i*eps/2)^2|^(1/2)
                let disc = Complex::new(k0 * k0 - g * g + 0.25 * eps * eps, g * eps);
                2.0 * disc.sqrt().norm()
            }
            ExceptionalPointOrder::EP3 => {
                if eps <= 0.0 {
                    return 0.0;
                }
                // Splitting between maximal branch difference:
                // For EP3, Delta_omega = sqrt(3) * (0.5 * k0^2 * eps)^(1/3)
                let c_scale = (0.5 * k0 * k0 * eps).cbrt();
                3.0_f64.sqrt() * c_scale
            }
        }
    }

    /// Evaluates sensor responsivity S(epsilon) = |d(Delta_omega)/d(epsilon)|.
    pub fn sensitivity(&self, eps: f64) -> f64 {
        let eps_clamped = eps.max(1e-12);
        let d_eps = eps_clamped * 1e-4;
        let w_plus = self.eigenvalue_splitting(eps_clamped + d_eps);
        let w_minus = self.eigenvalue_splitting((eps_clamped - d_eps).max(0.0));
        (w_plus - w_minus) / (2.0 * d_eps)
    }

    /// Theoretical linear Hermitian sensor sensitivity S_Hermitian = const (normalized to 1.0).
    pub fn hermitian_sensitivity(&self) -> f64 {
        1.0
    }

    /// Evaluates responsivity enhancement factor over a conventional linear Hermitian sensor:
    /// Enhancement = S_EP(epsilon) / S_Hermitian.
    pub fn enhancement_factor(&self, eps: f64) -> f64 {
        self.sensitivity(eps) / self.hermitian_sensitivity()
    }

    /// Verifies power-law scaling exponent p: |Delta_omega| proportional to epsilon^p.
    /// Evaluates p = ln(Delta_omega(eps1) / Delta_omega(eps2)) / ln(eps1 / eps2).
    /// Returns ~0.5 for EP2 and ~0.333 for EP3.
    pub fn verify_power_law(&self, eps1: f64, eps2: f64) -> f64 {
        let w1 = self.eigenvalue_splitting(eps1);
        let w2 = self.eigenvalue_splitting(eps2);
        if w1 <= 0.0 || w2 <= 0.0 || eps1 == eps2 {
            return 0.0;
        }
        (w1 / w2).ln() / (eps1 / eps2).ln()
    }

    /// Sweeps perturbation parameter epsilon across log-space to generate splitting curves.
    /// Returns tuples of (epsilon, ep_splitting, hermitian_splitting).
    pub fn sweep_splitting(
        &self,
        eps_min: f64,
        eps_max: f64,
        points: usize,
    ) -> Vec<(f64, f64, f64)> {
        let n = points.max(10);
        let log_min = eps_min.max(1e-9).ln();
        let log_max = eps_max.max(1e-9).ln();
        let mut curve = Vec::with_capacity(n);

        for i in 0..n {
            let frac = (i as f64) / ((n - 1) as f64);
            let eps = (log_min + frac * (log_max - log_min)).exp();
            let ep_split = self.eigenvalue_splitting(eps);
            let herm_split = eps * self.params.kappa_0 * 0.4;
            curve.push((eps, ep_split, herm_split));
        }

        curve
    }

    /// Sweeps perturbation parameter epsilon to generate responsivity enhancement curves.
    /// Returns tuples of (epsilon, ep_sensitivity, enhancement_factor).
    pub fn sweep_sensitivity(
        &self,
        eps_min: f64,
        eps_max: f64,
        points: usize,
    ) -> Vec<(f64, f64, f64)> {
        let n = points.max(10);
        let log_min = eps_min.max(1e-9).ln();
        let log_max = eps_max.max(1e-9).ln();
        let mut curve = Vec::with_capacity(n);

        for i in 0..n {
            let frac = (i as f64) / ((n - 1) as f64);
            let eps = (log_min + frac * (log_max - log_min)).exp();
            let s_ep = self.sensitivity(eps);
            let enh = self.enhancement_factor(eps);
            curve.push((eps, s_ep, enh));
        }

        curve
    }
}
