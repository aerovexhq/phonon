//! Tokamak magnetic geometry, poloidal flux functions, Solov'ev analytical equilibria,
//! safety factor profiles, and macroscopic beta limits.

use super::constants::VACUUM_PERMEABILITY;

/// Tokamak primary geometric and engineering specifications.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokamakGeometry {
    /// Major radius $R_0$ from the symmetry axis to the magnetic axis (meters).
    pub major_radius_r0: f64,
    /// Minor radius $a$ of the poloidal cross section (meters).
    pub minor_radius_a: f64,
    /// Plasma elongation $\kappa = b / a$ along the vertical $Z$ direction.
    pub elongation_kappa: f64,
    /// Plasma triangularity $\delta$.
    pub triangularity_delta: f64,
    /// On-axis toroidal magnetic field $B_0$ (Tesla).
    pub toroidal_b0: f64,
    /// Total toroidal plasma current $I_p$ (Amperes).
    pub plasma_current_ip: f64,
}

impl TokamakGeometry {
    /// Creates a new tokamak geometry specification.
    pub fn new(
        major_radius_r0: f64,
        minor_radius_a: f64,
        elongation_kappa: f64,
        triangularity_delta: f64,
        toroidal_b0: f64,
        plasma_current_ip: f64,
    ) -> Self {
        Self {
            major_radius_r0,
            minor_radius_a,
            elongation_kappa,
            triangularity_delta,
            toroidal_b0,
            plasma_current_ip,
        }
    }

    /// ITER baseline parameters (15 MA, 5.3 T, R0=6.2m, a=2.0m, kappa=1.70).
    pub fn iter_baseline() -> Self {
        Self {
            major_radius_r0: 6.2,
            minor_radius_a: 2.0,
            elongation_kappa: 1.70,
            triangularity_delta: 0.33,
            toroidal_b0: 5.3,
            plasma_current_ip: 15.0e6,
        }
    }

    /// SPARC high-field compact tokamak baseline (8.7 MA, 12.2 T, R0=1.85m, a=0.57m, kappa=1.97).
    pub fn sparc_baseline() -> Self {
        Self {
            major_radius_r0: 1.85,
            minor_radius_a: 0.57,
            elongation_kappa: 1.97,
            triangularity_delta: 0.40,
            toroidal_b0: 12.2,
            plasma_current_ip: 8.7e6,
        }
    }

    /// DIII-D research tokamak baseline (1.5 MA, 2.1 T, R0=1.67m, a=0.67m, kappa=1.80).
    pub fn diiid_baseline() -> Self {
        Self {
            major_radius_r0: 1.67,
            minor_radius_a: 0.67,
            elongation_kappa: 1.80,
            triangularity_delta: 0.35,
            toroidal_b0: 2.1,
            plasma_current_ip: 1.5e6,
        }
    }

    /// Aspect ratio $A = R_0 / a$.
    pub fn aspect_ratio(&self) -> f64 {
        self.major_radius_r0 / self.minor_radius_a
    }

    /// Inverse aspect ratio $\epsilon = a / R_0$.
    pub fn inverse_aspect_ratio(&self) -> f64 {
        self.minor_radius_a / self.major_radius_r0
    }

    /// Approximate plasma volume $V \approx 2\pi^2 R_0 a^2 \kappa$ ($m^3$).
    pub fn plasma_volume(&self) -> f64 {
        2.0 * std::f64::consts::PI
            * std::f64::consts::PI
            * self.major_radius_r0
            * self.minor_radius_a.powi(2)
            * self.elongation_kappa
    }

    /// Poloidal perimeter of cross section (Ramanujan ellipse approximation) in meters.
    pub fn poloidal_perimeter(&self) -> f64 {
        let a = self.minor_radius_a;
        let b = a * self.elongation_kappa;
        std::f64::consts::PI * (3.0 * (a + b) - ((3.0 * a + b) * (a + 3.0 * b)).sqrt())
    }

    /// Plasma cross-sectional area $S \approx \pi a^2 \kappa$ ($m^2$).
    pub fn cross_sectional_area(&self) -> f64 {
        std::f64::consts::PI * self.minor_radius_a.powi(2) * self.elongation_kappa
    }
}

