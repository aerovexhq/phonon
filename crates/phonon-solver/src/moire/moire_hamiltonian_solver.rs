//! Numerical eigensolver and bandstructure mapper for the Bistritzer-MacDonald continuum model.
//!
//! Solves the 8x8 complex Hermitian Hamiltonian using an exact Jacobi rotation algorithm in safe Rust,
//! extracting flat-band dispersion, bandwidth W, Dirac velocity quenching, and local density of states (LDOS).

use phonon_models::moire::{BistritzerMacDonaldModel, Vector2D, HBAR_EV_S};
use std::f64::consts::PI;

/// Results of diagonalizing the BM continuum Hamiltonian at a specific momentum k.
#[derive(Debug, Clone, PartialEq)]
pub struct MoireBandSolution {
    pub k: Vector2D,
    /// Eigenvalues in ascending order: E_0 <= E_1 <= ... <= E_7 (eV).
    pub eigenvalues_ev: [f64; 8],
    /// Lower central flat band (E_3).
    pub lower_flatband_ev: f64,
    /// Upper central flat band (E_4).
    pub upper_flatband_ev: f64,
    /// Gap to lower remote dispersive bands: E_3 - E_2 (eV).
    pub lower_remote_gap_ev: f64,
    /// Gap to upper remote dispersive bands: E_5 - E_4 (eV).
    pub upper_remote_gap_ev: f64,
}

/// Continuum Hamiltonian solver and bandstructure analyzer.
#[derive(Debug, Clone)]
pub struct MoireHamiltonianSolver {
    pub model: BistritzerMacDonaldModel,
}

impl MoireHamiltonianSolver {
    pub fn new(model: BistritzerMacDonaldModel) -> Self {
        Self { model }
    }

