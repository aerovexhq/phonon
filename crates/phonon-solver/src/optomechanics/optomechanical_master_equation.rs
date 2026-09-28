//! Linearized quantum master equation and continuous Lyapunov covariance solver for optomechanical systems.
//!
//! Solves the steady-state continuous-time Lyapunov matrix equation:
//! $$\mathbf{A} \mathbf{V} + \mathbf{V} \mathbf{A}^T = -\mathbf{D}$$
//! for the $4 \times 4$ covariance matrix of optical and mechanical quadratures
//! $\mathbf{u} = [\hat{X}_a, \hat{Y}_a, \hat{X}_b, \hat{Y}_b]^T$:
//! - Evaluates minimum optical quadrature variance $V_{min}$ and ponderomotive squeezing $S_{dB}$.
//! - Evaluates steady-state mechanical phonon occupancy $\bar{n}_m = \frac{V_{33} + V_{44} - 1}{2}$.
//! - Evaluates photon-phonon quantum entanglement via logarithmic negativity $E_N = \max(0, -\ln(2\tilde{\nu}_-))$.

use phonon_models::optomechanics::PonderomotiveSqueezingParams;

/// Stationary quadrature covariance matrix result.
#[derive(Debug, Clone, PartialEq)]
pub struct CovarianceResult {
    /// 4x4 covariance matrix elements $V_{ij}$.
    pub covariance_matrix: [[f64; 4]; 4],
    /// Minimum optical quadrature variance $V_{min}$ (shot noise limit = 1.0).
    pub optical_min_variance: f64,
    /// Optical squeezing in decibels below shot noise: $-10 \log_{10}(V_{min})$.
    pub optical_squeezing_db: f64,
    /// Mechanical mode phonon occupancy $\bar{n}_m$.
    pub mechanical_phonon_occupancy: f64,
    /// Smallest symplectic eigenvalue $\tilde{\nu}_-$ of the partially transposed covariance matrix.
    pub smallest_symplectic_eigenvalue: f64,
    /// Logarithmic negativity entanglement $E_N$.
    pub logarithmic_negativity: f64,
}

/// Continuous Lyapunov equation solver for linearized optomechanics.
pub struct OptomechanicalMasterEquationSolver;

#[allow(clippy::needless_range_loop)]
impl OptomechanicalMasterEquationSolver {
    /// Solves the steady-state covariance matrix for given ponderomotive squeezing parameters.
    pub fn solve(params: &PonderomotiveSqueezingParams) -> CovarianceResult {
        let kappa_rad = 2.0 * std::f64::consts::PI * params.system.cavity_linewidth_hz;
        let gamma_m_rad = 2.0 * std::f64::consts::PI * params.system.mechanical_damping_hz;
        let omega_m_rad = 2.0 * std::f64::consts::PI * params.system.mechanical_frequency_hz;
        let g_rad = 2.0
            * std::f64::consts::PI
            * params
                .system
                .effective_coupling_hz(params.intracavity_photons);
        let n_th = params
            .system
            .thermal_phonon_occupancy(params.bath_temperature_k);

        // Drift matrix A (4x4):
        // [ -kappa/2,       0,       0,       0 ]
        // [       0, -kappa/2,     2*g,       0 ]
        // [       0,       0, -gamma/2, -omega_m ]
        // [     2*g,       0,  omega_m, -gamma/2 ]
        let mut a = [[0.0; 4]; 4];
        a[0][0] = -0.5 * kappa_rad;
        a[1][1] = -0.5 * kappa_rad;
        a[1][2] = 2.0 * g_rad;
        a[2][2] = -0.5 * gamma_m_rad;
        a[2][3] = -omega_m_rad;
        a[3][0] = 2.0 * g_rad;
        a[3][2] = omega_m_rad;
        a[3][3] = -0.5 * gamma_m_rad;

        // Diffusion matrix D (4x4):
        // diag( kappa/2, kappa/2, gamma/2 * (2*n_th + 1), gamma/2 * (2*n_th + 1) )
        let mut d = [[0.0; 4]; 4];
        d[0][0] = 0.5 * kappa_rad;
        d[1][1] = 0.5 * kappa_rad;
        let d_mech = 0.5 * gamma_m_rad * (2.0 * n_th + 1.0);
        d[2][2] = d_mech;
        d[3][3] = d_mech;

        // Solve A V + V A^T = -D via vectorization into 16x16 linear system
        let v = Self::solve_lyapunov_4x4(&a, &d);

        // Optical subsystem: 2x2 submatrix V[0..2][0..2]
        let v11 = v[0][0];
        let v22 = v[1][1];
        let v12 = v[0][1];

        let center = 0.5 * (v11 + v22);
        let radius = (0.25 * (v11 - v22).powi(2) + v12 * v12).sqrt();
        let v_min = (center - radius).max(0.01);
        let squeezing_db = if v_min < 1.0 {
            -10.0 * v_min.log10()
        } else {
            0.0
        };

        // Mechanical phonon occupancy
        let n_mech = (0.5 * (v[2][2] + v[3][3] - 1.0)).max(0.0);

        // Symplectic analysis for entanglement
        // Block form: V = [ A, C; C^T, B ]
        let det_a = v[0][0] * v[1][1] - v[0][1] * v[1][0];
        let det_b = v[2][2] * v[3][3] - v[2][3] * v[3][2];
        let det_c = v[0][2] * v[1][3] - v[0][3] * v[1][2];

        let det_v = Self::det_4x4(&v);
        let sigma = det_a + det_b - 2.0 * det_c;

        let discriminant = (sigma * sigma - 4.0 * det_v).max(0.0);
        let nu_minus_sq = (0.5 * (sigma - discriminant.sqrt())).max(1e-12);
        let nu_minus = nu_minus_sq.sqrt();

        let log_neg = if nu_minus < 0.5 {
            -(2.0 * nu_minus).ln().max(0.0)
        } else {
            0.0
        };

        CovarianceResult {
            covariance_matrix: v,
            optical_min_variance: v_min,
            optical_squeezing_db: squeezing_db,
            mechanical_phonon_occupancy: n_mech,
            smallest_symplectic_eigenvalue: nu_minus,
            logarithmic_negativity: log_neg,
        }
    }

