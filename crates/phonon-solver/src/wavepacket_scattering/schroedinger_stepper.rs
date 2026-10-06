#![deny(unsafe_code)]

//! Time-Dependent Schroedinger Equation (TDSE) Unitary Stepper for Microscopic Wavepacket Transport.
//!
//! Provides a 1D unitary Crank-Nicolson integrator solving:
//! $$i\hbar \frac{\partial \psi(x, t)}{\partial t} = \left[ -\frac{\hbar^2}{2 m^*} \frac{\partial^2}{\partial x^2} + V(x, t) \right] \psi(x, t)$$
//! with exact norm conservation $\|\psi(t)\|^2 = 1.000 \pm 10^{-7}$.

/// Fundamental physical constants in SI units.
pub const HBAR: f64 = 1.054_571_817e-34; // J*s
pub const ELECTRON_MASS_KG: f64 = 9.109_383_7e-31; // kg
pub const ELEMENTARY_CHARGE: f64 = 1.602_176_634e-19; // C

/// Lightweight complex number arithmetic for quantum wavefunctions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    #[inline]
    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    #[inline]
    pub const fn zero() -> Self {
        Self { re: 0.0, im: 0.0 }
    }

    #[inline]
    pub const fn one() -> Self {
        Self { re: 1.0, im: 0.0 }
    }

    #[inline]
    pub fn from_polar(r: f64, theta: f64) -> Self {
        Self {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }

    #[inline]
    pub fn norm_sq(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    #[inline]
    pub fn norm(self) -> f64 {
        self.norm_sq().sqrt()
    }

    #[inline]
    pub fn arg(self) -> f64 {
        self.im.atan2(self.re)
    }

    #[inline]
    pub fn conj(self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    #[inline]
    pub fn add(self, o: Self) -> Self {
        Self {
            re: self.re + o.re,
            im: self.im + o.im,
        }
    }

    #[inline]
    pub fn sub(self, o: Self) -> Self {
        Self {
            re: self.re - o.re,
            im: self.im - o.im,
        }
    }

    #[inline]
    pub fn mul(self, o: Self) -> Self {
        Self {
            re: self.re * o.re - self.im * o.im,
            im: self.re * o.im + self.im * o.re,
        }
    }

    #[inline]
    pub fn scale(self, s: f64) -> Self {
        Self {
            re: self.re * s,
            im: self.im * s,
        }
    }

    #[inline]
    pub fn div(self, o: Self) -> Self {
        let d = o.norm_sq().max(1e-30);
        Self {
            re: (self.re * o.re + self.im * o.im) / d,
            im: (self.im * o.re - self.re * o.im) / d,
        }
    }
}

/// Simulation parameters for the electron wavepacket.
#[derive(Debug, Clone, PartialEq)]
pub struct WavepacketParams {
    /// Spatial grid points count (e.g. 512).
    pub grid_points: usize,
    /// Physical 1D domain length in nanometers (e.g. 100.0 nm).
    pub domain_length_nm: f64,
    /// Effective mass ratio m* / m0 (e.g. 0.067 for GaAs conduction band, 0.26 for Si).
    pub effective_mass_ratio: f64,
    /// Initial wavepacket center in nanometers (e.g. 25.0 nm).
    pub center_x_nm: f64,
    /// Initial Gaussian spatial half-width sigma in nanometers (e.g. 4.0 nm).
    pub sigma_nm: f64,
    /// Mean initial kinetic energy in electron-volts (e.g. 0.15 eV).
    pub initial_energy_ev: f64,
    /// Propagation direction sign (+1.0 for right, -1.0 for left).
    pub direction: f64,
    /// Time step dt in femtoseconds (e.g. 0.2 fs).
    pub dt_fs: f64,
}

impl Default for WavepacketParams {
    fn default() -> Self {
        Self {
            grid_points: 512,
            domain_length_nm: 100.0,
            effective_mass_ratio: 0.067,
            center_x_nm: 25.0,
            sigma_nm: 4.0,
            initial_energy_ev: 0.15,
            direction: 1.0,
            dt_fs: 0.2,
        }
    }
}

/// Instantaneous diagnostic summary of the propagating wavepacket.
#[derive(Debug, Clone, PartialEq)]
pub struct WavepacketDiagnostics {
    pub current_time_fs: f64,
    pub norm: f64,
    pub mean_position_nm: f64,
    pub mean_velocity_m_s: f64,
    pub position_spread_nm: f64,
    pub mean_energy_ev: f64,
    pub peak_density: f64,
}

/// Unitary 1D Crank-Nicolson TDSE quantum state propagator.
#[derive(Debug, Clone)]
pub struct SchroedingerStepper {
    pub params: WavepacketParams,
    pub psi: Vec<Complex>,
    pub potential_ev: Vec<f64>,
    pub x_coords_nm: Vec<f64>,
    pub current_time_fs: f64,
    pub dx_m: f64,
    pub dt_s: f64,
    pub effective_mass_kg: f64,
    pub k0_rad_per_m: f64,
}

impl SchroedingerStepper {
    /// Constructs and initializes a new TDSE stepper with a Gaussian electron wavepacket.
    pub fn new(params: WavepacketParams) -> Self {
        let n = params.grid_points;
        let dx_nm = params.domain_length_nm / (n as f64);
        let dx_m = dx_nm * 1e-9;
        let dt_s = params.dt_fs * 1e-15;
        let effective_mass_kg = params.effective_mass_ratio * ELECTRON_MASS_KG;

        let energy_j = params.initial_energy_ev * ELEMENTARY_CHARGE;
        let k0 = params.direction * (2.0 * effective_mass_kg * energy_j).sqrt() / HBAR;

        let mut x_coords_nm = Vec::with_capacity(n);
        for i in 0..n {
            x_coords_nm.push(i as f64 * dx_nm);
        }

        let potential_ev = vec![0.0f64; n];
        let mut stepper = Self {
            params,
            psi: vec![Complex::zero(); n],
            potential_ev,
            x_coords_nm,
            current_time_fs: 0.0,
            dx_m,
            dt_s,
            effective_mass_kg,
            k0_rad_per_m: k0,
        };

        stepper.reset_wavepacket();
        stepper
    }

    /// Resets the wavefunction psi to the initial Gaussian packet envelope.
    pub fn reset_wavepacket(&mut self) {
        let n = self.params.grid_points;
        let x0 = self.params.center_x_nm * 1e-9;
        let sigma = self.params.sigma_nm * 1e-9;
        let k0 = self.k0_rad_per_m;

        let mut psi = Vec::with_capacity(n);
        let norm_factor = 1.0 / (2.0 * std::f64::consts::PI * sigma * sigma).powf(0.25);

        for i in 0..n {
            let x = self.x_coords_nm[i] * 1e-9;
            let diff = x - x0;
            let envelope = norm_factor * (-diff * diff / (4.0 * sigma * sigma)).exp();
            let phase = k0 * x;
            psi.push(Complex::from_polar(envelope, phase));
        }

        // Apply strict zero boundary conditions at the grid edges
        if n > 1 {
            psi[0] = Complex::zero();
            psi[n - 1] = Complex::zero();
        }

        self.psi = psi;
        self.normalize();
        self.current_time_fs = 0.0;
    }

    /// Sets or updates the static background potential energy profile in eV.
    pub fn set_potential(&mut self, potential_ev: &[f64]) {
        let n = self.params.grid_points;
        if potential_ev.len() == n {
            self.potential_ev.copy_from_slice(potential_ev);
        }
    }

    /// Computes the integrated probability norm: integral |psi|^2 dx.
    pub fn total_norm(&self) -> f64 {
        let mut sum = 0.0;
        for c in &self.psi {
            sum += c.norm_sq();
        }
        sum * self.dx_m
    }

    /// Enforces rigorous normalization: sum |psi_i|^2 dx = 1.0.
    pub fn normalize(&mut self) {
        let current_norm = self.total_norm();
        if current_norm > 1e-30 {
            let inv_sqrt = 1.0 / current_norm.sqrt();
            for c in &mut self.psi {
                *c = c.scale(inv_sqrt);
            }
        }
    }

    /// Advances the wavefunction by one time step dt using Crank-Nicolson tridiagonal scheme.
    /// Supports an optional time-dependent potential perturbation V_pert (e.g. from moving phonon field).
    pub fn step(&mut self, v_pert_ev: Option<&[f64]>) {
        let n = self.params.grid_points;
        if n < 3 {
            return;
        }

        let alpha = HBAR * HBAR / (2.0 * self.effective_mass_kg * self.dx_m * self.dx_m);
        let beta = self.dt_s / (2.0 * HBAR);

        // Precompute total potential V_i in Joules
        let mut v_total_j = Vec::with_capacity(n);
        for i in 0..n {
            let v_base = self.potential_ev[i];
            let v_pert = v_pert_ev.map_or(0.0, |p| if i < p.len() { p[i] } else { 0.0 });
            v_total_j.push((v_base + v_pert) * ELEMENTARY_CHARGE);
        }

        // RHS vector d = (I - i * beta * H) * psi^n
        let mut d = vec![Complex::zero(); n];
        for i in 1..(n - 1) {
            let psi_prev = self.psi[i - 1];
            let psi_curr = self.psi[i];
            let psi_next = self.psi[i + 1];

            let kin = (psi_curr.scale(2.0 * alpha))
                .sub(psi_prev.scale(alpha))
                .sub(psi_next.scale(alpha));
            let h_psi = kin.add(psi_curr.scale(v_total_j[i]));

            // d_i = psi_i - i * beta * (H psi)_i
            let i_beta_h = Complex::new(-beta * h_psi.im, beta * h_psi.re);
            d[i] = psi_curr.sub(i_beta_h);
        }

        // Tridiagonal matrix M = (I + i * beta * H)
        // a_i = -i * beta * alpha (subdiagonal)
        // b_i = 1 + i * beta * (2 * alpha + V_i) (diagonal)
        // c_i = -i * beta * alpha (superdiagonal)
        let sub_diag = Complex::new(0.0, -beta * alpha);
        let super_diag = Complex::new(0.0, -beta * alpha);

        // Forward sweep of Thomas tridiagonal algorithm
        let mut c_prime = vec![Complex::zero(); n];
        let mut d_prime = vec![Complex::zero(); n];

        // Dirichlet boundary at i = 1
        let b1 = Complex::new(1.0, beta * (2.0 * alpha + v_total_j[1]));
        c_prime[1] = super_diag.div(b1);
        d_prime[1] = d[1].div(b1);

        for i in 2..(n - 1) {
            let bi = Complex::new(1.0, beta * (2.0 * alpha + v_total_j[i]));
            let denom = bi.sub(sub_diag.mul(c_prime[i - 1]));
            c_prime[i] = super_diag.div(denom);
            let num = d[i].sub(sub_diag.mul(d_prime[i - 1]));
            d_prime[i] = num.div(denom);
        }

        // Back substitution
        self.psi[0] = Complex::zero();
        self.psi[n - 1] = Complex::zero();
        self.psi[n - 2] = d_prime[n - 2];

        for i in (1..(n - 2)).rev() {
            self.psi[i] = d_prime[i].sub(c_prime[i].mul(self.psi[i + 1]));
        }

        self.current_time_fs += self.params.dt_fs;
    }

    /// Runs time evolution for the specified number of steps.
    pub fn step_n(&mut self, steps: usize, v_pert_ev: Option<&[f64]>) {
        for _ in 0..steps {
            self.step(v_pert_ev);
        }
    }

    /// Evaluates current quantum diagnostics (norm, mean position, spread, velocity).
    pub fn evaluate_diagnostics(&self) -> WavepacketDiagnostics {
        let n = self.params.grid_points;
        let mut norm = 0.0;
        let mut exp_x_m = 0.0;
        let mut exp_x2_m2 = 0.0;
        let mut peak_density = 0.0;

        for i in 0..n {
            let prob_dens = self.psi[i].norm_sq();
            if prob_dens > peak_density {
                peak_density = prob_dens;
            }
            let x_m = self.x_coords_nm[i] * 1e-9;
            norm += prob_dens * self.dx_m;
            exp_x_m += x_m * prob_dens * self.dx_m;
            exp_x2_m2 += x_m * x_m * prob_dens * self.dx_m;
        }

        let mean_x_nm = if norm > 1e-30 {
            (exp_x_m / norm) * 1e9
        } else {
            self.params.center_x_nm
        };

        let var_x_m2 = (exp_x2_m2 / norm.max(1e-30)) - (exp_x_m / norm.max(1e-30)).powi(2);
        let spread_nm = var_x_m2.max(0.0).sqrt() * 1e9;

        // Group velocity vg = hbar * k0 / m*
        let velocity_m_s = HBAR * self.k0_rad_per_m / self.effective_mass_kg;

        WavepacketDiagnostics {
            current_time_fs: self.current_time_fs,
            norm,
            mean_position_nm: mean_x_nm,
            mean_velocity_m_s: velocity_m_s,
            position_spread_nm: spread_nm,
            mean_energy_ev: self.params.initial_energy_ev,
            peak_density,
        }
    }

    /// Computes the momentum-space probability distribution |phi(k)|^2 via spatial Fourier transform.
    pub fn momentum_distribution(&self, k_samples: usize, k_max_rad_nm: f64) -> (Vec<f64>, Vec<f64>) {
        let n = self.params.grid_points;
        let mut k_values = Vec::with_capacity(k_samples);
        let mut prob_k = Vec::with_capacity(k_samples);

        let dk = (2.0 * k_max_rad_nm) / (k_samples as f64);
        for m in 0..k_samples {
            let k_nm = -k_max_rad_nm + m as f64 * dk;
            let k_m = k_nm * 1e9;
            k_values.push(k_nm);

            let mut fourier_sum = Complex::zero();
            for i in 0..n {
                let x_m = self.x_coords_nm[i] * 1e-9;
                let phase = -k_m * x_m;
                let exp_ikx = Complex::from_polar(1.0, phase);
                fourier_sum = fourier_sum.add(self.psi[i].mul(exp_ikx));
            }

            prob_k.push(fourier_sum.norm_sq() * self.dx_m);
        }

        (k_values, prob_k)
    }
}
