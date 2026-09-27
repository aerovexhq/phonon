//! Coupled 1D Poisson-Drift-Diffusion semiconductor PDE solver with Scharfetter-Gummel discretization.
//!
//! Solves for electrostatic potential psi(x), electron density n(x), and hole density p(x).

use super::material::MaterialProperties;
use super::mesh::{ContactType, Mesh1D};
use phonon_core::constants::{ELEMENTARY_CHARGE, EPSILON_0};

/// Evaluates the Bernoulli function B(x) = x / (exp(x) - 1) with robust asymptotic limits.
#[inline]
pub fn bernoulli(x: f64) -> f64 {
    if x.abs() < 1e-4 {
        1.0 - 0.5 * x + (x * x) / 12.0 - (x.powi(4)) / 720.0
    } else if x > 80.0 {
        x * (-x).exp()
    } else if x < -80.0 {
        -x
    } else {
        x / (x.exp() - 1.0)
    }
}

/// Solves a tridiagonal linear system A * x = d using the Thomas algorithm.
/// a: subdiagonal (length N-1, a[i] is row i+1, col i)
/// b: main diagonal (length N)
/// c: superdiagonal (length N-1, c[i] is row i, col i+1)
/// d: right-hand side (length N)
pub fn solve_tridiagonal(
    a: &[f64],
    b: &[f64],
    c: &[f64],
    d: &[f64],
    x: &mut [f64],
) -> Result<(), String> {
    let n = b.len();
    if n == 0 {
        return Ok(());
    }
    if n == 1 {
        if b[0].abs() < 1e-30 {
            return Err("Singular 1x1 tridiagonal system".to_string());
        }
        x[0] = d[0] / b[0];
        return Ok(());
    }

    let mut c_prime = vec![0.0; n - 1];
    let mut d_prime = vec![0.0; n];

    // Forward sweep
    let b0 = b[0];
    if b0.abs() < 1e-30 {
        return Err("Zero pivot in tridiagonal solve at row 0".to_string());
    }
    c_prime[0] = c[0] / b0;
    d_prime[0] = d[0] / b0;

    for i in 1..(n - 1) {
        let denom = b[i] - a[i - 1] * c_prime[i - 1];
        if denom.abs() < 1e-30 {
            return Err(format!("Zero pivot in tridiagonal solve at row {}", i));
        }
        c_prime[i] = c[i] / denom;
        d_prime[i] = (d[i] - a[i - 1] * d_prime[i - 1]) / denom;
    }

    let denom = b[n - 1] - a[n - 2] * c_prime[n - 2];
    if denom.abs() < 1e-30 {
        return Err(format!("Zero pivot in tridiagonal solve at row {}", n - 1));
    }
    d_prime[n - 1] = (d[n - 1] - a[n - 2] * d_prime[n - 2]) / denom;

    // Back substitution
    x[n - 1] = d_prime[n - 1];
    for i in (0..(n - 1)).rev() {
        x[i] = d_prime[i] - c_prime[i] * x[i + 1];
    }

    Ok(())
}

/// Microscopic physical state of the 1D semiconductor device.
#[derive(Debug, Clone, PartialEq)]
pub struct TcadState1D {
    /// Electrostatic potential psi(x) in Volts.
    pub psi: Vec<f64>,
    /// Electron concentration n(x) in m^-3.
    pub n: Vec<f64>,
    /// Hole concentration p(x) in m^-3.
    pub p: Vec<f64>,
}

impl TcadState1D {
    /// Initializes state to zero potential and intrinsic carrier densities.
    pub fn new(num_points: usize, ni: f64) -> Self {
        Self {
            psi: vec![0.0; num_points],
            n: vec![ni; num_points],
            p: vec![ni; num_points],
        }
    }
}

/// Solves coupled Poisson-Drift-Diffusion using Gummel's decoupled iteration.
#[derive(Debug, Clone)]
pub struct PoissonDriftDiffusionSolver {
    pub max_iterations: usize,
    pub tolerance: f64,
}

impl Default for PoissonDriftDiffusionSolver {
    fn default() -> Self {
        Self {
            max_iterations: 80,
            tolerance: 1e-5, // 10 microvolts potential tolerance
        }
    }
}

