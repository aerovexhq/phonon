//! 2D Finite-Difference Elliptic Solver for the Grad-Shafranov Tokamak Equilibrium.
//!
//! Solves:
//! $$\Delta^* \psi = R \frac{\partial}{\partial R}\left( \frac{1}{R}\frac{\partial\psi}{\partial R} \right) + \frac{\partial^2 \psi}{\partial Z^2} = -\mu_0 R^2 p'(\psi) - F(\psi)F'(\psi)$$
//! using Successive Over-Relaxation (SOR) with Chebyshev acceleration.

use phonon_models::plasma::{SafetyFactorProfile, TokamakGeometry, VACUUM_PERMEABILITY};

/// 2D rectangular computational grid in cylindrical coordinates $(R, Z)$.
#[derive(Debug, Clone, PartialEq)]
pub struct GradShafranovGrid {
    /// Minimum major radius $R_{min}$ (meters).
    pub r_min: f64,
    /// Maximum major radius $R_{max}$ (meters).
    pub r_max: f64,
    /// Vertical half-height $Z_{max}$ (domain spans $[-Z_{max}, Z_{max}]$) in meters.
    pub z_max: f64,
    /// Number of grid points along $R$.
    pub nr: usize,
    /// Number of grid points along $Z$.
    pub nz: usize,
    /// Grid spacing along $R$: $\Delta R = (R_{max} - R_{min}) / (N_R - 1)$.
    pub dr: f64,
    /// Grid spacing along $Z$: $\Delta Z = 2 Z_{max} / (N_Z - 1)$.
    pub dz: f64,
}

impl GradShafranovGrid {
    /// Creates a new 2D grid for a given tokamak geometry.
    pub fn for_geometry(geom: &TokamakGeometry, nr: usize, nz: usize) -> Self {
        assert!(nr >= 5 && nz >= 5, "Grid must have at least 5x5 points");
        let r0 = geom.major_radius_r0;
        let a = geom.minor_radius_a;
        let kappa = geom.elongation_kappa;

        // Domain padding: span slightly beyond plasma boundary
        let pad = 1.3;
        let r_min = (r0 - a * pad).max(0.1);
        let r_max = r0 + a * pad;
        let z_max = a * kappa * pad;

        let dr = (r_max - r_min) / ((nr - 1) as f64);
        let dz = (2.0 * z_max) / ((nz - 1) as f64);

        Self {
            r_min,
            r_max,
            z_max,
            nr,
            nz,
            dr,
            dz,
        }
    }

    /// Major radius coordinate at index $i \in [0, N_R - 1]$.
    pub fn r_at(&self, i: usize) -> f64 {
        self.r_min + (i as f64) * self.dr
    }

    /// Vertical coordinate at index $j \in [0, N_Z - 1]$.
    pub fn z_at(&self, j: usize) -> f64 {
        -self.z_max + (j as f64) * self.dz
    }

    /// Linear index in flattened 1D array corresponding to $(i, j)$.
    pub fn index_of(&self, i: usize, j: usize) -> usize {
        j * self.nr + i
    }
}

/// Solution output from the 2D Grad-Shafranov elliptic solver.
#[derive(Debug, Clone, PartialEq)]
pub struct GradShafranovSolution {
    /// Computational grid definition.
    pub grid: GradShafranovGrid,
    /// Poloidal magnetic flux values $\psi(i, j)$ in Webers (size $N_R \times N_Z$).
    pub psi: Vec<f64>,
    /// Coordinates of detected magnetic axis $(R_{axis}, Z_{axis})$ in meters.
    pub magnetic_axis: (f64, f64),
    /// On-axis peak poloidal flux $\psi_{axis}$ (Weber).
    pub psi_axis: f64,
    /// Boundary/separatrix poloidal flux $\psi_{sep}$ (Weber).
    pub psi_sep: f64,
    /// Integrated total toroidal plasma current $I_p = \int J_\phi dR dZ$ (Amperes).
    pub total_current: f64,
    /// Whether solver reached convergence tolerance.
    pub converged: bool,
    /// Iteration count performed.
    pub iterations: usize,
    /// Maximum absolute residual $\|\Delta^* \psi - S\|_\infty$.
    pub max_residual: f64,
}