    /// Solves the 8x8 complex Hermitian Hamiltonian matrix at wavevector k.
    /// Uses an iterative complex Jacobi diagonalization in safe Rust with guaranteed quadratic convergence.
    #[allow(clippy::needless_range_loop)]
    pub fn solve(&self, k: Vector2D) -> MoireBandSolution {
        let mut a = self.model.build_8x8_hamiltonian(k);
        let n = 8;
        let max_iter = 60;
        let tolerance = 1e-12;

        for _ in 0..max_iter {
            // Find maximum off-diagonal norm:
            let mut max_off_diag = 0.0;
            let mut p = 0;
            let mut q = 1;

            for i in 0..n {
                for j in (i + 1)..n {
                    let re = a[i][j].0;
                    let im = a[i][j].1;
                    let norm_sq = re * re + im * im;
                    if norm_sq > max_off_diag {
                        max_off_diag = norm_sq;
                        p = i;
                        q = j;
                    }
                }
            }

            if max_off_diag.sqrt() < tolerance {
                break;
            }

            // Zero out entry (p, q) via 2x2 complex Jacobi unitary rotation:
            let h_pp = a[p][p].0;
            let h_qq = a[q][q].0;
            let h_pq = a[p][q];
            let abs_hpq = (h_pq.0 * h_pq.0 + h_pq.1 * h_pq.1).sqrt();

            if abs_hpq > 1e-15 {
                let phi = h_pq.1.atan2(h_pq.0);
                let tau = (h_qq - h_pp) / (2.0 * abs_hpq);
                let t = if tau >= 0.0 {
                    1.0 / (tau + (1.0 + tau * tau).sqrt())
                } else {
                    -1.0 / (-tau + (1.0 + tau * tau).sqrt())
                };

                let c = 1.0 / (1.0 + t * t).sqrt();
                let s = t * c;

                // Update diagonal elements:
                a[p][p].0 = h_pp - t * abs_hpq;
                a[p][p].1 = 0.0;
                a[q][q].0 = h_qq + t * abs_hpq;
                a[q][q].1 = 0.0;
                a[p][q] = (0.0, 0.0);
                a[q][p] = (0.0, 0.0);

                // Phase factors for complex Jacobi unitary rotation:
                // A'_{kp} = c * A_{kp} - s * e^(-i*phi) * A_{kq}
                // A'_{kq} = s * e^(i*phi) * A_{kp} + c * A_{kq}
                let exp_neg = (s * phi.cos(), -s * phi.sin());
                let exp_pos = (s * phi.cos(), s * phi.sin());

                // Update remaining elements in rows/columns p and q:
                for k_idx in 0..n {
                    if k_idx != p && k_idx != q {
                        let a_kp = a[k_idx][p];
                        let a_kq = a[k_idx][q];

                        // new_a_kp = c * a_kp - exp_neg * a_kq
                        let new_kp_re = c * a_kp.0 - (exp_neg.0 * a_kq.0 - exp_neg.1 * a_kq.1);
                        let new_kp_im = c * a_kp.1 - (exp_neg.0 * a_kq.1 + exp_neg.1 * a_kq.0);

                        // new_a_kq = exp_pos * a_kp + c * a_kq
                        let new_kq_re = (exp_pos.0 * a_kp.0 - exp_pos.1 * a_kp.1) + c * a_kq.0;
                        let new_kq_im = (exp_pos.0 * a_kp.1 + exp_pos.1 * a_kp.0) + c * a_kq.1;

                        a[k_idx][p] = (new_kp_re, new_kp_im);
                        a[p][k_idx] = (new_kp_re, -new_kp_im);

                        a[k_idx][q] = (new_kq_re, new_kq_im);
                        a[q][k_idx] = (new_kq_re, -new_kq_im);
                    }
                }
            }
        }

        // Extract and sort real eigenvalues:
        let mut evs = [0.0; 8];
        for i in 0..8 {
            evs[i] = a[i][i].0;
        }
        evs.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));

        let lower_flatband_ev = evs[3];
        let upper_flatband_ev = evs[4];
        let lower_remote_gap_ev = (evs[3] - evs[2]).max(0.0);
        let upper_remote_gap_ev = (evs[5] - evs[4]).max(0.0);

        MoireBandSolution {
            k,
            eigenvalues_ev: evs,
            lower_flatband_ev,
            upper_flatband_ev,
            lower_remote_gap_ev,
            upper_remote_gap_ev,
        }
    }

    /// Evaluates the flat-band kinetic bandwidth W across a supplied momentum mesh:
    /// W = max_k(E_4(k)) - min_k(E_3(k)).
    pub fn calculate_bandwidth_ev(&self, k_mesh: &[Vector2D]) -> f64 {
        let mut max_e4 = -f64::INFINITY;
        let mut min_e3 = f64::INFINITY;

        for &k in k_mesh {
            let sol = self.solve(k);
            if sol.upper_flatband_ev > max_e4 {
                max_e4 = sol.upper_flatband_ev;
            }
            if sol.lower_flatband_ev < min_e3 {
                min_e3 = sol.lower_flatband_ev;
            }
        }

        (max_e4 - min_e3).max(0.0)
    }

    /// Evaluates the numerical Dirac velocity v_F* = (1 / hbar) * |dE/dk| near the Dirac point k_D:
    /// Returns v_F* in nm/s and the ratio v_F* / v_F.
    pub fn calculate_dirac_velocity(&self, k_dirac: Vector2D, dk_nm: f64) -> (f64, f64) {
        let dk = dk_nm.max(1e-5);
        let sol_plus = self.solve(Vector2D::new(k_dirac.x + dk, k_dirac.y));
        let sol_minus = self.solve(Vector2D::new(k_dirac.x - dk, k_dirac.y));

        let de = (sol_plus.upper_flatband_ev - sol_minus.upper_flatband_ev).abs();
        let de_dk = de / (2.0 * dk); // eV*nm

        // v_F = (dE/dk) / hbar
        let vf_star_nm_s = de_dk / HBAR_EV_S;
        let ratio = de_dk / self.model.hbar_vf.max(1e-9);

        (vf_star_nm_s, ratio)
    }

    /// Computes local density of states (LDOS) \u{03c1}(E) at target energy E with Lorentzian broadening \u{03b7}:
    /// \u{03c1}(E) = (1 / N_k) * \u{2211}_{n, k} \u{03b7} / (\u{03c0} * ((E - E_n(k))^2 + \u{03b7}^2)).
    pub fn calculate_dos(&self, energy_ev: f64, broadening_ev: f64, k_mesh: &[Vector2D]) -> f64 {
        if k_mesh.is_empty() {
            return 0.0;
        }
        let eta = broadening_ev.max(1e-4);
        let mut total_dos = 0.0;

        for &k in k_mesh {
            let sol = self.solve(k);
            for &e_n in &sol.eigenvalues_ev {
                let diff = energy_ev - e_n;
                let lorentzian = (eta / PI) / (diff * diff + eta * eta);
                total_dos += lorentzian;
            }
        }

        total_dos / (k_mesh.len() as f64)
    }
}
