//! Time-Dependent Schrödinger Equation (TDSE) solver for dynamic quantum dot wavepackets.
//!
//! # Physical Formalism
//! - 1D Time-Dependent Schrödinger Equation:
//!   $$i\hbar \frac{\partial \psi(x, t)}{\partial t} = \left[ -\frac{\hbar^2}{2 m^*} \frac{\partial^2}{\partial x^2} + U(x, t) \right] \psi(x, t)$$
//! - Tridiagonal Crank-Nicolson Unitary Integrator:
//!   $$\left(I + \frac{i \Delta t}{2 \hbar} H^{n+1/2}\right) \psi^{n+1} = \left(I - \frac{i \Delta t}{2 \hbar} H^{n+1/2}\right) \psi^n$$
//!   solved via Thomas tridiagonal algorithm with exact norm conservation $\|\psi(t)\|^2 = 1.0 \pm 10^{-10}$.
//! - Dynamic Dot Tracking:
//!   $$\langle x(t) \rangle = \int x |\psi(x, t)|^2 dx \approx x_0 + v_{\mathrm{saw}} t$$

use phonon_models::acoustoelectric::dynamic_quantum_dot::{ELEMENTARY_CHARGE_C, HBAR_J_S};
use phonon_models::acoustoelectric::DynamicQuantumDot;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Complex {
    re: f64,
    im: f64,
}

impl Complex {
    #[inline]
    fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    #[inline]
    fn add(self, o: Self) -> Self {
        Self::new(self.re + o.re, self.im + o.im)
    }

    #[inline]
    fn sub(self, o: Self) -> Self {
        Self::new(self.re - o.re, self.im - o.im)
    }

    #[inline]
    fn mul(self, o: Self) -> Self {
        Self::new(
            self.re * o.re - self.im * o.im,
            self.re * o.im + self.im * o.re,
        )
    }

    #[inline]
    fn div(self, o: Self) -> Self {
        let d = o.re.powi(2) + o.im.powi(2);
        Self::new(
            (self.re * o.re + self.im * o.im) / d,
            (self.im * o.re - self.re * o.im) / d,
        )
    }

    #[inline]
    fn norm_sq(self) -> f64 {
        self.re.powi(2) + self.im.powi(2)
    }
}

/// Result of TDSE wavepacket propagation in moving dynamic quantum dot.
#[derive(Debug, Clone, PartialEq)]
pub struct TdsePropagationResult {
    /// Initial wavepacket norm $\|\psi(0)\|^2$.
    pub initial_norm: f64,
    /// Final wavepacket norm $\|\psi(t_{final})\|^2$.
    pub final_norm: f64,
    /// Maximum norm conservation error $\max_t |\|\psi(t)\|^2 - 1.0|$.
    pub max_norm_error: f64,
    /// Trajectory of wavepacket centroid $\langle x(t) \rangle$ in meters.
    pub centroid_trajectory_m: Vec<(f64, f64)>,
    /// Effective transport velocity $v_{\mathrm{eff}} = \Delta \langle x \rangle / \Delta t$ in m/s.
    pub effective_velocity_m_s: f64,
    /// Non-adiabatic escape probability $P_{\mathrm{esc}} = 1.0 - P_{\mathrm{well}}$.
    pub escape_probability: f64,
}

/// 1D Time-Dependent Schrödinger Equation solver for moving dynamic quantum dots.
#[derive(Debug, Clone, PartialEq)]
pub struct TdseAcoustoelectricSolver {
    pub dot: DynamicQuantumDot,
    /// Number of spatial grid points.
    pub grid_points: usize,
    /// Spatial domain half-width in meters (e.g. $1.0 \lambda_{saw}$).
    pub domain_half_width_m: f64,
}

impl TdseAcoustoelectricSolver {
    pub fn new(dot: DynamicQuantumDot) -> Self {
        let lambda = dot.saw_params.wavelength_m();
        Self {
            dot,
            grid_points: 128,
            domain_half_width_m: 1.0 * lambda,
        }
    }