/// Analytical Solov'ev equilibrium for axisymmetric tokamak plasma.
///
/// Solves the Grad-Shafranov elliptic equation:
/// $$\Delta^* \psi = R \frac{\partial}{\partial R}\left(\frac{1}{R}\frac{\partial\psi}{\partial R}\right) + \frac{\partial^2\psi}{\partial Z^2} = -\mu_0 R^2 p'(\psi) - F(\psi) F'(\psi)$$
/// with constant $p'$ and $F F' = 0$:
/// $$\psi(R, Z) = \frac{\psi_0}{R_0^4} \left[ R^2 Z^2 + \frac{\kappa^2}{4}(R^2 - R_0^2)^2 \right]$$
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolovevEquilibrium {
    /// Associated tokamak geometry.
    pub geometry: TokamakGeometry,
    /// Reference poloidal flux constant $\psi_0$ (Weber).
    pub psi_0: f64,
    /// Value of $\psi$ at the plasma separatrix boundary $\psi_{sep}$ (Weber).
    pub psi_sep: f64,
}

impl SolovevEquilibrium {
    /// Creates a Solov'ev analytical equilibrium from geometry and flux scale.
    pub fn new(geometry: TokamakGeometry, psi_0: f64, psi_sep: f64) -> Self {
        Self {
            geometry,
            psi_0,
            psi_sep,
        }
    }

    /// Calibrates $\psi_0$ such that total toroidal current equals $I_p$.
    pub fn from_geometry_and_current(geometry: TokamakGeometry) -> Self {
        let r0 = geometry.major_radius_r0;
        let a = geometry.minor_radius_a;
        let kappa = geometry.elongation_kappa;
        let ip = geometry.plasma_current_ip;

        // Current density J_phi = (2 * (1 + kappa^2) * psi_0) / (mu_0 * R_0^4) * R
        // Integrating over cross section S = pi * a^2 * kappa:
        // I_p approx J_phi(R0) * S = (2 * (1 + kappa^2) * psi_0 / (mu_0 * R_0^3)) * (pi * a^2 * kappa)
        let s = std::f64::consts::PI * a.powi(2) * kappa;
        let psi_0 = (ip * VACUUM_PERMEABILITY * r0.powi(3)) / (2.0 * (1.0 + kappa.powi(2)) * s);
        let psi_sep = psi_0 * (kappa.powi(2) / 4.0) * (a.powi(2) / r0.powi(2)).powi(2) * 4.0;
        let psi_sep_adjusted = if psi_sep.abs() < 1e-12 {
            psi_0 * 0.1
        } else {
            psi_sep
        };

        Self {
            geometry,
            psi_0,
            psi_sep: psi_sep_adjusted,
        }
    }

    /// Evaluates the poloidal magnetic flux $\psi(R, Z)$ in Webers.
    pub fn flux_at(&self, r: f64, z: f64) -> f64 {
        let r0 = self.geometry.major_radius_r0;
        let kappa = self.geometry.elongation_kappa;
        let factor = self.psi_0 / r0.powi(4);
        factor * (r.powi(2) * z.powi(2) + 0.25 * kappa.powi(2) * (r.powi(2) - r0.powi(2)).powi(2))
    }

    /// Evaluates normalized flux $\bar{\psi} = \psi / \psi_{sep} \in [0, 1]$.
    pub fn normalized_flux_at(&self, r: f64, z: f64) -> f64 {
        let psi = self.flux_at(r, z);
        (psi / self.psi_sep).clamp(0.0, 1.0)
    }

    /// Gradient of poloidal flux $\nabla\psi = (\partial\psi/\partial R, \partial\psi/\partial Z)$.
    pub fn grad_flux_at(&self, r: f64, z: f64) -> (f64, f64) {
        let r0 = self.geometry.major_radius_r0;
        let kappa = self.geometry.elongation_kappa;
        let factor = self.psi_0 / r0.powi(4);

        let dpsi_dr = factor * r * (2.0 * z.powi(2) + kappa.powi(2) * (r.powi(2) - r0.powi(2)));
        let dpsi_dz = factor * 2.0 * r.powi(2) * z;
        (dpsi_dr, dpsi_dz)
    }

