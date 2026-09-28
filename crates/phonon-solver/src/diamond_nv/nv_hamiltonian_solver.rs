//! Diamond NV Spin-1 Hamiltonian Eigensolver & 3D Vector Magnetometry.
//!
//! Diagonalizes the general 3x3 Spin-1 Hamiltonian in safe Rust using
//! Jacobi rotations, extracting exact transition frequencies, and reconstructs
//! full 3D vector magnetic fields from 4 crystallographic NV orientations.

use phonon_models::diamond_nv::NvCenterConfig;

/// Result of diagonalizing the NV Spin-1 Hamiltonian.
#[derive(Debug, Clone, PartialEq)]
pub struct NvEigenResult {
    /// Sorted eigenenergies in Hertz: $E_0 \le E_1 \le E_2$.
    pub eigenenergies_hz: [f64; 3],
    /// Lower transition frequency $f_- = E_1 - E_0$ in Hertz.
    pub lower_transition_hz: f64,
    /// Upper transition frequency $f_+ = E_2 - E_0$ in Hertz.
    pub upper_transition_hz: f64,
    /// Zeeman splitting $\Delta f = f_+ - f_-$ in Hertz.
    pub zeeman_splitting_hz: f64,
}

/// Solver for NV Spin-1 Hamiltonians and vector field reconstruction.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NvHamiltonianSolver {
    /// NV center physical configuration.
    pub config: NvCenterConfig,
}

impl NvHamiltonianSolver {
    /// Creates a new NV Hamiltonian solver.
    pub fn new(config: NvCenterConfig) -> Self {
        Self { config }
    }

    /// Solves the 3x3 Spin-1 Hamiltonian for a given local magnetic field $\mathbf{B}_{loc} = (B_x, B_y, B_z)$
    /// defined in the NV frame (where $\hat{z}$ is aligned with the NV C3v symmetry axis).
    pub fn solve_hamiltonian_nv_frame(&self, b_nv: [f64; 3]) -> NvEigenResult {
        let d = self.config.zero_field_splitting_hz;
        let e = self.config.transverse_strain_hz;
        let gamma = self.config.electron_gyromagnetic_ratio_hz_per_t;

        let bx = b_nv[0];
        let by = b_nv[1];
        let bz = b_nv[2];

        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();

        // 3x3 complex Hermitian Hamiltonian matrix H = H_real + i H_imag
        let mut hr = [
            [d + gamma * bz, gamma * bx * inv_sqrt2, e],
            [gamma * bx * inv_sqrt2, 0.0, gamma * bx * inv_sqrt2],
            [e, gamma * bx * inv_sqrt2, d - gamma * bz],
        ];

        let mut hi = [
            [0.0, -gamma * by * inv_sqrt2, 0.0],
            [gamma * by * inv_sqrt2, 0.0, -gamma * by * inv_sqrt2],
            [0.0, gamma * by * inv_sqrt2, 0.0],
        ];

        // Complex Jacobi diagonalization for 3x3 Hermitian matrix
        for _ in 0..50 {
            let mut max_off = 0.0;
            let mut p = 0;
            let mut q = 1;

            for i in 0..3 {
                for j in (i + 1)..3 {
                    let off = hr[i][j].hypot(hi[i][j]);
                    if off > max_off {
                        max_off = off;
                        p = i;
                        q = j;
                    }
                }
            }

            if max_off < 1e-6 {
                break;
            }

            let app = hr[p][p];
            let aqq = hr[q][q];
            let apq_r = hr[p][q];
            let apq_i = hi[p][q];
            let abs_apq = apq_r.hypot(apq_i);

            if abs_apq > 1e-12 {
                let phi = apq_i.atan2(apq_r);
                let theta = 0.5 * (2.0 * abs_apq).atan2(app - aqq);

                let c = theta.cos();
                let s = theta.sin();
                let e_i_phi_r = phi.cos();
                let e_i_phi_i = phi.sin();

                // Unitary similarity transformation H' = U^dagger H U
                // with U = [[c, -s e^{i phi}], [s e^{-i phi}, c]] on subspace (p, q)
                let mut new_hr = hr;
                let mut new_hi = hi;

                for k in 0..3 {
                    if k != p && k != q {
                        // H[p][k] and H[q][k]
                        let akp_r = hr[k][p];
                        let akp_i = hi[k][p];
                        let akq_r = hr[k][q];
                        let akq_i = hi[k][q];

                        // c * akp + s * e^{i phi} * akq
                        let term2_r = s * (akq_r * e_i_phi_r - akq_i * e_i_phi_i);
                        let term2_i = s * (akq_r * e_i_phi_i + akq_i * e_i_phi_r);
                        new_hr[k][p] = c * akp_r + term2_r;
                        new_hi[k][p] = c * akp_i + term2_i;
                        new_hr[p][k] = new_hr[k][p];
                        new_hi[p][k] = -new_hi[k][p];

                        // -s * e^{-i phi} * akp + c * akq
                        let term1_r = -s * (akp_r * e_i_phi_r + akp_i * e_i_phi_i);
                        let term1_i = -s * (-akp_r * e_i_phi_i + akp_i * e_i_phi_r);
                        new_hr[k][q] = term1_r + c * akq_r;
                        new_hi[k][q] = term1_i + c * akq_i;
                        new_hr[q][k] = new_hr[k][q];
                        new_hi[q][k] = -new_hi[k][q];
                    }
                }

                new_hr[p][p] = c * c * app + s * s * aqq + 2.0 * s * c * abs_apq;
                new_hr[q][q] = s * s * app + c * c * aqq - 2.0 * s * c * abs_apq;
                new_hr[p][q] = 0.0;
                new_hi[p][q] = 0.0;
                new_hr[q][p] = 0.0;
                new_hi[q][p] = 0.0;

                hr = new_hr;
                hi = new_hi;
            }
        }

        let mut energies = [hr[0][0], hr[1][1], hr[2][2]];
        energies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let f_minus = energies[1] - energies[0];
        let f_plus = energies[2] - energies[0];

        NvEigenResult {
            eigenenergies_hz: energies,
            lower_transition_hz: f_minus,
            upper_transition_hz: f_plus,
            zeeman_splitting_hz: f_plus - f_minus,
        }
    }