    /// Solves 4x4 continuous Lyapunov equation A V + V A^T = -D using vectorization.
    fn solve_lyapunov_4x4(a: &[[f64; 4]; 4], d: &[[f64; 4]; 4]) -> [[f64; 4]; 4] {
        // Linear system M * vec(V) = b of dimension 16
        let mut m = [[0.0; 16]; 16];
        let mut b = [0.0; 16];

        for i in 0..4 {
            for j in 0..4 {
                let row = i * 4 + j;
                b[row] = -d[i][j];

                // (A V)_{ij} = sum_k A_{ik} V_{kj}
                for k in 0..4 {
                    let col = k * 4 + j;
                    m[row][col] += a[i][k];
                }

                // (V A^T)_{ij} = sum_k V_{ik} A_{jk}
                for k in 0..4 {
                    let col = i * 4 + k;
                    m[row][col] += a[j][k];
                }
            }
        }

        // Gaussian elimination with partial pivoting for 16x16 system
        let x = Self::solve_linear_system_16(&mut m, &mut b);

        let mut v = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                v[i][j] = x[i * 4 + j];
            }
        }
        v
    }

    fn solve_linear_system_16(m: &mut [[f64; 16]; 16], b: &mut [f64; 16]) -> [f64; 16] {
        let n = 16;
        for i in 0..n {
            // Find pivot
            let mut max_row = i;
            let mut max_val = m[i][i].abs();
            for k in (i + 1)..n {
                if m[k][i].abs() > max_val {
                    max_val = m[k][i].abs();
                    max_row = k;
                }
            }

            if max_row != i {
                m.swap(i, max_row);
                b.swap(i, max_row);
            }

            let pivot = m[i][i];
            if pivot.abs() < 1e-18 {
                continue;
            }

            for k in (i + 1)..n {
                let factor = m[k][i] / pivot;
                b[k] -= factor * b[i];
                for j in i..n {
                    m[k][j] -= factor * m[i][j];
                }
            }
        }

        let mut x = [0.0; 16];
        for i in (0..n).rev() {
            let mut sum = b[i];
            for j in (i + 1)..n {
                sum -= m[i][j] * x[j];
            }
            let denom = m[i][i];
            x[i] = if denom.abs() > 1e-18 {
                sum / denom
            } else {
                0.0
            };
        }
        x
    }

    fn det_4x4(m: &[[f64; 4]; 4]) -> f64 {
        // Laplace expansion on first row
        let mut det = 0.0;
        for j in 0..4 {
            let minor = Self::minor_3x3(m, 0, j);
            let sign = if j % 2 == 0 { 1.0 } else { -1.0 };
            det += sign * m[0][j] * minor;
        }
        det
    }

    fn minor_3x3(m: &[[f64; 4]; 4], skip_i: usize, skip_j: usize) -> f64 {
        let mut mat3 = [[0.0; 3]; 3];
        let mut r = 0;
        for i in 0..4 {
            if i == skip_i {
                continue;
            }
            let mut c = 0;
            for j in 0..4 {
                if j == skip_j {
                    continue;
                }
                mat3[r][c] = m[i][j];
                c += 1;
            }
            r += 1;
        }

        mat3[0][0] * (mat3[1][1] * mat3[2][2] - mat3[1][2] * mat3[2][1])
            - mat3[0][1] * (mat3[1][0] * mat3[2][2] - mat3[1][2] * mat3[2][0])
            + mat3[0][2] * (mat3[1][0] * mat3[2][1] - mat3[1][1] * mat3[2][0])
    }
}
