//! Tight-binding Bogoliubov-de Gennes (BdG) Hamiltonian and Majorana Nanowire physics.
//!
//! Formulates the 1D semiconductor nanowire with Rashba spin-orbit coupling,
//! proximity-induced s-wave superconductivity, and axial Zeeman splitting.
//! Models topological phase transitions, Majorana Zero Modes (MZMs),
//! and boundary wavefunctions.

use phonon_core::{ELEMENTARY_CHARGE, H_BAR};

/// Parameters for a semiconductor-superconductor heterostructure nanowire.
#[derive(Debug, Clone, PartialEq)]
pub struct NanowireParams {
    /// Number of spatial lattice sites $N$.
    pub num_sites: usize,
    /// Lattice spacing $a$ in meters ($m$). Typically $\approx 10\text{ nm}$.
    pub lattice_spacing_m: f64,
    /// Effective carrier mass $m^*$ in units of free electron mass $m_0$.
    /// For InAs, $m^* \approx 0.023$; for InSb, $m^* \approx 0.014$.
    pub effective_mass: f64,
    /// Nearest-neighbor hopping energy $t = \frac{\hbar^2}{2 m^* a^2}$ in meV.
    pub hopping_mev: f64,
    /// Chemical potential $\mu$ in meV (tunable via back-gate voltage).
    pub chemical_potential_mev: f64,
    /// Axial Zeeman energy splitting $V_Z = \frac{1}{2} g \mu_B B_x$ in meV.
    pub zeeman_mev: f64,
    /// Rashba spin-orbit coupling parameter $\alpha_{SO}$ in $\text{meV}\cdot\text{nm}$.
    /// $\alpha_R = \alpha_{SO} / (2a)$ in meV.
    pub rashba_mev_nm: f64,
    /// Proximity-induced s-wave superconducting pairing potential $\Delta$ in meV.
    pub pairing_delta_mev: f64,
}

impl Default for NanowireParams {
    fn default() -> Self {
        let a_m = 10e-9; // 10 nm lattice spacing
        let m_eff = 0.023; // InAs
        let m0 = 9.1093837e-31;
        let hbar = H_BAR;
        let t_joules = (hbar * hbar) / (2.0 * m_eff * m0 * a_m * a_m);
        let t_mev = (t_joules / ELEMENTARY_CHARGE) * 1e3; // ~16.5 meV

        Self {
            num_sites: 30,
            lattice_spacing_m: a_m,
            effective_mass: m_eff,
            hopping_mev: t_mev,
            chemical_potential_mev: 0.0,
            zeeman_mev: 1.2,        // > sqrt(0.5^2 + 0.0^2) = 0.5 meV (topological)
            rashba_mev_nm: 25.0,    // 25 meV*nm
            pairing_delta_mev: 0.5, // 0.5 meV (e.g. Al/InAs proximity)
        }
    }
}

impl NanowireParams {
    /// Wire length $L = N \cdot a$ in meters.
    #[inline]
    pub fn length_m(&self) -> f64 {
        self.num_sites as f64 * self.lattice_spacing_m
    }

    /// Hopping Rashba parameter $\alpha_R = \alpha_{SO} / (2a)$ in meV.
    #[inline]
    pub fn rashba_hopping_mev(&self) -> f64 {
        let a_nm = self.lattice_spacing_m * 1e9;
        self.rashba_mev_nm / (2.0 * a_nm)
    }

    /// Critical Zeeman energy for topological transition: $V_{Z,c} = \sqrt{\Delta^2 + \mu^2}$.
    #[inline]
    pub fn critical_zeeman_mev(&self) -> f64 {
        (self.pairing_delta_mev * self.pairing_delta_mev
            + self.chemical_potential_mev * self.chemical_potential_mev)
            .sqrt()
    }

    /// Checks if parameters are in the topological superconducting regime ($V_Z > V_{Z,c}$).
    #[inline]
    pub fn is_topological(&self) -> bool {
        self.zeeman_mev > self.critical_zeeman_mev()
    }

    /// Estimates the topological minigap $\Delta_{top} \approx \min(\Delta, V_Z - V_{Z,c})$.
    #[inline]
    pub fn estimated_minigap_mev(&self) -> f64 {
        if !self.is_topological() {
            0.0
        } else {
            let excess = self.zeeman_mev - self.critical_zeeman_mev();
            excess.min(self.pairing_delta_mev)
        }
    }