    /// Evaluates the 3D magnetic field $\mathbf{B} = (B_R, B_\phi, B_Z)$ in Tesla at $(R, Z)$.
    pub fn magnetic_field_at(&self, r: f64, z: f64) -> [f64; 3] {
        let safe_r = r.max(0.01);
        let (dpsi_dr, dpsi_dz) = self.grad_flux_at(safe_r, z);

        // B_R = - (1 / R) * dpsi/dZ
        let b_r = -dpsi_dz / safe_r;
        // B_Z = + (1 / R) * dpsi/dR
        let b_z = dpsi_dr / safe_r;
        // B_phi = F(psi) / R = R0 * B0 / R
        let b_phi = (self.geometry.major_radius_r0 * self.geometry.toroidal_b0) / safe_r;

        [b_r, b_phi, b_z]
    }

    /// Total magnetic field strength $B = \|\mathbf{B}\|$ at $(R, Z)$.
    pub fn b_magnitude_at(&self, r: f64, z: f64) -> f64 {
        let b = self.magnetic_field_at(r, z);
        (b[0].powi(2) + b[1].powi(2) + b[2].powi(2)).sqrt()
    }

    /// Poloidal magnetic field strength $B_p = \sqrt{B_R^2 + B_Z^2} = \frac{|\nabla\psi|}{R}$.
    pub fn poloidal_b_at(&self, r: f64, z: f64) -> f64 {
        let safe_r = r.max(0.01);
        let (dpsi_dr, dpsi_dz) = self.grad_flux_at(safe_r, z);
        (dpsi_dr.powi(2) + dpsi_dz.powi(2)).sqrt() / safe_r
    }

    /// Evaluates the Grad-Shafranov elliptic residual $\Delta^* \psi + \mu_0 R^2 p'(\psi) + F F'$.
    pub fn grad_shafranov_operator_at(&self, r: f64, _z: f64) -> f64 {
        let r0 = self.geometry.major_radius_r0;
        let kappa = self.geometry.elongation_kappa;
        // Exact Delta* psi = 2 * (1 + kappa^2) * (psi_0 / R_0^4) * R^2
        2.0 * (1.0 + kappa.powi(2)) * (self.psi_0 / r0.powi(4)) * r.powi(2)
    }
}

/// Helical safety factor profile $q(r)$ and magnetic shear $s(r)$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SafetyFactorProfile {
    /// On-axis safety factor $q_0 = q(0)$ (typically $\approx 1.05$).
    pub q_0: f64,
    /// Edge safety factor at limiter/separatrix $q_a = q(a)$ (typically $3.0 - 4.5$).
    pub q_a: f64,
    /// Profile shaping exponent $\alpha$ ($q(\rho) = q_0 + (q_a - q_0) \rho^\alpha$).
    pub alpha: f64,
}

impl SafetyFactorProfile {
    /// Standard parabolic safety factor profile ($\alpha = 2.0$).
    pub fn new(q_0: f64, q_a: f64) -> Self {
        Self {
            q_0,
            q_a,
            alpha: 2.0,
        }
    }

    /// Generalized power-law profile.
    pub fn with_alpha(q_0: f64, q_a: f64, alpha: f64) -> Self {
        Self { q_0, q_a, alpha }
    }

    /// Evaluates safety factor $q(\rho)$ at normalized minor radius $\rho = r / a \in [0, 1]$.
    pub fn q_at_rho(&self, rho: f64) -> f64 {
        let r_norm = rho.clamp(0.0, 1.0);
        self.q_0 + (self.q_a - self.q_0) * r_norm.powf(self.alpha)
    }

    /// Evaluates derivative $dq/d\rho$.
    pub fn dq_drho(&self, rho: f64) -> f64 {
        let r_norm = rho.clamp(0.0, 1.0);
        if r_norm < 1e-9 {
            0.0
        } else {
            (self.q_a - self.q_0) * self.alpha * r_norm.powf(self.alpha - 1.0)
        }
    }

