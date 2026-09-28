//! Numerical eigensolver for the Giant Spin Hamiltonian:
//! exact Jacobi matrix diagonalization, energy spectra, avoided level crossings,
//! and resonant quantum tunneling of magnetization (QTM) splittings.

use phonon_models::stno::{GiantSpinParams, BOHR_MAGNETON, BOLTZMANN_K};

/// Numerical solution of the Giant Spin Hamiltonian.
#[derive(Debug, Clone, PartialEq)]
pub struct GiantSpinSolution {
    /// Dimension of the Hilbert space (2S + 1).
    pub dim: usize,
    /// Sorted eigenenergies in Kelvin (E_0 <= E_1 <= ... <= E_{2S}).
    pub eigenenergies_k: Vec<f64>,
    /// Ground-state tunnel splitting Delta_0 = E_1 - E_0 in Kelvin.
    pub ground_state_splitting_k: f64,
    /// Tunnel splitting in Joules.
    pub ground_state_splitting_joules: f64,
    /// Tunneling frequency nu_QTM = Delta_0 / h in Hz.
    pub tunneling_frequency_hz: f64,
}

/// Eigensolver for single-molecule magnet giant spin Hamiltonians.
pub struct GiantSpinSolver;

impl GiantSpinSolver {
    /// Solves the full Giant Spin Hamiltonian in the presence of longitudinal magnetic field B_z (Tesla)
    /// and transverse field B_x (Tesla).
    pub fn solve(params: &GiantSpinParams, b_z_tesla: f64, b_x_tesla: f64) -> GiantSpinSolution {
        let dim = params.hilbert_dim();
        let s = params.spin_s;
        let mut h = vec![0.0; dim * dim];

        let d_k = params.d_uniaxial_k;
        let e_k = params.e_rhombic_k;
        let b44_k = params.b44_crystal_field_k;

        let g = params.g_factor;
        let bz_zeeman_k = (g * BOHR_MAGNETON * b_z_tesla) / BOLTZMANN_K;
        let bx_zeeman_k = (g * BOHR_MAGNETON * b_x_tesla) / BOLTZMANN_K;

        // Basis index i in 0..dim corresponds to spin projection m = -S + i
        for i in 0..dim {
            let m = -s + i as f64;

            // 1. Diagonal elements: -D * S_z^2 - g * mu_B * B_z * S_z
            let h_diag = -d_k * m.powi(2) - bz_zeeman_k * m;
            h[i * dim + i] = h_diag;

            // 2. Off-diagonal Delta m = +-1: -g * mu_B * B_x * S_x = - (1/2) g mu_B B_x (S_+ + S_-)
            if i + 1 < dim {
                let m_next = m;
                let s_plus_mat = (s * (s + 1.0) - m_next * (m_next + 1.0)).max(0.0).sqrt();
                let h_trans = -0.5 * bx_zeeman_k * s_plus_mat;
                h[i * dim + (i + 1)] = h_trans;
                h[(i + 1) * dim + i] = h_trans;
            }

            // 3. Off-diagonal Delta m = +-2: E * (S_x^2 - S_y^2) = (E / 2) * (S_+^2 + S_-^2)
            if i + 2 < dim {
                let factor1 = (s * (s + 1.0) - m * (m + 1.0)).max(0.0).sqrt();
                let factor2 = (s * (s + 1.0) - (m + 1.0) * (m + 2.0)).max(0.0).sqrt();
                let h_rhombic = 0.5 * e_k * factor1 * factor2;
                h[i * dim + (i + 2)] = h_rhombic;
                h[(i + 2) * dim + i] = h_rhombic;
            }

            // 4. Off-diagonal Delta m = +-4: B_4^4 * (S_+^4 + S_-^4)
            if i + 4 < dim {
                let f1 = (s * (s + 1.0) - m * (m + 1.0)).max(0.0).sqrt();
                let f2 = (s * (s + 1.0) - (m + 1.0) * (m + 2.0)).max(0.0).sqrt();
                let f3 = (s * (s + 1.0) - (m + 2.0) * (m + 3.0)).max(0.0).sqrt();
                let f4 = (s * (s + 1.0) - (m + 3.0) * (m + 4.0)).max(0.0).sqrt();
                let h_b44 = b44_k * f1 * f2 * f3 * f4;
                h[i * dim + (i + 4)] = h_b44;
                h[(i + 4) * dim + i] = h_b44;
            }
        }

        // Diagonalize using classic Jacobi algorithm
        let mut eigenvalues = jacobi_diagonalize(&mut h, dim);
        eigenvalues.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let ground_k = (eigenvalues[1] - eigenvalues[0]).abs();
        let ground_j = ground_k * BOLTZMANN_K;
        let h_planck = 6.626_070_15e-34;
        let nu_hz = ground_j / h_planck;

        GiantSpinSolution {
            dim,
            eigenenergies_k: eigenvalues,
            ground_state_splitting_k: ground_k,
            ground_state_splitting_joules: ground_j,
            tunneling_frequency_hz: nu_hz,
        }
    }
}

fn jacobi_diagonalize(matrix: &mut [f64], n: usize) -> Vec<f64> {
    let max_iter = 100;
    let tol = 1e-13;

    for _ in 0..max_iter {
        let mut max_off = 0.0;
        let mut p = 0;
        let mut q = 1;

        for i in 0..n {
            for j in (i + 1)..n {
                let val = matrix[i * n + j].abs();
                if val > max_off {
                    max_off = val;
                    p = i;
                    q = j;
                }
            }
        }

        if max_off < tol {
            break;
        }

        let app = matrix[p * n + p];
        let aqq = matrix[q * n + q];
        let apq = matrix[p * n + q];

        let phi = 0.5 * (2.0 * apq).atan2(aqq - app);
        let c = phi.cos();
        let s = phi.sin();

        // Rotate matrix
        for k in 0..n {
            if k != p && k != q {
                let akp = matrix[k * n + p];
                let akq = matrix[k * n + q];
                matrix[k * n + p] = c * akp - s * akq;
                matrix[p * n + k] = matrix[k * n + p];
                matrix[k * n + q] = s * akp + c * akq;
                matrix[q * n + k] = matrix[k * n + q];
            }
        }

        let new_app = c * c * app - 2.0 * s * c * apq + s * s * aqq;
        let new_aqq = s * s * app + 2.0 * s * c * apq + c * c * aqq;

        matrix[p * n + p] = new_app;
        matrix[q * n + q] = new_aqq;
        matrix[p * n + q] = 0.0;
        matrix[q * n + p] = 0.0;
    }

    (0..n).map(|i| matrix[i * n + i]).collect()
}
