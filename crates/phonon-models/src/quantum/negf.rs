//! Non-Equilibrium Green's Function (NEGF) quantum transport solver.
//!
//! Solves the 1D Schrödinger-Poisson open-boundary quantum transport equations
//! via retarded Green's functions $G(E)$, contact self-energies $\Sigma_{1, 2}(E)$,
//! and the Landauer-Büttiker transmission formula $\mathcal{T}(E) = \text{Tr}(\Gamma_1 G \Gamma_2 G^\dagger)$.

use phonon_core::{ELEMENTARY_CHARGE, H_BAR, PLANCK_CONSTANT};

/// Complex number structure for pure safe-Rust Green's function linear algebra.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub const ZERO: Self = Self { re: 0.0, im: 0.0 };
    pub const ONE: Self = Self { re: 1.0, im: 0.0 };
    pub const I: Self = Self { re: 0.0, im: 1.0 };

    #[inline(always)]
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    #[inline(always)]
    pub fn norm_sq(&self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    #[inline(always)]
    pub fn conj(&self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    #[inline(always)]
    pub fn add(&self, other: Self) -> Self {
        Self {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }

    #[inline(always)]
    pub fn sub(&self, other: Self) -> Self {
        Self {
            re: self.re - other.re,
            im: self.im - other.im,
        }
    }

    #[inline(always)]
    pub fn mul(&self, other: Self) -> Self {
        Self {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }

    #[inline(always)]
    pub fn div(&self, other: Self) -> Self {
        let d = other.norm_sq().max(1e-30);
        Self {
            re: (self.re * other.re + self.im * other.im) / d,
            im: (self.im * other.re - self.re * other.im) / d,
        }
    }

    #[inline(always)]
    pub fn scale(&self, s: f64) -> Self {
        Self {
            re: self.re * s,
            im: self.im * s,
        }
    }

    #[inline(always)]
    pub fn abs(&self) -> f64 {
        self.norm_sq().sqrt()
    }
}

impl std::ops::Add for Complex {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        Self {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl std::ops::Sub for Complex {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        Self {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}

impl std::ops::Mul for Complex {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: Self) -> Self {
        Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}

impl std::ops::Mul<f64> for Complex {
    type Output = Self;
    #[inline(always)]
    fn mul(self, rhs: f64) -> Self {
        self.scale(rhs)
    }
}

impl std::ops::Mul<Complex> for f64 {
    type Output = Complex;
    #[inline(always)]
    fn mul(self, rhs: Complex) -> Complex {
        rhs.scale(self)
    }
}

impl std::ops::Div for Complex {
    type Output = Self;
    #[inline(always)]
    fn div(self, rhs: Self) -> Self {
        let d = rhs.norm_sq().max(1e-30);
        Self {
            re: (self.re * rhs.re + self.im * rhs.im) / d,
            im: (self.im * rhs.re - self.re * rhs.im) / d,
        }
    }
}

impl std::ops::Neg for Complex {
    type Output = Self;
    #[inline(always)]
    fn neg(self) -> Self {
        Self {
            re: -self.re,
            im: -self.im,
        }
    }
}

/// 1D Discretized tight-binding / finite-difference Hamiltonian grid for a quantum wire/channel.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumChannel1D {
    /// Number of spatial mesh points $N$.
    pub num_points: usize,
    /// Spatial grid spacing $\Delta x$ in meters ($m$).
    pub grid_spacing_m: f64,
    /// Effective carrier mass $m^*$ in units of free electron mass $m_0$.
    pub effective_mass: f64,
    /// On-site electrostatic potential energy profile $U(x_i)$ in electron-volts ($eV$).
    pub potential_ev: Vec<f64>,
}

impl QuantumChannel1D {
    /// Creates a 1D quantum channel of length $L = (N - 1) \Delta x$.
    pub fn new(num_points: usize, length_m: f64, effective_mass: f64) -> Self {
        assert!(num_points >= 2, "Channel must have at least 2 grid points");
        let grid_spacing = length_m / ((num_points - 1) as f64);
        Self {
            num_points,
            grid_spacing_m: grid_spacing,
            effective_mass,
            potential_ev: vec![0.0; num_points],
        }
    }

    /// Sets the on-site electrostatic potential energy profile across the channel.
    pub fn with_potential(mut self, potential_ev: Vec<f64>) -> Self {
        assert_eq!(potential_ev.len(), self.num_points);
        self.potential_ev = potential_ev;
        self
    }

    /// Coupling energy (nearest-neighbor hopping parameter) $t_0 = \frac{\hbar^2}{2 m^* \Delta x^2}$ in Joules.
    pub fn hopping_energy_joules(&self) -> f64 {
        let m0 = 9.109_383_701_5e-31;
        let m_eff = self.effective_mass * m0;
        let dx = self.grid_spacing_m;
        (H_BAR * H_BAR) / (2.0 * m_eff * dx * dx)
    }

    /// Coupling energy in electron-volts ($eV$).
    pub fn hopping_energy_ev(&self) -> f64 {
        self.hopping_energy_joules() / ELEMENTARY_CHARGE
    }

    /// Computes the semi-infinite lead surface self-energy $\Sigma(E)$ in $eV$
    /// for contact lead with on-site potential $U_{lead}$.
    pub fn contact_self_energy(&self, energy_ev: f64, lead_potential_ev: f64) -> Complex {
        let t0 = self.hopping_energy_ev();
        let e_rel = energy_ev - lead_potential_ev;

        // Dispersion: E_rel = 2 * t0 * (1 - cos(k * dx))
        // => cos(k * dx) = 1 - E_rel / (2 * t0)
        let cos_kdx = 1.0 - e_rel / (2.0 * t0);

        if cos_kdx.abs() <= 1.0 {
            // Propagating wave mode inside the band: sin(k * dx) > 0
            let sin_kdx = (1.0 - cos_kdx * cos_kdx).sqrt();
            // exp(i * k * dx) = cos(k*dx) + i * sin(k*dx)
            // Sigma = -t0 * exp(i * k * dx)
            Complex::new(-t0 * cos_kdx, -t0 * sin_kdx)
        } else if cos_kdx > 1.0 {
            // Decaying evanescent mode below the band: exp(-kappa * dx)
            let exp_minus_kappa = cos_kdx - (cos_kdx * cos_kdx - 1.0).sqrt();
            Complex::new(-t0 * exp_minus_kappa, 0.0)
        } else {
            // Evanescent mode above the band: -exp(-kappa * dx)
            let exp_minus_kappa = -cos_kdx - (cos_kdx * cos_kdx - 1.0).sqrt();
            Complex::new(t0 * exp_minus_kappa, 0.0)
        }
    }

    /// Computes the electron transmission probability $\mathcal{T}(E)$ at energy $E$ (in $eV$)
    /// using the Recursive Green's Function (RGF) algorithm for tridiagonal systems.
    pub fn transmission(&self, energy_ev: f64, v_ds: f64) -> f64 {
        let n = self.num_points;
        let t0 = self.hopping_energy_ev();

        // Source contact at x = 0 (grounded / ref lead), Drain contact at x = N - 1 (biased by V_ds)
        let u_s = self.potential_ev[0];
        let u_d = self.potential_ev[n - 1] - v_ds;

        let sigma_1 = self.contact_self_energy(energy_ev, u_s);
        let sigma_2 = self.contact_self_energy(energy_ev, u_d);

        // Broadening functions: Gamma = i * (Sigma - Sigma^\dagger) = -2 * Im(Sigma)
        let gamma_1 = -2.0 * sigma_1.im;
        let gamma_2 = -2.0 * sigma_2.im;

        if gamma_1 <= 1e-15 || gamma_2 <= 1e-15 {
            // Evanescent mode in one or both leads -> zero transmission
            return 0.0;
        }

        // Tridiagonal elements of matrix A = (E + i*eta)*I - H - Sigma_1 - Sigma_2
        // Diagonal: alpha_i = (E + i*eta) - (2*t0 + U_i) - delta_i1 * Sigma_1 - delta_iN * Sigma_2
        // Off-diagonal: beta = t0
        let eta = 1e-8; // Infinitesimal imaginary energy broadening
        let e_c = Complex::new(energy_ev, eta);

        // Solve for the end-to-end Green's function element G_{1, N} via recursive Gauss elimination
        // Forward elimination of tridiagonal system:
        let mut d = vec![Complex::ZERO; n];

        let alpha_0 = e_c
            .sub(Complex::new(2.0 * t0 + self.potential_ev[0], 0.0))
            .sub(sigma_1);
        d[0] = alpha_0;

        for i in 1..n {
            let u_i = if i == n - 1 {
                self.potential_ev[i] - v_ds
            } else {
                self.potential_ev[i]
            };
            let mut alpha_i = e_c.sub(Complex::new(2.0 * t0 + u_i, 0.0));
            if i == n - 1 {
                alpha_i = alpha_i.sub(sigma_2);
            }

            // d_i = alpha_i - (t0^2) / d_{i-1}
            let off = Complex::new(t0 * t0, 0.0);
            let prev_term = off.div(d[i - 1]);
            d[i] = alpha_i.sub(prev_term);
        }

        // G_{1, N} = product_{i=0}^{N-2} (-t0) / product_{i=0}^{N-1} d_i
        // The magnitude squared |G_{1, N}|^2 can be accumulated numerically stably:
        let mut ln_mag_sq = 2.0 * (n as f64 - 1.0) * t0.ln();
        for item in d.iter().take(n) {
            ln_mag_sq -= item.norm_sq().ln();
        }

        let g1n_sq = ln_mag_sq.exp();

        // Caroli / Fisher-Lee formula: T(E) = Tr(Gamma_1 * G * Gamma_2 * G^\dagger)
        // For 1D single-mode: T(E) = Gamma_1 * Gamma_2 * |G_{1, N}|^2
        let trans = gamma_1 * gamma_2 * g1n_sq;
        trans.clamp(0.0, 1.0)
    }

    /// Evaluates total ballistic terminal current $I_{DS}$ (in Amperes) at temperature $T$
    /// via Landauer-Büttiker numerical energy quadrature:
    /// $$I_{DS} = \frac{2q}{h} \int_{E_{min}}^{E_{max}} \mathcal{T}(E) \left[ f_S(E) - f_D(E) \right] dE$$
    pub fn calculate_current(&self, v_ds: f64, temp_k: f64, num_energy_steps: usize) -> f64 {
        let kb_ev = 8.617_333_262e-5;
        let kt = (kb_ev * temp_k).max(1e-5);

        let mu_s: f64 = 0.0; // Source Fermi level reference at 0 eV
        let mu_d: f64 = -v_ds; // Drain Fermi level shifted by -q * V_ds

        // Energy integration bounds: 8 * k_B * T beyond chemical potentials
        let e_min = (mu_s.min(mu_d) - 8.0 * kt).max(-2.0);
        let e_max = (mu_s.max(mu_d) + 8.0 * kt).min(5.0);

        let de = (e_max - e_min) / (num_energy_steps as f64);
        let mut integral = 0.0;

        for step in 0..num_energy_steps {
            let e = e_min + (step as f64 + 0.5) * de;
            let trans = self.transmission(e, v_ds);

            // Fermi-Dirac occupation factors
            let f_s = 1.0 / (1.0 + ((e - mu_s) / kt).clamp(-50.0, 50.0).exp());
            let f_d = 1.0 / (1.0 + ((e - mu_d) / kt).clamp(-50.0, 50.0).exp());

            integral += trans * (f_s - f_d) * de;
        }

        // Constant prefactor: 2 * q / h in Amperes / eV
        // Since integral is in eV, multiply by q to convert to Joules:
        // (2 * q / h) * (integral * q) = (2 * q^2 / h) * integral
        let prefactor = 2.0 * ELEMENTARY_CHARGE * ELEMENTARY_CHARGE / PLANCK_CONSTANT;
        prefactor * integral
    }
}