    /// Evaluates dimensionless magnetic shear $s(\rho) = \frac{\rho}{q} \frac{dq}{d\rho}$.
    pub fn shear_at_rho(&self, rho: f64) -> f64 {
        let q = self.q_at_rho(rho);
        if q.abs() < 1e-12 {
            0.0
        } else {
            (rho * self.dq_drho(rho)) / q
        }
    }

    /// Computes the radial location $\rho_{res}$ of a rational resonant surface where $q(\rho) = m / n$.
    pub fn resonant_surface(&self, m: usize, n: usize) -> Option<f64> {
        let q_target = (m as f64) / (n as f64);
        if q_target < self.q_0 || q_target > self.q_a {
            None
        } else {
            let frac = (q_target - self.q_0) / (self.q_a - self.q_0);
            Some(frac.powf(1.0 / self.alpha))
        }
    }

    /// Evaluates Mercier ideal MHD interchange stability criterion $D_I > 0$.
    ///
    /// $D_I = \frac{1}{4} s^2 + \frac{2\mu_0 r}{B^2}\left(-\frac{dp}{dr}\right)(1 - q^2) > 0$.
    pub fn mercier_criterion(&self, rho: f64, r_minor: f64, dp_dr: f64, b_total: f64) -> f64 {
        let s = self.shear_at_rho(rho);
        let q = self.q_at_rho(rho);
        let b_sq = b_total.powi(2).max(1e-6);

        let shear_term = 0.25 * s.powi(2);
        let well_term = (2.0 * VACUUM_PERMEABILITY * r_minor / b_sq) * (-dp_dr) * (1.0 - q.powi(2));
        shear_term + well_term
    }
}

/// Tokamak operational beta parameters and Troyon empirical limit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TokamakBeta {
    /// Toroidal beta $\beta_t = \frac{2\mu_0 \langle p \rangle}{B_0^2}$.
    pub beta_t: f64,
    /// Poloidal beta $\beta_p = \frac{2\mu_0 \langle p \rangle}{B_p^2(a)}$.
    pub beta_p: f64,
    /// Normalized beta $\beta_N = \beta_t (\%) \cdot \frac{a B_0}{I_p (\text{MA})}$.
    pub beta_n: f64,
}

impl TokamakBeta {
    /// Calculates operational betas given volume-averaged pressure, geometry, and current.
    pub fn calculate(p_avg_pa: f64, geom: &TokamakGeometry) -> Self {
        let b0 = geom.toroidal_b0;
        let beta_t = (2.0 * VACUUM_PERMEABILITY * p_avg_pa) / b0.powi(2);

        // Approximate edge poloidal field B_p(a) approx mu_0 * I_p / (2 * pi * a * sqrt((1+kappa^2)/2))
        let perim = geom.poloidal_perimeter();
        let bp_edge = (VACUUM_PERMEABILITY * geom.plasma_current_ip) / perim.max(0.1);
        let beta_p = (2.0 * VACUUM_PERMEABILITY * p_avg_pa) / bp_edge.powi(2).max(1e-6);

        let ip_ma = geom.plasma_current_ip / 1.0e6;
        let beta_t_percent = beta_t * 100.0;
        let beta_n = if ip_ma.abs() > 1e-6 {
            beta_t_percent * (geom.minor_radius_a * b0) / ip_ma
        } else {
            0.0
        };

        Self {
            beta_t,
            beta_p,
            beta_n,
        }
    }

    /// Checks if the operating state is below the Troyon beta limit (typically $\beta_N \le 2.8 - 3.5$).
    pub fn is_below_troyon_limit(&self, troyon_limit: f64) -> bool {
        self.beta_n <= troyon_limit
    }

    /// Evaluates Greenwald empirical density limit $n_G = \frac{I_p}{\pi a^2} \times 10^{20}\text{ m}^{-3}$.
    pub fn greenwald_density_limit(ip_amps: f64, minor_radius_a: f64) -> f64 {
        let ip_ma = ip_amps / 1.0e6;
        let area = std::f64::consts::PI * minor_radius_a.powi(2);
        (ip_ma / area.max(1e-6)) * 1.0e20
    }
}