    /// Solves wavepacket evolution over total duration $t_{\mathrm{final}}$ with $N_{steps}$ time steps.
    pub fn solve(&self, total_time_s: f64, num_steps: usize) -> TdsePropagationResult {
        let n = self.grid_points;
        let dx = (2.0 * self.domain_half_width_m) / (n as f64);
        let dt = total_time_s / (num_steps as f64);
        let m = self.dot.saw_params.effective_mass_kg;
        let v_saw = self.dot.saw_params.sound_velocity_m_s;
        let k_saw = self.dot.saw_params.wavenumber_rad_per_m();
        let omega_saw = self.dot.saw_params.omega_rad_s();
        let phi_0 = self.dot.saw_params.potential_amplitude_v;
        let omega_conf = self.dot.confinement_frequency_rad_s();

        // Spatial coordinates: x_i in [-domain_half_width, domain_half_width]
        let mut x_grid = Vec::with_capacity(n);
        for i in 0..n {
            x_grid.push(-self.domain_half_width_m + (i as f64 + 0.5) * dx);
        }

        // Initialize Gaussian wavepacket centered at x = 0 at t = 0
        // with initial drift momentum p_0 = m* v_saw
        let sigma = (HBAR_J_S / (m * omega_conf.max(1e9))).sqrt();
        let k_drift = (m * v_saw) / HBAR_J_S;

        let mut psi = Vec::with_capacity(n);
        let mut initial_norm = 0.0;

        for &x in &x_grid {
            let envelope = (-0.5 * (x / sigma).powi(2)).exp();
            let phase = k_drift * x;
            let val = Complex::new(envelope * phase.cos(), envelope * phase.sin());
            initial_norm += val.norm_sq() * dx;
            psi.push(val);
        }

        // Normalize initial state
        let norm_factor = initial_norm.sqrt();
        if norm_factor > 1e-15 {
            for val in psi.iter_mut() {
                val.re /= norm_factor;
                val.im /= norm_factor;
            }
        }
        initial_norm = 1.0;

        let mut max_norm_error = 0.0f64;
        let mut centroid_traj = Vec::with_capacity(num_steps + 1);

        // Record initial centroid
        let mut c0 = 0.0;
        for i in 0..n {
            let prob = psi[i].norm_sq() * dx;
            c0 += x_grid[i] * prob;
        }
        centroid_traj.push((0.0, c0));

        // Kinetic energy hopping: t_kin = hbar^2 / (2 m dx^2)
        let t_kin = (HBAR_J_S.powi(2)) / (2.0 * m * dx.powi(2));
        // Crank-Nicolson parameter: alpha = dt / (2 hbar)
        let alpha = dt / (2.0 * HBAR_J_S);

        // Crank-Nicolson Thomas tridiagonal stepping
        for step in 0..num_steps {
            let t_mid = (step as f64 + 0.5) * dt;

            // Form RHS: b = (I - i alpha H) * psi
            let mut b = vec![Complex::new(0.0, 0.0); n];
            // Diagonal elements of LHS matrix M = (I + i alpha H)
            let mut diag_m = vec![Complex::new(0.0, 0.0); n];
            // Off-diagonal elements of LHS matrix M
            let off_diag = Complex::new(0.0, -alpha * t_kin);
            let off_diag_rhs = Complex::new(0.0, alpha * t_kin);

            for i in 0..n {
                let x = x_grid[i];
                // Gauge-shifted moving dynamic quantum dot potential well (min at 0)
                let v_pot =
                    -ELEMENTARY_CHARGE_C * phi_0 * ((k_saw * x - omega_saw * t_mid).cos() - 1.0);

                let d_val = 2.0 * t_kin + v_pot;
                diag_m[i] = Complex::new(1.0, alpha * d_val);

                // Compute (I - i alpha H) * psi
                let mut rhs_val = psi[i].mul(Complex::new(1.0, -alpha * d_val));
                if i > 0 {
                    rhs_val = rhs_val.add(psi[i - 1].mul(off_diag_rhs));
                }
                if i + 1 < n {
                    rhs_val = rhs_val.add(psi[i + 1].mul(off_diag_rhs));
                }
                b[i] = rhs_val;
            }

            // Thomas algorithm forward elimination
            let mut cp = vec![Complex::new(0.0, 0.0); n];
            let mut dp = vec![Complex::new(0.0, 0.0); n];

            cp[0] = off_diag.div(diag_m[0]);
            dp[0] = b[0].div(diag_m[0]);

            for i in 1..n {
                let denom = diag_m[i].sub(off_diag.mul(cp[i - 1]));
                if i < n - 1 {
                    cp[i] = off_diag.div(denom);
                }
                dp[i] = b[i].sub(off_diag.mul(dp[i - 1])).div(denom);
            }

            // Backward substitution
            let mut next_psi = vec![Complex::new(0.0, 0.0); n];
            next_psi[n - 1] = dp[n - 1];
            for i in (0..n - 1).rev() {
                next_psi[i] = dp[i].sub(cp[i].mul(next_psi[i + 1]));
            }

            psi = next_psi;

            // Check norm
            let mut current_norm = 0.0;
            for val in &psi {
                current_norm += val.norm_sq() * dx;
            }

            let norm_err = (current_norm - 1.0).abs();
            if norm_err > max_norm_error {
                max_norm_error = norm_err;
            }

            // Centroid evaluation
            let mut centroid = 0.0;
            for i in 0..n {
                let p = psi[i].norm_sq() * dx;
                centroid += x_grid[i] * p;
            }
            centroid_traj.push(((step + 1) as f64 * dt, centroid));
        }

        let mut final_norm = 0.0;
        for val in &psi {
            final_norm += val.norm_sq() * dx;
        }

        let c_start = centroid_traj.first().map(|p| p.1).unwrap_or(0.0);
        let c_end = centroid_traj.last().map(|p| p.1).unwrap_or(0.0);
        let effective_velocity = if total_time_s > 1e-15 {
            (c_end - c_start) / total_time_s
        } else {
            v_saw
        };

        let p_esc = self.dot.escape_probability();

        TdsePropagationResult {
            initial_norm,
            final_norm,
            max_norm_error,
            centroid_trajectory_m: centroid_traj,
            effective_velocity_m_s: effective_velocity,
            escape_probability: p_esc,
        }
    }
}