    /// Solves the NV center transition frequencies for orientation $i \in \{0, 1, 2, 3\}$
    /// under laboratory frame magnetic field $\mathbf{B}_{lab}$.
    pub fn solve_orientation(&self, b_lab: [f64; 3], orientation_idx: usize) -> NvEigenResult {
        let b_parallel = self.config.projected_field_tesla(b_lab, orientation_idx);
        // Project onto NV frame with B_z = B_parallel
        self.solve_hamiltonian_nv_frame([0.0, 0.0, b_parallel])
    }

    /// Reconstructs the 3D vector magnetic field $\mathbf{B} = (B_x, B_y, B_z)$
    /// from measured ODMR resonance pairs $(f_-, f_+)$ across the 4 NV orientations.
    /// An optional reference direction can be supplied to resolve the global $\pm \mathbf{B}$ inversion degeneracy.
    pub fn reconstruct_vector_field_from_odmr(
        &self,
        resonance_pairs: &[(f64, f64); 4],
        reference_direction: Option<[f64; 3]>,
    ) -> [f64; 3] {
        let gamma = self.config.electron_gyromagnetic_ratio_hz_per_t;
        let e = self.config.transverse_strain_hz;

        let mut magnitudes = [0.0; 4];
        for i in 0..4 {
            let (f_minus, f_plus) = resonance_pairs[i];
            let half_split = 0.5 * (f_plus - f_minus).abs();
            let b_mag = if half_split > e {
                (half_split.powi(2) - e.powi(2)).sqrt() / gamma
            } else {
                0.0
            };
            magnitudes[i] = b_mag;
        }

        // Exhaustive tetrahedral zero-sum search across 16 sign combinations
        let mut best_sum = f64::INFINITY;
        let mut best_s = [1.0; 4];

        for s0 in [1.0, -1.0] {
            for s1 in [1.0, -1.0] {
                for s2 in [1.0, -1.0] {
                    for s3 in [1.0, -1.0] {
                        let sum_val = (s0 * magnitudes[0]
                            + s1 * magnitudes[1]
                            + s2 * magnitudes[2]
                            + s3 * magnitudes[3])
                            .abs();
                        if sum_val < best_sum {
                            best_sum = sum_val;
                            best_s = [s0, s1, s2, s3];
                        }
                    }
                }
            }
        }

        let proj = [
            best_s[0] * magnitudes[0],
            best_s[1] * magnitudes[1],
            best_s[2] * magnitudes[2],
            best_s[3] * magnitudes[3],
        ];

        let mut b_rec = self.config.reconstruct_vector_field(proj);

        // Resolve global inversion sign against reference direction (defaults to [0, 0, 1])
        let ref_dir = reference_direction.unwrap_or([0.0, 0.0, 1.0]);
        let dot = b_rec[0] * ref_dir[0] + b_rec[1] * ref_dir[1] + b_rec[2] * ref_dir[2];
        if dot < 0.0 {
            b_rec = [-b_rec[0], -b_rec[1], -b_rec[2]];
        }

        b_rec
    }
}