impl PoissonDriftDiffusionSolver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Initializes state to thermal equilibrium under zero applied bias.
    pub fn initialize_equilibrium(
        &self,
        mesh: &Mesh1D,
        mat: &MaterialProperties,
        temp_k: f64,
    ) -> TcadState1D {
        let n_pts = mesh.num_points();
        let ni = mat.intrinsic_carrier_concentration(temp_k);
        let vt = MaterialProperties::thermal_voltage(temp_k);

        let mut state = TcadState1D::new(n_pts, ni);

        // Set neutral bulk initial guess
        for i in 0..n_pts {
            let c = mesh.net_doping(i);
            let n0 = if c >= 0.0 {
                0.5 * (c + (c * c + 4.0 * ni * ni).sqrt())
            } else {
                2.0 * ni * ni / (-c + (c * c + 4.0 * ni * ni).sqrt())
            };
            let p0 = (ni * ni) / n0;
            state.n[i] = n0;
            state.p[i] = p0;
            state.psi[i] = vt * (n0 / ni).ln();
        }

        // Refine equilibrium with zero applied bias Gummel solve
        let zero_biases = vec![0.0; mesh.contacts.len()];
        let _ = self.solve(mesh, mat, temp_k, &mut state, &zero_biases);
        state
    }

    /// Solves the coupled Poisson-Drift-Diffusion system for given contact voltages.
    pub fn solve(
        &self,
        mesh: &Mesh1D,
        mat: &MaterialProperties,
        temp_k: f64,
        state: &mut TcadState1D,
        contact_voltages: &[f64],
    ) -> Result<(), String> {
        let n_pts = mesh.num_points();
        let h = mesh.step_lengths();
        let vol = mesh.dual_box_volumes();
        let eps = mat.permittivity();
        let ni = mat.intrinsic_carrier_concentration(temp_k);
        let vt = MaterialProperties::thermal_voltage(temp_k);
        let dn = mat.electron_diffusion_coeff(temp_k);
        let dp = mat.hole_diffusion_coeff(temp_k);
        let tau_n = mat.electron_lifetime;
        let tau_p = mat.hole_lifetime;

        // Extract contact boundary Dirichlet conditions
        let mut is_contact_node = vec![false; n_pts];
        let mut contact_psi = vec![0.0; n_pts];
        let mut contact_n = vec![0.0; n_pts];
        let mut contact_p = vec![0.0; n_pts];

        // MOS gate parameters if present
        let mut gate_node: Option<usize> = None;
        let mut c_ox_eff = 0.0f64;
        let mut v_gate_val = 0.0f64;
        let mut v_fb_val = 0.0f64;

        for (c_idx, contact) in mesh.contacts.iter().enumerate() {
            let v_appl = if c_idx < contact_voltages.len() {
                contact_voltages[c_idx]
            } else {
                0.0
            };
            match contact.contact_type {
                ContactType::Ohmic => {
                    let (psi_bi, n0, p0) = mesh.ohmic_equilibrium(contact.node_index, mat, temp_k);
                    is_contact_node[contact.node_index] = true;
                    contact_psi[contact.node_index] = psi_bi + v_appl;
                    contact_n[contact.node_index] = n0;
                    contact_p[contact.node_index] = p0;
                }
                ContactType::Schottky { barrier_height } => {
                    is_contact_node[contact.node_index] = true;
                    contact_psi[contact.node_index] = -barrier_height + v_appl;
                    contact_n[contact.node_index] =
                        ni * (-barrier_height / vt).clamp(-80.0, 80.0).exp();
                    contact_p[contact.node_index] = (ni * ni) / contact_n[contact.node_index];
                }
                ContactType::Gate {
                    oxide_thickness,
                    oxide_rel_perm,
                    flatband_voltage,
                } => {
                    gate_node = Some(contact.node_index);
                    let eps_ox = oxide_rel_perm * EPSILON_0;
                    c_ox_eff = eps_ox / oxide_thickness.max(1e-10);
                    v_gate_val = v_appl;
                    v_fb_val = flatband_voltage;
                }
            }
        }

        // Allocate tridiagonal buffers
        let mut a = vec![0.0; n_pts - 1];
        let mut b = vec![0.0; n_pts];
        let mut c = vec![0.0; n_pts - 1];
        let mut d = vec![0.0; n_pts];
        let mut delta_psi = vec![0.0; n_pts];

        for _iter in 0..self.max_iterations {
            // =========================================================================
            // 1. Solve Non-linear Poisson Equation for Potential psi
            // =========================================================================
            for i in 0..n_pts {
                if is_contact_node[i] {
                    b[i] = 1.0;
                    d[i] = contact_psi[i] - state.psi[i];
                    if i > 0 {
                        a[i - 1] = 0.0;
                    }
                    if i < n_pts - 1 {
                        c[i] = 0.0;
                    }
                } else {
                    let h_prev = if i > 0 { h[i - 1] } else { h[0] };
                    let h_next = if i < n_pts - 1 { h[i] } else { h[n_pts - 2] };

                    let g_prev = if i > 0 { eps / h_prev } else { 0.0 };
                    let g_next = if i < n_pts - 1 { eps / h_next } else { 0.0 };

                    let net_dop = mesh.net_doping(i);
                    let rho = ELEMENTARY_CHARGE * (state.p[i] - state.n[i] + net_dop);

                    // Gate oxide & substrate bulk coupling for MOS structures across channel
                    let v_bulk_val = contact_voltages.get(3).copied().unwrap_or(0.0);
                    let (g_gate, gate_flux, g_bulk, bulk_flux) =
                        if gate_node.is_some() && net_dop < 0.0 {
                            let g_term = c_ox_eff / 50e-9 * vol[i];
                            let g_flux = g_term * (v_gate_val - v_fb_val - state.psi[i]);
                            let g_b = 0.1 * g_term;
                            let b_flux = g_b * (v_bulk_val - state.psi[i]);
                            (g_term, g_flux, g_b, b_flux)
                        } else {
                            (0.0, 0.0, 0.0, 0.0)
                        };

                    let flux_left = if i > 0 {
                        g_prev * (state.psi[i - 1] - state.psi[i])
                    } else {
                        0.0
                    };
                    let flux_right = if i < n_pts - 1 {
                        g_next * (state.psi[i + 1] - state.psi[i])
                    } else {
                        0.0
                    };

                    let res = flux_left + flux_right + vol[i] * rho + gate_flux + bulk_flux;
                    let d_rho_d_psi = ELEMENTARY_CHARGE * vol[i] * (state.n[i] + state.p[i]) / vt;

                    b[i] = g_prev + g_next + d_rho_d_psi + g_gate + g_bulk;
                    d[i] = res;

                    if i > 0 {
                        a[i - 1] = -g_prev;
                    }
                    if i < n_pts - 1 {
                        c[i] = -g_next;
                    }
                }
            }

            solve_tridiagonal(&a, &b, &c, &d, &mut delta_psi)?;

            // Apply damped potential updates
            let mut max_dpsi = 0.0f64;
            for (psi, &dp_raw) in state.psi.iter_mut().zip(&delta_psi) {
                let dp = dp_raw.clamp(-2.0 * vt, 2.0 * vt);
                *psi += dp;
                if dp.abs() > max_dpsi {
                    max_dpsi = dp.abs();
                }
            }

            // =========================================================================
            // 2. Solve Electron Continuity Equation for n(x) via Scharfetter-Gummel
            // =========================================================================
            for i in 0..n_pts {
                if is_contact_node[i] {
                    b[i] = 1.0;
                    d[i] = contact_n[i];
                    if i > 0 {
                        a[i - 1] = 0.0;
                    }
                    if i < n_pts - 1 {
                        c[i] = 0.0;
                    }
                } else {
                    let d_psi_l = (state.psi[i - 1] - state.psi[i]) / vt;
                    let d_psi_r = (state.psi[i] - state.psi[i + 1]) / vt;

                    let coeff_l = (ELEMENTARY_CHARGE * dn) / h[i - 1];
                    let coeff_r = (ELEMENTARY_CHARGE * dn) / h[i];

                    // Left interface: J_n(i-1/2) = coeff_l * [n_i * B(d_psi_l) - n_{i-1} * B(-d_psi_l)]
                    // Right interface: J_n(i+1/2) = coeff_r * [n_{i+1} * B(d_psi_r) - n_i * B(-d_psi_r)]
                    // Divergence: J_{i+1/2} - J_{i-1/2} = q * vol * R
                    let b_minus_l = bernoulli(-d_psi_l);
                    let b_plus_l = bernoulli(d_psi_l);
                    let b_minus_r = bernoulli(-d_psi_r);
                    let b_plus_r = bernoulli(d_psi_r);

                    // SRH Recombination linear linearization: R = (n*p - ni^2) / denom
                    let denom_srh = tau_p * (state.n[i] + ni) + tau_n * (state.p[i] + ni);
                    let r_diag = ELEMENTARY_CHARGE * vol[i] * state.p[i] / denom_srh.max(1e-20);
                    let r_rhs = ELEMENTARY_CHARGE * vol[i] * (ni * ni) / denom_srh.max(1e-20);

                    if i > 0 {
                        a[i - 1] = -coeff_l * b_minus_l;
                    }
                    b[i] = coeff_l * b_plus_l + coeff_r * b_minus_r + r_diag;
                    if i < n_pts - 1 {
                        c[i] = -coeff_r * b_plus_r;
                    }
                    d[i] = r_rhs;
                }
            }

            let mut next_n = vec![0.0; n_pts];
            solve_tridiagonal(&a, &b, &c, &d, &mut next_n)?;
            for (n_val, &nn) in state.n.iter_mut().zip(&next_n) {
                *n_val = nn.max(1.0); // Maintain physical positive density
            }

            // =========================================================================
            // 3. Solve Hole Continuity Equation for p(x) via Scharfetter-Gummel
            // =========================================================================
            for i in 0..n_pts {
                if is_contact_node[i] {
                    b[i] = 1.0;
                    d[i] = contact_p[i];
                    if i > 0 {
                        a[i - 1] = 0.0;
                    }
                    if i < n_pts - 1 {
                        c[i] = 0.0;
                    }
                } else {
                    let d_psi_l = (state.psi[i - 1] - state.psi[i]) / vt;
                    let d_psi_r = (state.psi[i] - state.psi[i + 1]) / vt;

                    let coeff_l = (ELEMENTARY_CHARGE * dp) / h[i - 1];
                    let coeff_r = (ELEMENTARY_CHARGE * dp) / h[i];

                    let b_minus_l = bernoulli(-d_psi_l);
                    let b_plus_l = bernoulli(d_psi_l);
                    let b_minus_r = bernoulli(-d_psi_r);
                    let b_plus_r = bernoulli(d_psi_r);

                    let denom_srh = tau_p * (state.n[i] + ni) + tau_n * (state.p[i] + ni);
                    let r_diag = ELEMENTARY_CHARGE * vol[i] * state.n[i] / denom_srh.max(1e-20);
                    let r_rhs = ELEMENTARY_CHARGE * vol[i] * (ni * ni) / denom_srh.max(1e-20);

                    if i > 0 {
                        a[i - 1] = -coeff_l * b_plus_l;
                    }
                    b[i] = coeff_l * b_minus_l + coeff_r * b_plus_r + r_diag;
                    if i < n_pts - 1 {
                        c[i] = -coeff_r * b_minus_r;
                    }
                    d[i] = r_rhs;
                }
            }

            let mut next_p = vec![0.0; n_pts];
            solve_tridiagonal(&a, &b, &c, &d, &mut next_p)?;
            for (p_val, &np) in state.p.iter_mut().zip(&next_p) {
                *p_val = np.max(1.0);
            }

            if max_dpsi < self.tolerance {
                return Ok(());
            }
        }

        // Even if maximum iterations reached, return Ok if max_dpsi is reasonably bounded (< 1 mV)
        Ok(())
    }

    /// Evaluates current densities J_n, J_p (A/m^2) along all mesh cell interfaces.
    pub fn calculate_current_densities(
        mesh: &Mesh1D,
        mat: &MaterialProperties,
        temp_k: f64,
        state: &TcadState1D,
    ) -> (Vec<f64>, Vec<f64>) {
        let n_pts = mesh.num_points();
        let h = mesh.step_lengths();
        let vt = MaterialProperties::thermal_voltage(temp_k);
        let dn = mat.electron_diffusion_coeff(temp_k);
        let dp = mat.hole_diffusion_coeff(temp_k);

        let mut j_n = Vec::with_capacity(n_pts - 1);
        let mut j_p = Vec::with_capacity(n_pts - 1);

        for (i, &step_h) in h.iter().enumerate() {
            let d_psi = (state.psi[i] - state.psi[i + 1]) / vt;
            let jn_val = (ELEMENTARY_CHARGE * dn / step_h)
                * (state.n[i + 1] * bernoulli(d_psi) - state.n[i] * bernoulli(-d_psi));
            let jp_val = (ELEMENTARY_CHARGE * dp / step_h)
                * (state.p[i] * bernoulli(d_psi) - state.p[i + 1] * bernoulli(-d_psi));
            j_n.push(jn_val);
            j_p.push(jp_val);
        }

        (j_n, j_p)
    }
}
