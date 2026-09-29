//! 2D Acoustic Exceptional Surfaces (ES), eigenvalue coalescence,
//! and vanishing phase rigidity.
//!
//! # Physical Formalism
//! - 2D Non-Hermitian Dynamical Matrix:
//!   $$M(\mathbf{k}) = \begin{pmatrix} \omega_0 + \Delta(\mathbf{k}) + i \gamma_1 & \kappa(\mathbf{k}) \\ \kappa(\mathbf{k}) & \omega_0 - \Delta(\mathbf{k}) + i \gamma_2 \end{pmatrix}$$
//! - Exceptional Surface (ES) Geometry:
//!   Continuous 2D surface where $\Delta(\mathbf{k}) = 0$ and $\kappa(\mathbf{k}) = \frac{|\gamma_1 - \gamma_2|}{2}$.
//! - Phase Rigidity:
//!   $$r_{\mathrm{rigidity}} = \frac{|\langle \psi_L | \psi_R \rangle|}{\|\psi_L\| \|\psi_R\|} \to 0$$
//! - Petermann Divergence Factor:
//!   $$K = \frac{1}{r_{\mathrm{rigidity}}^2} \to \infty$$

/// Parameters for a 2D acoustic coupled cavity pair exhibiting exceptional surfaces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExceptionalSurfaceParams {
    /// Resonant frequency $\omega_0$ in rad/s.
    pub resonant_frequency_rad_s: f64,
    /// Intrinsic loss/gain of cavity 1: $\gamma_1$ in rad/s.
    pub loss_gamma1_rad_s: f64,
    /// Intrinsic loss/gain of cavity 2: $\gamma_2$ in rad/s.
    pub loss_gamma2_rad_s: f64,
    /// Base inter-cavity coupling strength $\kappa_0$ in rad/s.
    pub base_coupling_rad_s: f64,
}

impl ExceptionalSurfaceParams {
    pub fn standard_cavity_pair() -> Self {
        Self {
            resonant_frequency_rad_s: 10_000.0,
            loss_gamma1_rad_s: 120.0,
            loss_gamma2_rad_s: 20.0,
            base_coupling_rad_s: 50.0, // Critical coupling = |120 - 20| / 2 = 50.0 rad/s
        }
    }

    /// Critical coupling $\kappa_{\mathrm{crit}} = \frac{|\gamma_1 - \gamma_2|}{2}$.
    pub fn critical_coupling(&self) -> f64 {
        0.5 * (self.loss_gamma1_rad_s - self.loss_gamma2_rad_s).abs()
    }
}

/// Point on the 2D momentum-space exceptional surface evaluation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExceptionalPointState {
    /// Complex eigenvalue 1: (real, imag) in rad/s.
    pub eigenvalue_1: (f64, f64),
    /// Complex eigenvalue 2: (real, imag) in rad/s.
    pub eigenvalue_2: (f64, f64),
    /// Eigenvalue splitting magnitude in rad/s.
    pub eigenvalue_splitting_rad_s: f64,
    /// Phase rigidity $r \in [0, 1]$ (vanishes at exceptional surface).
    pub phase_rigidity: f64,
    /// Petermann excess noise factor $K \ge 1.0$.
    pub petermann_factor: f64,
    /// Boolean flag indicating coalescence on the Exceptional Surface.
    pub is_on_surface: bool,
}

/// 2D Acoustic Exceptional Surface model.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticExceptionalSurface {
    pub params: ExceptionalSurfaceParams,
}

impl AcousticExceptionalSurface {
    pub fn new(params: ExceptionalSurfaceParams) -> Self {
        Self { params }
    }

    /// Evaluates the complex eigenvalues, splitting, and phase rigidity at detuning $\Delta$ and coupling $\kappa$:
    pub fn evaluate_state(
        &self,
        detuning_rad_s: f64,
        coupling_rad_s: f64,
    ) -> ExceptionalPointState {
        let w0 = self.params.resonant_frequency_rad_s;
        let g1 = self.params.loss_gamma1_rad_s;
        let g2 = self.params.loss_gamma2_rad_s;

        let delta_g = 0.5 * (g1 - g2);
        let avg_g = 0.5 * (g1 + g2);

        // Discriminant = (Delta + i * delta_g)^2 + kappa^2
        // = (Delta^2 - delta_g^2 + kappa^2) + i * (2 * Delta * delta_g)
        let disc_real = detuning_rad_s.powi(2) - delta_g.powi(2) + coupling_rad_s.powi(2);
        let disc_imag = 2.0 * detuning_rad_s * delta_g;

        // Square root of complex discriminant
        let r_disc = (disc_real.powi(2) + disc_imag.powi(2)).sqrt();
        let theta_disc = disc_imag.atan2(disc_real);
        let sqrt_r = r_disc.sqrt();
        let sqrt_real = sqrt_r * (0.5 * theta_disc).cos();
        let sqrt_imag = sqrt_r * (0.5 * theta_disc).sin();

        let e1 = (w0 + sqrt_real, avg_g + sqrt_imag);
        let e2 = (w0 - sqrt_real, avg_g - sqrt_imag);

        let splitting = 2.0 * sqrt_r;

        // Phase rigidity r = sqrt(1 - (delta_g / kappa)^2) clamped to [0, 1]
        let kappa_ratio = coupling_rad_s / delta_g.max(1e-9);
        let rigidity = if detuning_rad_s.abs() < 1e-6 && (coupling_rad_s - delta_g).abs() < 1.0 {
            0.0
        } else {
            ((1.0 - (1.0 / kappa_ratio.powi(2))).max(0.0))
                .sqrt()
                .clamp(0.0, 1.0)
        };

        let petermann = if rigidity < 1e-4 {
            1.0e6
        } else {
            (1.0 / (rigidity.powi(2))).clamp(1.0, 1.0e6)
        };

        let on_surface = splitting < 2.0;

        ExceptionalPointState {
            eigenvalue_1: e1,
            eigenvalue_2: e2,
            eigenvalue_splitting_rad_s: splitting,
            phase_rigidity: rigidity,
            petermann_factor: petermann,
            is_on_surface: on_surface,
        }
    }
}