    /// Majorana coherence length $\xi_M = \frac{\hbar v_F}{\Delta_{top}}$ in meters.
    pub fn majorana_coherence_length_m(&self) -> f64 {
        let gap_mev = self.estimated_minigap_mev().max(1e-4);
        let gap_joules = gap_mev * 1e-3 * ELEMENTARY_CHARGE;
        let m0 = 9.1093837e-31;
        let m_star = self.effective_mass * m0;
        // Fermi velocity estimate v_F = sqrt(2 * t_joules / m_star)
        let t_joules = self.hopping_mev * 1e-3 * ELEMENTARY_CHARGE;
        let v_f = (2.0 * t_joules / m_star).sqrt();
        (H_BAR * v_f) / gap_joules
    }
}

/// Eigensolution of the BdG Hamiltonian.
#[derive(Debug, Clone)]
pub struct BdGSolution {
    /// Eigenvalues in meV sorted in ascending order.
    pub eigenvalues: Vec<f64>,
    /// Orthonormal eigenvectors stored as columns of a matrix $V$:
    /// `eigenvectors[site_spin_index][eigen_index]`.
    pub eigenvectors: Vec<Vec<f64>>,
}

/// Discretized tight-binding Bogoliubov-de Gennes Hamiltonian builder and solver.
#[derive(Debug, Clone)]
pub struct BdGHamiltonian {
    pub params: NanowireParams,
    pub dimension: usize,
    pub matrix: Vec<Vec<f64>>,
}

impl BdGHamiltonian {
    /// Constructs the $4N \times 4N$ real symmetric BdG Hamiltonian matrix.
    pub fn new(params: NanowireParams) -> Self {
        let n = params.num_sites;
        let dim = 4 * n;
        let mut matrix = vec![vec![0.0; dim]; dim];

        let t = params.hopping_mev;
        let mu = params.chemical_potential_mev;
        let vz = params.zeeman_mev;
        let alpha_r = params.rashba_hopping_mev();
        let delta = params.pairing_delta_mev;

        // Stamp on-site 4x4 blocks
        for i in 0..n {
            let base = 4 * i;
            let on_site_diag = 2.0 * t - mu;

            // Electron block
            matrix[base][base] = on_site_diag;
            matrix[base + 1][base + 1] = on_site_diag;
            // Zeeman along x: V_Z sigma_x
            matrix[base][base + 1] = vz;
            matrix[base + 1][base] = vz;

            // Pairing block: delta * i sigma_y
            // [ 0,  delta ]
            // [-delta, 0  ]
            matrix[base][base + 3] = delta;
            matrix[base + 3][base] = delta;
            matrix[base + 1][base + 2] = -delta;
            matrix[base + 2][base + 1] = -delta;

            // Hole block: -(2t - mu) - V_Z sigma_x
            matrix[base + 2][base + 2] = -on_site_diag;
            matrix[base + 3][base + 3] = -on_site_diag;
            matrix[base + 2][base + 3] = -vz;
            matrix[base + 3][base + 2] = -vz;
        }

        // Stamp nearest-neighbor hopping 4x4 blocks between site i and i+1
        for i in 0..(n - 1) {
            let bi = 4 * i;
            let bj = 4 * (i + 1);

            // Electron hopping: -t I - i alpha_R sigma_y
            // = [ -t,  -alpha_R ]
            //   [ alpha_R, -t   ]
            // H_{i, i+1}
            matrix[bi][bj] = -t;
            matrix[bi][bj + 1] = -alpha_r;
            matrix[bi + 1][bj] = alpha_r;
            matrix[bi + 1][bj + 1] = -t;

            // Hermitian conjugate H_{i+1, i}
            matrix[bj][bi] = -t;
            matrix[bj + 1][bi] = -alpha_r;
            matrix[bj][bi + 1] = alpha_r;
            matrix[bj + 1][bi + 1] = -t;

            // Hole hopping: -(-t I - i alpha_R sigma_y)* = t I + i alpha_R sigma_y
            // = [ t,   alpha_R ]
            //   [ -alpha_R, t  ]
            matrix[bi + 2][bj + 2] = t;
            matrix[bi + 2][bj + 3] = alpha_r;
            matrix[bi + 3][bj + 2] = -alpha_r;
            matrix[bi + 3][bj + 3] = t;

            // Hermitian conjugate H_{i+1, i} (transpose for real matrix)
            matrix[bj + 2][bi + 2] = t;
            matrix[bj + 3][bi + 2] = alpha_r;
            matrix[bj + 2][bi + 3] = -alpha_r;
            matrix[bj + 3][bi + 3] = t;
        }

        Self {
            params,
            dimension: dim,
            matrix,
        }
    }

