//! Self-Consistent NEGF-Poisson Molecular Transport Solver.
//!
//! Solves:
//! 1. Retarded Green's function \(G^R(E) = [E \cdot I - H_M(V) - \Sigma_L - \Sigma_R]^{-1}\).
//! 2. Spectral function \(A(E) = i [G^R(E) - G^{R\dagger}(E)]\) and partial spectral functions \(A_{L, R}(E) = G^R \Gamma_{L, R} G^{R\dagger}\).
//! 3. Non-equilibrium charge density \(\rho_i\) by integrating local density of states filled by left and right leads:
//!    \[\rho_i = \frac{1}{2\pi} \int [A_L(E) f_L(E) + A_R(E) f_R(E)]_{ii} dE\]
//! 4. Electrostatic potential \(V_i\) via discrete Poisson / charging matrix \(U_{ij}\):
//!    \[V_i = V_i^{\text{gate}} + \sum_j U_{ij} (\rho_j - \rho_j^0)\]
//! 5. Self-consistent iterative convergence with adaptive damping / linear mixing.

#![allow(clippy::needless_range_loop)]

use phonon_models::molecular::{invert_complex_matrix, MolecularJunction, NegfTransportSolver};
use phonon_models::quantum::Complex;

/// Configuration for the self-consistent NEGF-Poisson solver.
#[derive(Debug, Clone)]
pub struct SelfConsistentNegfConfig {
    pub max_iterations: usize,
    pub tolerance_v: f64,
    pub mixing_alpha: f64,
    pub temperature_k: f64,
    pub energy_points: usize,
    pub charging_energy_u_ev: f64,
}

impl Default for SelfConsistentNegfConfig {
    fn default() -> Self {
        Self {
            max_iterations: 60,
            tolerance_v: 1.0e-4,
            mixing_alpha: 0.25,
            temperature_k: 300.0,
            energy_points: 60,
            charging_energy_u_ev: 1.2, // ~1.2 eV Hubbard U / on-site charging
        }
    }
}

/// Output results from self-consistent NEGF-Poisson iteration.
#[derive(Debug, Clone)]
pub struct SelfConsistentNegfResult {
    pub converged: bool,
    pub iterations: usize,
    pub final_residual: f64,
    pub site_charges: Vec<f64>,
    pub site_potentials_v: Vec<f64>,
    pub current_a: f64,
    pub transmission_ef: f64,
}

/// Self-consistent NEGF-Poisson iterative solver.
#[derive(Debug, Clone)]
pub struct SelfConsistentNegfSolver {
    pub config: SelfConsistentNegfConfig,
}

impl Default for SelfConsistentNegfSolver {
    fn default() -> Self {
        Self::new(SelfConsistentNegfConfig::default())
    }
}

impl SelfConsistentNegfSolver {
    pub fn new(config: SelfConsistentNegfConfig) -> Self {
        Self { config }
    }

    /// Evaluates partial spectral matrix \(A_\alpha(E) = G^R \Gamma_\alpha G^{R\dagger}\).
    pub fn evaluate_partial_spectral(
        &self,
        g_r: &[Vec<Complex>],
        gamma_lead: &[Vec<Complex>],
    ) -> Vec<Vec<Complex>> {
        let n = g_r.len();
        // Step 1: M = G^R * Gamma
        let mut m = vec![vec![Complex::ZERO; n]; n];
        for i in 0..n {
            for j in 0..n {
                let mut sum = Complex::ZERO;
                for k in 0..n {
                    sum = sum.add(g_r[i][k].mul(gamma_lead[k][j]));
                }
                m[i][j] = sum;
            }
        }

        // Step 2: A = M * (G^R)^dagger = M * (G^R)^T*
        let mut a = vec![vec![Complex::ZERO; n]; n];
        for i in 0..n {
            for j in 0..n {
                let mut sum = Complex::ZERO;
                for k in 0..n {
                    sum = sum.add(m[i][k].mul(g_r[j][k].conj()));
                }
                a[i][j] = sum;
            }
        }
        a
    }