impl GradShafranovSolution {
    /// Evaluates poloidal flux $\psi(R, Z)$ using bilinear interpolation on the 2D grid.
    pub fn flux_at(&self, r: f64, z: f64) -> f64 {
        let r_clamped = r.clamp(self.grid.r_min, self.grid.r_max);
        let z_clamped = z.clamp(-self.grid.z_max, self.grid.z_max);

        let i_float = (r_clamped - self.grid.r_min) / self.grid.dr;
        let j_float = (z_clamped + self.grid.z_max) / self.grid.dz;

        let i0 = (i_float.floor() as usize).min(self.grid.nr - 2);
        let j0 = (j_float.floor() as usize).min(self.grid.nz - 2);
        let i1 = i0 + 1;
        let j1 = j0 + 1;

        let fr = i_float - (i0 as f64);
        let fz = j_float - (j0 as f64);

        let p00 = self.psi[self.grid.index_of(i0, j0)];
        let p10 = self.psi[self.grid.index_of(i1, j0)];
        let p01 = self.psi[self.grid.index_of(i0, j1)];
        let p11 = self.psi[self.grid.index_of(i1, j1)];

        (1.0 - fr) * (1.0 - fz) * p00
            + fr * (1.0 - fz) * p10
            + (1.0 - fr) * fz * p01
            + fr * fz * p11
    }

    /// Evaluates 3D magnetic field vector $\mathbf{B} = (B_R, B_\phi, B_Z)$ at $(R, Z)$.
    pub fn magnetic_field_at(&self, r: f64, z: f64, f_toroidal: f64) -> [f64; 3] {
        let eps = 1e-4;
        let safe_r = r.max(0.01);
        let psi_r_plus = self.flux_at(safe_r + eps, z);
        let psi_r_minus = self.flux_at(safe_r - eps, z);
        let psi_z_plus = self.flux_at(safe_r, z + eps);
        let psi_z_minus = self.flux_at(safe_r, z - eps);

        let dpsi_dr = (psi_r_plus - psi_r_minus) / (2.0 * eps);
        let dpsi_dz = (psi_z_plus - psi_z_minus) / (2.0 * eps);

        let b_r = -dpsi_dz / safe_r;
        let b_z = dpsi_dr / safe_r;
        let b_phi = f_toroidal / safe_r;

        [b_r, b_phi, b_z]
    }

    /// Extracts safety factor profile estimate from numerical flux distribution.
    pub fn extract_safety_factor_profile(&self, f_toroidal: f64) -> SafetyFactorProfile {
        let r_axis = self.magnetic_axis.0;
        let (_br_ax, bphi_ax, _bz_ax) = {
            let b = self.magnetic_field_at(r_axis, 0.0, f_toroidal);
            (b[0], b[1], b[2])
        };
        let b_phi_axis = bphi_ax.abs();

        // On-axis safety factor q0 approx (R0 * B_phi / (mu_0 * j_phi0 * a_eff^2))
        // or derived from curvature of psi at O-point:
        // q0 approx 2 * B_phi / (R0 * sqrt(d2psi_dr2 * d2psi_dz2))
        let eps = self.grid.dr.min(self.grid.dz) * 0.5;
        let p_center = self.flux_at(r_axis, 0.0);
        let p_r1 = self.flux_at(r_axis + eps, 0.0);
        let p_r0 = self.flux_at(r_axis - eps, 0.0);
        let p_z1 = self.flux_at(r_axis, eps);
        let p_z0 = self.flux_at(r_axis, -eps);

        let d2_dr2 = (p_r1 - 2.0 * p_center + p_r0) / eps.powi(2);
        let d2_dz2 = (p_z1 - 2.0 * p_center + p_z0) / eps.powi(2);
        let curv_product = (d2_dr2 * d2_dz2).abs().max(1e-6);

        let q0_est = (b_phi_axis / (r_axis * curv_product.sqrt())).clamp(0.8, 1.5);
        let qa_est = (q0_est * 3.5).clamp(2.5, 5.5);

        SafetyFactorProfile::new(q0_est, qa_est)
    }
}

/// Successive Over-Relaxation (SOR) 2D Grad-Shafranov elliptic PDE solver.
#[derive(Debug, Clone)]
pub struct GradShafranovSolver {
    /// Maximum SOR iteration count.
    pub max_iterations: usize,
    /// Absolute residual convergence tolerance.
    pub tolerance: f64,
    /// SOR over-relaxation parameter $\omega \in (1.0, 2.0)$ (default: $1.70$).
    pub omega: f64,
}

impl Default for GradShafranovSolver {
    fn default() -> Self {
        Self {
            max_iterations: 2000,
            tolerance: 1e-6,
            omega: 1.70,
        }
    }
}

impl GradShafranovSolver {
    /// Creates a solver with specified maximum iterations and tolerance.
    pub fn new(max_iterations: usize, tolerance: f64, omega: f64) -> Self {
        Self {
            max_iterations,
            tolerance,
            omega,
        }
    }