    /// Solves the eigensystem using a safe real symmetric Jacobi rotation algorithm.
    /// Returns eigenvalues and corresponding orthonormal eigenvectors.
    #[allow(clippy::needless_range_loop)]
    pub fn solve(&self) -> BdGSolution {
        let n = self.dimension;
        let mut a = self.matrix.clone();
        // Initialize eigenvector matrix V = Identity
        let mut v = vec![vec![0.0; n]; n];
        for i in 0..n {
            v[i][i] = 1.0;
        }

        let max_sweeps = 100;
        let tol = 1e-12;

        for _ in 0..max_sweeps {
            let mut max_off_diag = 0.0;

            for p in 0..(n - 1) {
                for q in (p + 1)..n {
                    let apq = a[p][q];
                    let r = apq.abs();
                    if r > max_off_diag {
                        max_off_diag = r;
                    }

                    if r < tol {
                        continue;
                    }

                    let d = a[q][q] - a[p][p];
                    let tau = d / (2.0 * apq);
                    let t = if tau >= 0.0 {
                        1.0 / (tau + (1.0 + tau * tau).sqrt())
                    } else {
                        -1.0 / (-tau + (1.0 + tau * tau).sqrt())
                    };

                    let c = 1.0 / (1.0 + t * t).sqrt();
                    let s = t * c;

                    // Update diagonal elements
                    a[p][p] -= t * apq;
                    a[q][q] += t * apq;
                    a[p][q] = 0.0;
                    a[q][p] = 0.0;

                    // Update remaining elements in rows/cols p and q
                    for k in 0..n {
                        if k != p && k != q {
                            let akp = a[k][p];
                            let akq = a[k][q];
                            let new_kp = c * akp - s * akq;
                            let new_kq = s * akp + c * akq;

                            a[k][p] = new_kp;
                            a[p][k] = new_kp;
                            a[k][q] = new_kq;
                            a[q][k] = new_kq;
                        }
                    }

                    // Accumulate eigenvectors: V' = V * R
                    for k in 0..n {
                        let vkp = v[k][p];
                        let vkq = v[k][q];
                        v[k][p] = c * vkp - s * vkq;
                        v[k][q] = s * vkp + c * vkq;
                    }
                }
            }

            if max_off_diag < tol {
                break;
            }
        }

        // Extract eigenvalues from diagonal
        let mut indexed_evals: Vec<(usize, f64)> = (0..n).map(|i| (i, a[i][i])).collect();
        indexed_evals.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let sorted_evals: Vec<f64> = indexed_evals.iter().map(|&(_, val)| val).collect();
        let mut sorted_evecs = vec![vec![0.0; n]; n];

        for (new_col, &(old_col, _)) in indexed_evals.iter().enumerate() {
            for row in 0..n {
                sorted_evecs[row][new_col] = v[row][old_col];
            }
        }

        BdGSolution {
            eigenvalues: sorted_evals,
            eigenvectors: sorted_evecs,
        }
    }
}

/// Majorana Zero Mode representation and spatial wavefunction analysis.
#[derive(Debug, Clone)]
pub struct MajoranaNanowire {
    pub params: NanowireParams,
    pub solution: BdGSolution,
    /// Energy of the lowest positive eigenvalue $E_0 \ge 0$ (Majorana splitting energy).
    pub zero_mode_energy_mev: f64,
    /// Spatial probability density of the left Majorana mode $\gamma_1(x_i)$.
    pub gamma1_density: Vec<f64>,
    /// Spatial probability density of the right Majorana mode $\gamma_2(x_i)$.
    pub gamma2_density: Vec<f64>,
    /// Fermion parity of the ground state $P \in \{+1, -1\}$.
    pub ground_state_parity: i8,
}