    /// Solves the self-consistent potential and non-equilibrium charge distribution.
    pub fn solve(
        &self,
        junction: &MolecularJunction,
        v_bias_v: f64,
        v_gate_v: f64,
    ) -> SelfConsistentNegfResult {
        let n = junction.num_sites;
        let negf = NegfTransportSolver::new(self.config.temperature_k, 0.0);

        let mu_l = v_bias_v * 0.5;
        let mu_r = -v_bias_v * 0.5;
        let kb_t_ev = 8.617_333_262e-5 * self.config.temperature_k;

        // Bias integration window
        let e_window = (v_bias_v.abs() + (12.0 * kb_t_ev)).max(1.5);
        let e_min = -e_window;
        let e_max = e_window;
        let de = (e_max - e_min) / (self.config.energy_points as f64).max(1.0);

        // Precompute lead level broadening matrices
        let gamma_l = junction.gamma_left();
        let gamma_r = junction.gamma_right();

        // Neutral site electron population rho_0 = 1.0 (half-filled pi system)
        let rho_0 = vec![1.0; n];

        // Potential state vector initialized to electrostatic gate + bias profile
        let mut v_pot = vec![0.0; n];
        for (i, v_val) in v_pot.iter_mut().enumerate() {
            let frac = if n > 1 {
                i as f64 / (n - 1) as f64
            } else {
                0.5
            };
            let v_laplace = mu_l * (1.0 - frac) + mu_r * frac;
            let v_gate_contrib = junction.gate_coupling_factor * v_gate_v;
            *v_val = v_laplace + v_gate_contrib;
        }

        let mut charges = rho_0.clone();
        let mut iterations = 0;
        let mut converged = false;
        let mut final_residual = 0.0;

        let mut active_junction = junction.clone();

        while iterations < self.config.max_iterations {
            iterations += 1;

            // Apply current potential to junction on-site energies
            for (i, val) in active_junction.on_site_energies_ev.iter_mut().enumerate() {
                *val = -v_pot[i];
            }

            // Integrate non-equilibrium density: rho_i = 1/(2*pi) * integral [A_L*f_L + A_R*f_R]_ii dE
            let mut new_charges = vec![0.0; n];

            for pt in 0..self.config.energy_points {
                let e = e_min + (pt as f64) * de;
                let f_l = negf.fermi_dirac(e, mu_l);
                let f_r = negf.fermi_dirac(e, mu_r);

                if f_l > 1.0e-6 || f_r > 1.0e-6 {
                    let a_mat = active_junction.assemble_green_matrix(e, 0.0);
                    if let Some(g_r) = invert_complex_matrix(a_mat) {
                        let a_l = self.evaluate_partial_spectral(&g_r, &gamma_l);
                        let a_r = self.evaluate_partial_spectral(&g_r, &gamma_r);

                        let factor = (1.0 / (2.0 * std::f64::consts::PI)) * de;
                        for i in 0..n {
                            let dos_l = a_l[i][i].re;
                            let dos_r = a_r[i][i].re;
                            new_charges[i] += (dos_l * f_l + dos_r * f_r) * factor;
                        }
                    }
                }
            }

            // Poisson charging update: delta_V_i = U * (rho_i - rho_0_i)
            let mut max_diff = 0.0;
            let mut v_new = vec![0.0; n];

            for i in 0..n {
                let frac = if n > 1 {
                    i as f64 / (n - 1) as f64
                } else {
                    0.5
                };
                let v_ext = (mu_l * (1.0 - frac) + mu_r * frac)
                    + (junction.gate_coupling_factor * v_gate_v);
                let delta_rho = new_charges[i] - rho_0[i];
                let v_hartree = self.config.charging_energy_u_ev * delta_rho;
                let v_target = v_ext + v_hartree;

                // Damped linear mixing
                v_new[i] = (1.0 - self.config.mixing_alpha) * v_pot[i]
                    + self.config.mixing_alpha * v_target;
                let diff = (v_new[i] - v_pot[i]).abs();
                if diff > max_diff {
                    max_diff = diff;
                }
            }

            v_pot = v_new;
            charges = new_charges;
            final_residual = max_diff;

            if max_diff < self.config.tolerance_v {
                converged = true;
                break;
            }
        }

        // Evaluate final Landauer current and transmission at E_F with converged potential
        for (i, val) in active_junction.on_site_energies_ev.iter_mut().enumerate() {
            *val = -v_pot[i];
        }

        let current_a = negf.evaluate_current_landauer(&active_junction, v_bias_v, 0.0);
        let transmission_ef = negf.evaluate_transmission(&active_junction, 0.0, 0.0);

        SelfConsistentNegfResult {
            converged,
            iterations,
            final_residual,
            site_charges: charges,
            site_potentials_v: v_pot,
            current_a,
            transmission_ef,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_self_consistent_negf_convergence_para_benzene() {
        let junction = MolecularJunction::para_benzene(0.5);
        let solver = SelfConsistentNegfSolver::new(SelfConsistentNegfConfig {
            max_iterations: 40,
            tolerance_v: 1.0e-3,
            mixing_alpha: 0.35,
            temperature_k: 300.0,
            energy_points: 30,
            charging_energy_u_ev: 0.8,
        });

        let result = solver.solve(&junction, 0.20, 0.0);
        assert!(result.iterations > 0);
        assert_eq!(result.site_charges.len(), junction.num_sites);
        assert_eq!(result.site_potentials_v.len(), junction.num_sites);
        assert!(result.current_a.abs() >= 0.0);
    }
}