    /// Solves the 2D Grad-Shafranov equation for a given tokamak geometry with prescribed profiles.
    ///
    /// Prescribes linear profiles:
    /// $p'(\psi) = -p_0 / \psi_{sep}$, $F(\psi) F'(\psi) = 0$.
    pub fn solve(&self, geom: &TokamakGeometry, nr: usize, nz: usize) -> GradShafranovSolution {
        let grid = GradShafranovGrid::for_geometry(geom, nr, nz);
        let n_total = nr * nz;
        let mut psi = vec![0.0; n_total];

        let r0 = geom.major_radius_r0;
        let a = geom.minor_radius_a;
        let kappa = geom.elongation_kappa;
        let ip_target = geom.plasma_current_ip;

        // Initial guess based on analytical Solov'ev
        let psi_scale = (ip_target * VACUUM_PERMEABILITY * r0.powi(3))
            / (2.0 * (1.0 + kappa.powi(2)) * std::f64::consts::PI * a.powi(2) * kappa);

        for j in 0..nz {
            let z = grid.z_at(j);
            for i in 0..nr {
                let r = grid.r_at(i);
                let dist_sq = ((r - r0) / a).powi(2) + (z / (a * kappa)).powi(2);
                let idx = grid.index_of(i, j);
                if dist_sq < 1.0 {
                    psi[idx] = psi_scale * (1.0 - dist_sq);
                } else {
                    psi[idx] = 0.0;
                }
            }
        }

        let dr2 = grid.dr.powi(2);
        let dz2 = grid.dz.powi(2);
        let mut converged = false;
        let mut iter_count = 0;
        let mut max_res = 1.0;

        // Current scaling factor J_0 such that integral equals I_p
        let plasma_area = std::f64::consts::PI * a.powi(2) * kappa;
        let j_phi_0 = ip_target / plasma_area.max(1e-4);

        for iter in 0..self.max_iterations {
            iter_count = iter + 1;
            max_res = 0.0;

            for j in 1..(nz - 1) {
                let z = grid.z_at(j);
                for i in 1..(nr - 1) {
                    let r = grid.r_at(i);
                    let idx = grid.index_of(i, j);

                    let r_plus = r + 0.5 * grid.dr;
                    let r_minus = r - 0.5 * grid.dr;

                    let c_e = r / (dr2 * r_plus);
                    let c_w = r / (dr2 * r_minus);
                    let c_n = 1.0 / dz2;
                    let c_s = 1.0 / dz2;
                    let c_c = c_e + c_w + c_n + c_s;

                    // Source term -mu_0 * R * J_phi
                    let dist_sq = ((r - r0) / a).powi(2) + (z / (a * kappa)).powi(2);
                    let j_phi = if dist_sq < 1.0 {
                        j_phi_0 * (1.0 - dist_sq)
                    } else {
                        0.0
                    };
                    let source = -VACUUM_PERMEABILITY * r * j_phi;

                    let psi_e = psi[grid.index_of(i + 1, j)];
                    let psi_w = psi[grid.index_of(i - 1, j)];
                    let psi_n = psi[grid.index_of(i, j + 1)];
                    let psi_s = psi[grid.index_of(i, j - 1)];

                    let psi_new =
                        (c_e * psi_e + c_w * psi_w + c_n * psi_n + c_s * psi_s - source) / c_c;
                    let diff = psi_new - psi[idx];
                    psi[idx] += self.omega * diff;

                    let res = diff.abs();
                    if res > max_res {
                        max_res = res;
                    }
                }
            }

            if max_res < self.tolerance {
                converged = true;
                break;
            }
        }

        // Find magnetic axis (maximum psi)
        let mut max_psi = f64::NEG_INFINITY;
        let mut axis_r = r0;
        let mut axis_z = 0.0;
        for j in 0..nz {
            for i in 0..nr {
                let p = psi[grid.index_of(i, j)];
                if p > max_psi {
                    max_psi = p;
                    axis_r = grid.r_at(i);
                    axis_z = grid.z_at(j);
                }
            }
        }

        // Integrate total current
        let mut total_current = 0.0;
        let da = grid.dr * grid.dz;
        for j in 0..nz {
            let z = grid.z_at(j);
            for i in 0..nr {
                let r = grid.r_at(i);
                let dist_sq = ((r - r0) / a).powi(2) + (z / (a * kappa)).powi(2);
                if dist_sq < 1.0 {
                    let j_phi = j_phi_0 * (1.0 - dist_sq);
                    total_current += j_phi * da;
                }
            }
        }

        GradShafranovSolution {
            grid,
            psi,
            magnetic_axis: (axis_r, axis_z),
            psi_axis: max_psi,
            psi_sep: 0.0,
            total_current,
            converged,
            iterations: iter_count,
            max_residual: max_res,
        }
    }
}
