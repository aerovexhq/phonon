//! High-frequency Magnus expansion solver and unitary time-evolution propagator
//! for Floquet topological bandstructures.

use phonon_core::constants::{ELEMENTARY_CHARGE, H_BAR};
use phonon_models::floquet_topological::{
    FloquetBandState, FloquetDiracMaterial, FloquetDriveParams,
};

/// High-frequency Floquet-Magnus solver and unitary Floquet propagator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetMagnusSolver {
    pub material: FloquetDiracMaterial,
    pub drive: FloquetDriveParams,
}

impl FloquetMagnusSolver {
    /// Creates a new Floquet-Magnus solver with given material and drive parameters.
    pub fn new(material: FloquetDiracMaterial, drive: FloquetDriveParams) -> Self {
        Self { material, drive }
    }

    /// Evaluates the quasi-energies and topological state at momentum $\mathbf{p} = (p_x, p_y)$.
    pub fn solve_quasienergies(&self, px_j_s_m: f64, py_j_s_m: f64) -> FloquetBandState {
        self.material
            .evaluate_band_state(&self.drive, px_j_s_m, py_j_s_m)
    }

    /// Solves the dynamic Floquet bandgap opening at the Dirac point in $\text{meV}$.
    pub fn solve_dynamic_gap_mev(&self) -> f64 {
        self.material.dynamic_gap_ev(&self.drive) * 1000.0
    }

    /// Computes the momentum-dependent Berry curvature $\Omega_z(\mathbf{p})$ in $\text{m}^2$:
    /// $$\Omega_z(\mathbf{p}) = -\frac{1}{2} \frac{\hbar^2 v_F^2 M_{\mathrm{eff}} \cdot e}{((v_F p)^2 + (M_{\mathrm{eff}} e)^2)^{3/2}}$$
    pub fn solve_berry_curvature(&self, px_j_s_m: f64, py_j_s_m: f64) -> f64 {
        let m_eff_j = self.material.effective_mass_ev(&self.drive) * ELEMENTARY_CHARGE;
        if m_eff_j.abs() < 1e-40 {
            return 0.0;
        }
        let vf = self.material.fermi_velocity_m_s;
        let vp_sq = (vf * px_j_s_m).powi(2) + (vf * py_j_s_m).powi(2);
        let denom = (vp_sq + m_eff_j.powi(2)).powf(1.5).max(1e-60);
        -0.5 * (H_BAR * vf).powi(2) * m_eff_j / denom
    }

    #[inline]
    fn cmul(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
        (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
    }

    #[inline]
    fn cadd(a: (f64, f64), b: (f64, f64)) -> (f64, f64) {
        (a.0 + b.0, a.1 + b.1)
    }

    /// Complex $2 \times 2$ matrix multiplication: $C = A \times B$.
    fn mat2_mul(a: [[(f64, f64); 2]; 2], b: [[(f64, f64); 2]; 2]) -> [[(f64, f64); 2]; 2] {
        let c00 = Self::cadd(Self::cmul(a[0][0], b[0][0]), Self::cmul(a[0][1], b[1][0]));
        let c01 = Self::cadd(Self::cmul(a[0][0], b[0][1]), Self::cmul(a[0][1], b[1][1]));
        let c10 = Self::cadd(Self::cmul(a[1][0], b[0][0]), Self::cmul(a[1][1], b[1][0]));
        let c11 = Self::cadd(Self::cmul(a[1][0], b[0][1]), Self::cmul(a[1][1], b[1][1]));
        [[c00, c01], [c10, c11]]
    }

    /// Numerically integrates the exact unitary time-evolution operator over one optical period $T$:
    /// $$U(T) = \mathcal{T} \exp\left( -\frac{i}{\hbar} \int_0^T H(t) dt \right) = \prod_{k=0}^{N-1} \exp\left( -\frac{i}{\hbar} H(t_k) \Delta t \right)$$
    pub fn compute_one_period_propagator(
        &self,
        num_time_steps: usize,
        px_j_s_m: f64,
        py_j_s_m: f64,
    ) -> [[(f64, f64); 2]; 2] {
        let period = self.drive.optical_period_s();
        let n_steps = num_time_steps.max(16);
        let dt = period / (n_steps as f64);

        let mut u_total: [[(f64, f64); 2]; 2] =
            [[(1.0, 0.0), (0.0, 0.0)], [(0.0, 0.0), (1.0, 0.0)]];

        let vf = self.material.fermi_velocity_m_s;
        let delta_half_j = 0.5 * self.material.intrinsic_gap_ev * ELEMENTARY_CHARGE;

        for k in 0..n_steps {
            let t_mid = (k as f64 + 0.5) * dt;
            let (ax, ay) = self.drive.vector_potential_at(t_mid);

            let kx = px_j_s_m + ELEMENTARY_CHARGE * ax;
            let ky = py_j_s_m + ELEMENTARY_CHARGE * ay;

            let dx = vf * kx;
            let dy = vf * ky;
            let dz = delta_half_j;

            let d_norm = (dx.powi(2) + dy.powi(2) + dz.powi(2)).sqrt();
            let theta = d_norm * dt / H_BAR;

            let cos_th = theta.cos();
            let sin_th_over_d = if d_norm > 1e-45 {
                theta.sin() / d_norm
            } else {
                dt / H_BAR
            };

            let u00 = (cos_th, -sin_th_over_d * dz);
            let u11 = (cos_th, sin_th_over_d * dz);
            let u01 = (-sin_th_over_d * dy, -sin_th_over_d * dx);
            let u10 = (sin_th_over_d * dy, -sin_th_over_d * dx);

            let u_slice = [[u00, u01], [u10, u11]];
            u_total = Self::mat2_mul(u_slice, u_total);
        }

        u_total
    }

    /// Evaluates the Frobenius norm error of unitarity: $\|U^\dagger U - I\|_F$.
    pub fn unitarity_error(&self, u: [[(f64, f64); 2]; 2]) -> f64 {
        let ud = [
            [(u[0][0].0, -u[0][0].1), (u[1][0].0, -u[1][0].1)],
            [(u[0][1].0, -u[0][1].1), (u[1][1].0, -u[1][1].1)],
        ];
        let prod = Self::mat2_mul(ud, u);

        let err00 = (prod[0][0].0 - 1.0).hypot(prod[0][0].1);
        let err01 = prod[0][1].0.hypot(prod[0][1].1);
        let err10 = prod[1][0].0.hypot(prod[1][0].1);
        let err11 = (prod[1][1].0 - 1.0).hypot(prod[1][1].1);

        (err00.powi(2) + err01.powi(2) + err10.powi(2) + err11.powi(2)).sqrt()
    }

    /// Extracts the quasi-energies $(\epsilon_+, \epsilon_-)$ in $\text{eV}$ from the numerical propagator $U(T)$.
    pub fn extract_quasienergies(&self, u: [[(f64, f64); 2]; 2]) -> (f64, f64) {
        let tr_re = 0.5 * (u[0][0].0 + u[1][1].0);
        let cos_phi = tr_re.clamp(-1.0, 1.0);
        let phi = cos_phi.acos();

        let period = self.drive.optical_period_s();
        let energy_j = (H_BAR * phi) / period;
        let energy_ev = energy_j / ELEMENTARY_CHARGE;
        (energy_ev, -energy_ev)
    }
}
