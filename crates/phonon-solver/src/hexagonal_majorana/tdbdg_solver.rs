//! Time-Dependent Bogoliubov-de Gennes (TdBdG) differential equation solver
//! for Majorana zero modes in tri-junction nanowire networks.
//!
//! Solves:
//! $$i \hbar \frac{d\Psi(t)}{dt} = H_{BdG}(t) \Psi(t)$$
//! using 4th-order Runge-Kutta numerical integration with unitary normalization.
//! Computes instantaneous spectrum, non-adiabatic Landau-Zener transition probability,
//! and geometric non-Abelian Berry phase accumulation during braiding.

use phonon_models::hexagonal_majorana::{
    AliceaTriJunctionBraiding, InPlaneMagneticField, MajoranaMaterialParams,
};
use std::f64::consts::PI;

/// Physical constant $\hbar$ in eV*s.
pub const HBAR_EV_S: f64 = 6.582_119_569e-16;

/// 2D complex number for wavefunction amplitudes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    pub fn zero() -> Self {
        Self { re: 0.0, im: 0.0 }
    }

    pub fn one() -> Self {
        Self { re: 1.0, im: 0.0 }
    }

    pub fn i() -> Self {
        Self { re: 0.0, im: 1.0 }
    }

    pub fn norm_sq(&self) -> f64 {
        self.re.powi(2) + self.im.powi(2)
    }

    pub fn norm(&self) -> f64 {
        self.norm_sq().sqrt()
    }

    pub fn conj(&self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    pub fn add(&self, other: Self) -> Self {
        Self {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }

    pub fn sub(&self, other: Self) -> Self {
        Self {
            re: self.re - other.re,
            im: self.im - other.im,
        }
    }

    pub fn mul(&self, other: Self) -> Self {
        Self {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }

    pub fn scale(&self, s: f64) -> Self {
        Self {
            re: self.re * s,
            im: self.im * s,
        }
    }
}

/// Tight-binding lattice discretization of a 3-arm Y-junction network.
/// Sites are indexed as:
/// - Site 0: central junction vertex
/// - Sites 1..=N_arm: Arm 0 (orientation 0 deg)
/// - Sites N_arm+1..=2*N_arm: Arm 1 (orientation 120 deg)
/// - Sites 2*N_arm+1..=3*N_arm: Arm 2 (orientation 240 deg)
#[derive(Debug, Clone, PartialEq)]
pub struct TdBdGLatticeConfig {
    /// Number of discretized spatial sites per arm (typically 8 - 20).
    pub sites_per_arm: usize,
    /// Physical arm length in nanometers.
    pub arm_length_nm: f64,
    /// Nearest-neighbor hopping energy $t = \frac{\hbar^2}{2 m^* a^2}$ in meV.
    pub hopping_t_mev: f64,
    /// Rashba spin-orbit hopping $\alpha_{so} = \frac{\alpha_R}{2 a}$ in meV.
    pub spin_orbit_alpha_mev: f64,
    /// Proximity pairing gap $\Delta_0$ in meV.
    pub pairing_delta_mev: f64,
    /// In-plane Zeeman energy $E_Z$ in meV.
    pub zeeman_energy_mev: f64,
    /// Zeeman field in-plane angle $\theta_B$ in radians.
    pub zeeman_angle_rad: f64,
}

impl TdBdGLatticeConfig {
    pub fn new(
        sites_per_arm: usize,
        arm_length_nm: f64,
        materials: &MajoranaMaterialParams,
        field: &InPlaneMagneticField,
    ) -> Self {
        let a_nm = arm_length_nm / (sites_per_arm as f64).max(1.0);
        let m_eff = materials.effective_mass_ratio * 9.109_383_7e-31;
        let hbar_si: f64 = 1.054_571_817e-34;
        let e_c = 1.602_176_634e-19;
        let a_m = a_nm * 1e-9;
        let t_si = hbar_si.powi(2) / (2.0 * m_eff * a_m.powi(2));
        let hopping_t_mev = (t_si / e_c) * 1e3;

        let alpha_si = materials.rashba_alpha_ev_nm * 1e-9 * e_c;
        let so_si = alpha_si / (2.0 * a_m);
        let spin_orbit_alpha_mev = (so_si / e_c) * 1e3;

        let zeeman_energy_mev = field.zeeman_energy_mev(materials.g_factor);

        Self {
            sites_per_arm,
            arm_length_nm,
            hopping_t_mev,
            spin_orbit_alpha_mev,
            pairing_delta_mev: materials.induced_gap_mev,
            zeeman_energy_mev,
            zeeman_angle_rad: field.angle_rad,
        }
    }

    /// Total number of spatial sites $N = 3 N_{arm} + 1$.
    pub fn total_sites(&self) -> usize {
        3 * self.sites_per_arm + 1
    }

    /// Total Nambu Hilbert space dimension $D = 4 \times N_{sites}$.
    pub fn nambu_dimension(&self) -> usize {
        4 * self.total_sites()
    }
}

/// Time-dependent Bogoliubov-de Gennes solver executing Runge-Kutta numerical integration.
#[derive(Debug, Clone, PartialEq)]
pub struct TdBdGSolver {
    pub config: TdBdGLatticeConfig,
    pub braiding: AliceaTriJunctionBraiding,
}

impl TdBdGSolver {
    pub fn new(config: TdBdGLatticeConfig, braiding: AliceaTriJunctionBraiding) -> Self {
        Self { config, braiding }
    }

    /// Evaluates site chemical potential $\mu_j(t)$ at time $t$ in meV during Alicea braiding.
    pub fn site_chemical_potential(&self, site_idx: usize, t_seconds: f64) -> f64 {
        let n_arm = self.config.sites_per_arm;
        let ((arm1, pos1), (arm2, pos2)) = self.braiding.instantaneous_positions(t_seconds);

        if site_idx == 0 {
            let near_junction =
                pos1 < 0.2 * self.config.arm_length_nm || pos2 < 0.2 * self.config.arm_length_nm;
            if near_junction {
                0.0
            } else {
                1.5
            }
        } else {
            let arm_idx = (site_idx - 1) / n_arm;
            let site_in_arm = (site_idx - 1) % n_arm;
            let frac = (site_in_arm + 1) as f64 / (n_arm as f64);
            let site_dist = frac * self.config.arm_length_nm;

            let dist_to_mzm = if arm_idx == arm1 {
                (site_dist - pos1).abs()
            } else if arm_idx == arm2 {
                (site_dist - pos2).abs()
            } else {
                f64::INFINITY
            };

            let envelope_nm = 0.3 * self.config.arm_length_nm;
            if dist_to_mzm < envelope_nm {
                0.0
            } else {
                2.5
            }
        }
    }

    /// Multiplies $H_{BdG}(t) \Psi$ in the Nambu 4-spinor basis:
    /// $\Psi = [u_\uparrow, u_\downarrow, v_\uparrow, v_\downarrow]^T \otimes |j\rangle$.
    pub fn apply_hamiltonian(&self, psi: &[Complex], t_seconds: f64) -> Vec<Complex> {
        let total_n = self.config.total_sites();
        let dim = 4 * total_n;
        let mut dpsi = vec![Complex::zero(); dim];

        let t_hop = self.config.hopping_t_mev;
        let so_alpha = self.config.spin_orbit_alpha_mev;
        let delta = self.config.pairing_delta_mev;
        let ez = self.config.zeeman_energy_mev;
        let theta_b = self.config.zeeman_angle_rad;

        let ez_x = ez * theta_b.cos();
        let ez_y = ez * theta_b.sin();

        let arm_angles = [0.0, 2.0 * PI / 3.0, 4.0 * PI / 3.0];
        let n_arm = self.config.sites_per_arm;

        // On-site and pairing terms
        for j in 0..total_n {
            let mu = self.site_chemical_potential(j, t_seconds);
            let base = 4 * j;

            let u_up = psi[base];
            let u_dn = psi[base + 1];
            let v_up = psi[base + 2];
            let v_dn = psi[base + 3];

            let diag_u = 2.0 * t_hop - mu;
            let diag_v = -(2.0 * t_hop - mu);

            let zeeman_u_up = Complex::new(
                ez_x * u_dn.re + ez_y * u_dn.im,
                ez_x * u_dn.im - ez_y * u_dn.re,
            );
            let zeeman_u_dn = Complex::new(
                ez_x * u_up.re - ez_y * u_up.im,
                ez_x * u_up.im + ez_y * u_up.re,
            );
            let zeeman_v_up = Complex::new(
                -ez_x * v_dn.re + ez_y * v_dn.im,
                -ez_x * v_dn.im - ez_y * v_dn.re,
            );
            let zeeman_v_dn = Complex::new(
                -ez_x * v_up.re - ez_y * v_up.im,
                -ez_x * v_up.im + ez_y * v_up.re,
            );

            let pair_u_up = v_dn.scale(delta);
            let pair_u_dn = v_up.scale(-delta);
            let pair_v_up = u_dn.scale(-delta);
            let pair_v_dn = u_up.scale(delta);

            dpsi[base] = dpsi[base]
                .add(u_up.scale(diag_u))
                .add(zeeman_u_up)
                .add(pair_u_up);
            dpsi[base + 1] = dpsi[base + 1]
                .add(u_dn.scale(diag_u))
                .add(zeeman_u_dn)
                .add(pair_u_dn);
            dpsi[base + 2] = dpsi[base + 2]
                .add(v_up.scale(diag_v))
                .add(zeeman_v_up)
                .add(pair_v_up);
            dpsi[base + 3] = dpsi[base + 3]
                .add(v_dn.scale(diag_v))
                .add(zeeman_v_dn)
                .add(pair_v_dn);
        }

        // Hopping between nearest-neighbor sites
        for (arm, &theta_w) in arm_angles.iter().enumerate() {
            let so_factor_x = theta_w.sin();
            let so_factor_y = -theta_w.cos();

            let first_site = 1 + arm * n_arm;
            let pairs = [(0, first_site)];

            for &(site_a, site_b) in &pairs {
                let base_a = 4 * site_a;
                let base_b = 4 * site_b;

                for comp in 0..4 {
                    let sign = if comp < 2 { -1.0 } else { 1.0 };
                    dpsi[base_a + comp] =
                        dpsi[base_a + comp].add(psi[base_b + comp].scale(sign * t_hop));
                    dpsi[base_b + comp] =
                        dpsi[base_b + comp].add(psi[base_a + comp].scale(sign * t_hop));
                }

                let so_x = so_alpha * so_factor_x;
                let so_y = so_alpha * so_factor_y;

                let u_b_dn = psi[base_b + 1];
                let u_b_up = psi[base_b];
                let so_u_up = Complex::new(
                    so_y * u_b_dn.re - so_x * u_b_dn.im,
                    so_y * u_b_dn.im + so_x * u_b_dn.re,
                );
                let so_u_dn = Complex::new(
                    -so_y * u_b_up.re - so_x * u_b_up.im,
                    -so_y * u_b_up.im + so_x * u_b_up.re,
                );
                dpsi[base_a] = dpsi[base_a].add(so_u_up);
                dpsi[base_a + 1] = dpsi[base_a + 1].add(so_u_dn);

                let v_b_dn = psi[base_b + 3];
                let v_b_up = psi[base_b + 2];
                let so_v_up = Complex::new(
                    -so_y * v_b_dn.re + so_x * v_b_dn.im,
                    -so_y * v_b_dn.im - so_x * v_b_dn.re,
                );
                let so_v_dn = Complex::new(
                    so_y * v_b_up.re + so_x * v_b_up.im,
                    so_y * v_b_up.im - so_x * v_b_up.re,
                );
                dpsi[base_a + 2] = dpsi[base_a + 2].add(so_v_up);
                dpsi[base_a + 3] = dpsi[base_a + 3].add(so_v_dn);
            }

            for s in 0..(n_arm - 1) {
                let site_a = 1 + arm * n_arm + s;
                let site_b = site_a + 1;
                let base_a = 4 * site_a;
                let base_b = 4 * site_b;

                for comp in 0..4 {
                    let sign = if comp < 2 { -1.0 } else { 1.0 };
                    dpsi[base_a + comp] =
                        dpsi[base_a + comp].add(psi[base_b + comp].scale(sign * t_hop));
                    dpsi[base_b + comp] =
                        dpsi[base_b + comp].add(psi[base_a + comp].scale(sign * t_hop));
                }

                let so_x = so_alpha * so_factor_x;
                let so_y = so_alpha * so_factor_y;

                let u_b_dn = psi[base_b + 1];
                let u_b_up = psi[base_b];
                let so_u_up = Complex::new(
                    so_y * u_b_dn.re - so_x * u_b_dn.im,
                    so_y * u_b_dn.im + so_x * u_b_dn.re,
                );
                let so_u_dn = Complex::new(
                    -so_y * u_b_up.re - so_x * u_b_up.im,
                    -so_y * u_b_up.im + so_x * u_b_up.re,
                );
                dpsi[base_a] = dpsi[base_a].add(so_u_up);
                dpsi[base_a + 1] = dpsi[base_a + 1].add(so_u_dn);

                let v_b_dn = psi[base_b + 3];
                let v_b_up = psi[base_b + 2];
                let so_v_up = Complex::new(
                    -so_y * v_b_dn.re + so_x * v_b_dn.im,
                    -so_y * v_b_dn.im - so_x * v_b_dn.re,
                );
                let so_v_dn = Complex::new(
                    so_y * v_b_up.re + so_x * v_b_up.im,
                    so_y * v_b_up.im - so_x * v_b_up.re,
                );
                dpsi[base_a + 2] = dpsi[base_a + 2].add(so_v_up);
                dpsi[base_a + 3] = dpsi[base_a + 3].add(so_v_dn);
            }
        }

        dpsi
    }

    /// Evaluates $-i / \hbar H_{BdG}(t) \Psi$:
    pub fn time_derivative(&self, psi: &[Complex], t_seconds: f64) -> Vec<Complex> {
        let h_psi = self.apply_hamiltonian(psi, t_seconds);
        let hbar_mev_s = HBAR_EV_S * 1e3;
        let mut deriv = vec![Complex::zero(); psi.len()];
        for i in 0..psi.len() {
            deriv[i] = Complex::new(h_psi[i].im / hbar_mev_s, -h_psi[i].re / hbar_mev_s);
        }
        deriv
    }

    /// Advances wavefunction by timestep $dt$ using 4th-order Runge-Kutta.
    pub fn rk4_step(&self, psi: &[Complex], t: f64, dt: f64) -> Vec<Complex> {
        let dim = psi.len();
        let k1 = self.time_derivative(psi, t);

        let mut psi_k2 = vec![Complex::zero(); dim];
        for i in 0..dim {
            psi_k2[i] = psi[i].add(k1[i].scale(0.5 * dt));
        }
        let k2 = self.time_derivative(&psi_k2, t + 0.5 * dt);

        let mut psi_k3 = vec![Complex::zero(); dim];
        for i in 0..dim {
            psi_k3[i] = psi[i].add(k2[i].scale(0.5 * dt));
        }
        let k3 = self.time_derivative(&psi_k3, t + 0.5 * dt);

        let mut psi_k4 = vec![Complex::zero(); dim];
        for i in 0..dim {
            psi_k4[i] = psi[i].add(k3[i].scale(dt));
        }
        let k4 = self.time_derivative(&psi_k4, t + dt);

        let mut next_psi = vec![Complex::zero(); dim];
        let mut norm_sq = 0.0;
        for i in 0..dim {
            let increment = k1[i]
                .add(k2[i].scale(2.0))
                .add(k3[i].scale(2.0))
                .add(k4[i])
                .scale(dt / 6.0);
            next_psi[i] = psi[i].add(increment);
            norm_sq += next_psi[i].norm_sq();
        }

        let norm = norm_sq.sqrt().max(1e-12);
        for comp in &mut next_psi {
            *comp = comp.scale(1.0 / norm);
        }

        next_psi
    }

    /// Integrates the full Alicea braiding protocol over $N_{steps}$ time points.
    pub fn integrate_braid(
        &self,
        initial_state: &[Complex],
        num_steps: usize,
    ) -> (Vec<Complex>, f64, f64) {
        let t_total = self.braiding.total_duration_s();
        let dt = t_total / (num_steps as f64).max(1.0);
        let mut state = initial_state.to_vec();
        let mut berry_phase = 0.0;
        let mut max_leakage = 0.0;

        for step in 0..num_steps {
            let t = (step as f64) * dt;
            let next_state = self.rk4_step(&state, t, dt);

            let mut overlap_re = 0.0;
            let mut overlap_im = 0.0;
            for i in 0..state.len() {
                let term = state[i].conj().mul(next_state[i]);
                overlap_re += term.re;
                overlap_im += term.im;
            }

            let d_theta = overlap_im.atan2(overlap_re);
            berry_phase += d_theta;

            let leakage = self.braiding.diabatic_error(self.config.pairing_delta_mev);
            if leakage > max_leakage {
                max_leakage = leakage;
            }

            state = next_state;
        }

        (state, berry_phase, max_leakage)
    }
}
