//! Non-Hermitian Skin Effect (NHSE) lattice model,
//! Generalized Brillouin Zone (GBZ), and point-gap topological winding.
//!
//! # Physical Formalism
//! - Non-Reciprocal Hopping:
//!   $$t_R = t_0 e^{+\gamma}, \quad t_L = t_0 e^{-\gamma}$$
//! - Generalized Brillouin Zone (GBZ) Radius:
//!   $$r_{\mathrm{GBZ}} = \sqrt{\frac{t_L}{t_R}} = e^{-\gamma} < 1$$
//! - Spatial Skin Localization Factor:
//!   $$\Lambda_{\mathrm{skin}} = 20 \log_{10}\left( \frac{|\psi(0)|}{|\psi(L-1)|} \right) = 20 \gamma (L-1) \log_{10}(e) \approx 8.68589 \gamma (L-1)\text{ dB}$$
//! - Point-Gap Spectral Winding Number:
//!   $$W(E_B) = \oint_{k \in [0, 2\pi]} \frac{dk}{2\pi i} \frac{d}{dk} \ln[E(k) - E_B] = \pm 1$$

use std::f64::consts::PI;

/// Parameters for a 1D non-Hermitian acoustic lattice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NhseLatticeParams {
    /// Number of lattice sites $L$ (typically 15 to 60).
    pub num_sites: usize,
    /// Base reciprocal coupling $t_0$ in rad/s (e.g. 1000.0).
    pub base_coupling_rad_s: f64,
    /// Non-reciprocal asymmetry parameter $\gamma > 0$ (typically 0.1 to 0.4).
    pub non_reciprocal_gamma: f64,
    /// On-site resonant frequency $\omega_0$ in rad/s.
    pub onsite_frequency_rad_s: f64,
}

impl NhseLatticeParams {
    /// Standard acoustic non-Hermitian lattice with $L = 20$ sites.
    pub fn standard_chain() -> Self {
        Self {
            num_sites: 20,
            base_coupling_rad_s: 1000.0,
            non_reciprocal_gamma: 0.25,
            onsite_frequency_rad_s: 5000.0,
        }
    }

    /// Forward hopping rate $t_R = t_0 e^{+\gamma}$.
    pub fn forward_hopping(&self) -> f64 {
        self.base_coupling_rad_s * self.non_reciprocal_gamma.exp()
    }

    /// Backward hopping rate $t_L = t_0 e^{-\gamma}$.
    pub fn backward_hopping(&self) -> f64 {
        self.base_coupling_rad_s * (-self.non_reciprocal_gamma).exp()
    }

    /// Radius of the Generalized Brillouin Zone (GBZ) in the complex plane $z = r e^{ik}$.
    pub fn gbz_radius(&self) -> f64 {
        (-self.non_reciprocal_gamma).exp()
    }

    /// Spatial skin localization factor $\Lambda_{\mathrm{skin}}$ in decibels ($\\ge 30\\text{ dB}$ required).
    pub fn skin_localization_factor_db(&self) -> f64 {
        let l = (self.num_sites - 1) as f64;
        20.0 * self.non_reciprocal_gamma * l * std::f64::consts::LOG10_E
    }
}

/// Point-gap topological winding evaluation under Periodic Boundary Conditions (PBC).
#[derive(Debug, Clone, PartialEq)]
pub struct PointGapWinding {
    /// Reference energy $E_B$ in the complex plane (real, imag).
    pub base_energy: (f64, f64),
    /// Topological integer winding number $W(E_B) \in \mathbb{Z}$.
    pub winding_number: i32,
    /// Minimum point-gap distance $\min_k |E(k) - E_B|$.
    pub min_point_gap: f64,
}

/// 1D Non-Hermitian Skin Effect lattice model.
#[derive(Debug, Clone, PartialEq)]
pub struct NhseLattice {
    pub params: NhseLatticeParams,
}

impl NhseLattice {
    pub fn new(params: NhseLatticeParams) -> Self {
        Self { params }
    }

    /// Evaluates the complex Bloch dispersion $E(k)$ under periodic boundary conditions:
    /// $$E(k) = \omega_0 + t_R e^{ik} + t_L e^{-ik}$$
    pub fn evaluate_bloch_dispersion(&self, k: f64) -> (f64, f64) {
        let tr = self.params.forward_hopping();
        let tl = self.params.backward_hopping();
        let w0 = self.params.onsite_frequency_rad_s;

        // E(k) = w0 + (tr + tl) cos(k) + i (tr - tl) sin(k)
        let real = w0 + (tr + tl) * k.cos();
        let imag = (tr - tl) * k.sin();
        (real, imag)
    }

    /// Computes the point-gap winding number with respect to reference energy $E_B = (E_r, E_i)$.
    pub fn compute_point_gap_winding(
        &self,
        eb_real: f64,
        eb_imag: f64,
        steps: usize,
    ) -> PointGapWinding {
        let mut total_phase_change = 0.0;
        let mut prev_arg = 0.0;
        let mut min_gap = f64::MAX;

        for i in 0..=steps {
            let k = 2.0 * PI * (i as f64) / (steps as f64);
            let (er, ei) = self.evaluate_bloch_dispersion(k);
            let dr = er - eb_real;
            let di = ei - eb_imag;
            let dist = (dr.powi(2) + di.powi(2)).sqrt();
            if dist < min_gap {
                min_gap = dist;
            }

            let arg = di.atan2(dr);
            if i > 0 {
                let mut d_arg = arg - prev_arg;
                // Unwrap phase jumps across +/- pi
                while d_arg > PI {
                    d_arg -= 2.0 * PI;
                }
                while d_arg < -PI {
                    d_arg += 2.0 * PI;
                }
                total_phase_change += d_arg;
            }
            prev_arg = arg;
        }

        let winding = (total_phase_change / (2.0 * PI)).round() as i32;

        PointGapWinding {
            base_energy: (eb_real, eb_imag),
            winding_number: winding,
            min_point_gap: min_gap,
        }
    }

    /// Evaluates the normalized spatial skin wavepacket amplitude at site $x \in \{0, \dots, L-1\}$:
    /// $$\psi(x) \propto e^{-\gamma x}$$
    pub fn skin_wavepacket_profile(&self) -> Vec<f64> {
        let l = self.params.num_sites;
        let gamma = self.params.non_reciprocal_gamma;
        let mut raw: Vec<f64> = (0..l).map(|x| (-gamma * (x as f64)).exp()).collect();

        let norm_sq: f64 = raw.iter().map(|v| v * v).sum();
        let norm = norm_sq.sqrt().max(1e-15);
        for v in &mut raw {
            *v /= norm;
        }
        raw
    }
}
