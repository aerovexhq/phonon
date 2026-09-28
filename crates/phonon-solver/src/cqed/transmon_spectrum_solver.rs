//! Exact numerical eigensolver for transmon charge-phase Hamiltonians in truncated charge basis.

use phonon_models::cqed::TransmonParams;

/// Eigensolution of the transmon Hamiltonian.
#[derive(Debug, Clone, PartialEq)]
pub struct TransmonSpectrumSolution {
    /// Ground state energy $E_0$ in GHz.
    pub e0_ghz: f64,
    /// First excited state energy $E_1$ in GHz.
    pub e1_ghz: f64,
    /// Second excited state energy $E_2$ in GHz.
    pub e2_ghz: f64,
    /// Third excited state energy $E_3$ in GHz.
    pub e3_ghz: f64,
    /// Numerical qubit transition frequency $\omega_{01} = E_1 - E_0$ in GHz.
    pub omega_01_ghz: f64,
    /// Numerical higher transition frequency $\omega_{12} = E_2 - E_1$ in GHz.
    pub omega_12_ghz: f64,
    /// Numerical negative anharmonicity $\alpha = \omega_{12} - \omega_{01}$ in GHz.
    pub anharmonicity_ghz: f64,
    /// Charge matrix element $|\langle 0 | \hat{n} | 1 \rangle|$.
    pub n01_matrix_element: f64,
}

/// Transmon charge-basis Hamiltonian eigensolver.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransmonSpectrumSolver {
    /// Charge basis truncation cutoff $N_{cut}$ (dimension $D = 2 N_{cut} + 1$).
    pub cutoff: usize,
}

impl Default for TransmonSpectrumSolver {
    fn default() -> Self {
        Self::new(10)
    }
}

impl TransmonSpectrumSolver {
    /// Creates a solver with specified charge cutoff $N_{cut}$.
    pub fn new(cutoff: usize) -> Self {
        assert!(cutoff >= 4, "Cutoff must be at least 4");
        Self { cutoff }
    }

    /// Solves the transmon Hamiltonian:
    /// $$\hat{H} = 4 E_C (\hat{n} - n_g)^2 - \frac{E_J}{2} \sum_n (|n\rangle\langle n+1| + |n+1\rangle\langle n|)$$
    pub fn solve(&self, params: &TransmonParams) -> TransmonSpectrumSolution {
        let n_cut = self.cutoff as i32;
        let dim = (2 * n_cut + 1) as usize;

        // Construct full symmetric Hamiltonian matrix
        let mut h = vec![0.0; dim * dim];
        for i in 0..dim {
            let n = (i as i32) - n_cut;
            let diag_val = 4.0 * params.ec_ghz * ((n as f64) - params.offset_charge_ng).powi(2);
            h[i * dim + i] = diag_val;
            if i < dim - 1 {
                h[i * dim + i + 1] = -0.5 * params.ej_ghz;
                h[(i + 1) * dim + i] = -0.5 * params.ej_ghz;
            }
        }

        let mut d = vec![0.0; dim];
        let mut v = vec![0.0; dim * dim];
        jacobi_eigen(&h, &mut d, &mut v, dim);

        // Sort eigenvalues in ascending order
        let mut indices: Vec<usize> = (0..dim).collect();
        indices.sort_by(|&a, &b| d[a].partial_cmp(&d[b]).unwrap());

        let e0 = d[indices[0]];
        let e1 = d[indices[1]];
        let e2 = d[indices[2]];
        let e3 = d[indices[3]];

        let omega_01 = e1 - e0;
        let omega_12 = e2 - e1;
        let alpha = omega_12 - omega_01;

        // Compute charge matrix element |<0| n |1>|:
        // n operator is diagonal in charge basis: n_{i,i} = i - n_cut
        let mut n01 = 0.0;
        for i in 0..dim {
            let n_val = ((i as i32) - n_cut) as f64;
            let v0 = v[i * dim + indices[0]];
            let v1 = v[i * dim + indices[1]];
            n01 += v0 * n_val * v1;
        }

        TransmonSpectrumSolution {
            e0_ghz: e0,
            e1_ghz: e1,
            e2_ghz: e2,
            e3_ghz: e3,
            omega_01_ghz: omega_01,
            omega_12_ghz: omega_12,
            anharmonicity_ghz: alpha,
            n01_matrix_element: n01.abs(),
        }
    }
}

/// Computes eigenvalues and eigenvectors of a real symmetric matrix via Jacobi rotations.
fn jacobi_eigen(a_in: &[f64], d: &mut [f64], v: &mut [f64], n: usize) {
    let mut a = a_in.to_vec();
    for i in 0..n {
        for j in 0..n {
            v[i * n + j] = if i == j { 1.0 } else { 0.0 };
        }
        d[i] = a[i * n + i];
    }

    for _sweep in 0..50 {
        let mut sm = 0.0;
        for p in 0..n {
            for q in (p + 1)..n {
                sm += a[p * n + q].abs();
            }
        }
        if sm <= 1e-12 {
            break;
        }

        for p in 0..n {
            for q in (p + 1)..n {
                let apq = a[p * n + q];
                if apq.abs() > 1e-14 {
                    let h = d[q] - d[p];
                    let theta = 0.5 * h / apq;
                    let mut t = 1.0 / (theta.abs() + (1.0 + theta * theta).sqrt());
                    if theta < 0.0 {
                        t = -t;
                    }
                    let c = 1.0 / (1.0 + t * t).sqrt();
                    let s = t * c;
                    let tau = s / (1.0 + c);
                    let h_rot = t * apq;
                    d[p] -= h_rot;
                    d[q] += h_rot;
                    a[p * n + q] = 0.0;

                    for j in 0..p {
                        let g1 = a[j * n + p];
                        let h1 = a[j * n + q];
                        a[j * n + p] = g1 - s * (h1 + g1 * tau);
                        a[j * n + q] = h1 + s * (g1 - h1 * tau);
                    }
                    for j in (p + 1)..q {
                        let g1 = a[p * n + j];
                        let h1 = a[j * n + q];
                        a[p * n + j] = g1 - s * (h1 + g1 * tau);
                        a[j * n + q] = h1 + s * (g1 - h1 * tau);
                    }
                    for j in (q + 1)..n {
                        let g1 = a[p * n + j];
                        let h1 = a[q * n + j];
                        a[p * n + j] = g1 - s * (h1 + g1 * tau);
                        a[q * n + j] = h1 + s * (g1 - h1 * tau);
                    }
                    for j in 0..n {
                        let g1 = v[j * n + p];
                        let h1 = v[j * n + q];
                        v[j * n + p] = g1 - s * (h1 + g1 * tau);
                        v[j * n + q] = h1 + s * (g1 - h1 * tau);
                    }
                }
            }
        }
    }
}