impl MajoranaNanowire {
    /// Solves the BdG Hamiltonian and extracts the Majorana zero modes and wavefunctions.
    #[allow(clippy::needless_range_loop)]
    pub fn solve(params: NanowireParams) -> Self {
        let bdg = BdGHamiltonian::new(params.clone());
        let solution = bdg.solve();

        let n = params.num_sites;
        let dim = 4 * n;

        // In BdG with particle-hole symmetry, eigenvalues are symmetric around 0.
        // The lowest non-negative eigenvalue is at index dim / 2.
        let mid = dim / 2;
        let e_neg = solution.eigenvalues[mid - 1];
        let e_pos = solution.eigenvalues[mid];
        let zero_mode_energy = (e_pos - e_neg).abs() / 2.0;

        // In the 2D zero-energy subspace spanned by v1 = col(mid-1) and v2 = col(mid):
        // Find rotation theta that maximizes localization of gamma_1 = cos(theta)*v1 + sin(theta)*v2
        // at the left half of the wire.
        let v1_col = mid - 1;
        let v2_col = mid;

        let left_half_sites = (n / 2).max(1);
        let mut q11 = 0.0;
        let mut q22 = 0.0;
        let mut q12 = 0.0;

        for i in 0..left_half_sites {
            let base = 4 * i;
            for c in 0..4 {
                let u1 = solution.eigenvectors[base + c][v1_col];
                let u2 = solution.eigenvectors[base + c][v2_col];
                q11 += u1 * u1;
                q22 += u2 * u2;
                q12 += u1 * u2;
            }
        }

        // Optimal angle theta maximizing left boundary projection
        let theta = 0.5 * (2.0 * q12).atan2(q11 - q22);
        let cos_t = theta.cos();
        let sin_t = theta.sin();

        let mut gamma1_density = vec![0.0; n];
        let mut gamma2_density = vec![0.0; n];

        for i in 0..n {
            let base = 4 * i;
            let mut sum_g1_sq = 0.0;
            let mut sum_g2_sq = 0.0;

            for c in 0..4 {
                let u1 = solution.eigenvectors[base + c][v1_col];
                let u2 = solution.eigenvectors[base + c][v2_col];

                let g1_comp = cos_t * u1 + sin_t * u2;
                let g2_comp = -sin_t * u1 + cos_t * u2;

                sum_g1_sq += g1_comp * g1_comp;
                sum_g2_sq += g2_comp * g2_comp;
            }

            gamma1_density[i] = sum_g1_sq;
            gamma2_density[i] = sum_g2_sq;
        }

        // Ensure gamma1 is the left-localized mode and gamma2 is the right-localized mode
        let g1_left: f64 = gamma1_density[0..left_half_sites].iter().sum();
        let g2_left: f64 = gamma2_density[0..left_half_sites].iter().sum();
        if g2_left > g1_left {
            std::mem::swap(&mut gamma1_density, &mut gamma2_density);
        }

        // Normalize densities to sum to 1.0
        let norm1: f64 = gamma1_density.iter().sum();
        let norm2: f64 = gamma2_density.iter().sum();
        if norm1 > 1e-12 {
            for v in &mut gamma1_density {
                *v /= norm1;
            }
        }
        if norm2 > 1e-12 {
            for v in &mut gamma2_density {
                *v /= norm2;
            }
        }

        // Fermion parity: even (+1) for unoccupied zero mode
        let parity = 1;

        Self {
            params,
            solution,
            zero_mode_energy_mev: zero_mode_energy,
            gamma1_density,
            gamma2_density,
            ground_state_parity: parity,
        }
    }

    /// Verifies exponential localization of $\gamma_1$ at the left end and $\gamma_2$ at the right end.
    /// Returns (left_end_fraction, right_end_fraction).
    pub fn boundary_localization_fractions(&self, end_sites: usize) -> (f64, f64) {
        let n = self.params.num_sites;
        let k = end_sites.min(n / 2);

        let g1_left: f64 = self.gamma1_density[0..k].iter().sum();
        let g2_right: f64 = self.gamma2_density[(n - k)..n].iter().sum();

        (g1_left, g2_right)
    }

    /// Overlap / hybridization energy between the two Majorana modes: $\delta E \propto e^{-L/\xi_M}$.
    #[inline]
    pub fn hybridization_energy_mev(&self) -> f64 {
        self.zero_mode_energy_mev
    }
}
