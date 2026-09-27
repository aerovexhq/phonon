//! Ab initio Density Functional Theory (DFT) and Wannier90 tight-binding interface.
//!
//! Formulates:
//! - Real-space tight-binding Hamiltonian: $H_{ij}(\mathbf{R}) = \langle \mathbf{0}, i | \hat{H} | \mathbf{R}, j \rangle$
//! - Reciprocal-space Fourier transform: $H(\mathbf{k}) = \sum_{\mathbf{R}} H(\mathbf{R}) \exp(i \mathbf{k} \cdot \mathbf{R})$
//! - Safe Hermitian Jacobi eigenvalue solver for electronic band dispersion $E_n(\mathbf{k})$
//! - Gaussian-broadened electronic Density of States (DOS) and Fermi level calculation.

use crate::quantum::Complex;
use phonon_core::BOLTZMANN_CONSTANT;

/// A single hopping element in the real-space Wannier tight-binding representation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WannierHopping {
    /// Bravais lattice vector indices $(R_x, R_y, R_z)$ in unit cell multiples.
    pub r_indices: [i32; 3],
    /// Target orbital index $i$ in unit cell $\mathbf{0}$ (0-indexed).
    pub orb_i: usize,
    /// Source orbital index $j$ in unit cell $\mathbf{R}$ (0-indexed).
    pub orb_j: usize,
    /// Complex hopping matrix element $H_{ij}(\mathbf{R})$ in electron-volts ($eV$).
    pub hopping_ev: Complex,
}

/// Real-space Wannier tight-binding Hamiltonian.
#[derive(Debug, Clone, PartialEq)]
pub struct WannierHamiltonian {
    /// Number of Wannier orbitals / basis functions per unit cell.
    pub num_orbitals: usize,
    /// Real-space Bravais lattice basis vectors $\mathbf{a}_1, \mathbf{a}_2, \mathbf{a}_3$ in Angstroms ($\text{Å}$).
    pub lattice_vectors: [[f64; 3]; 3],
    /// Hopping matrix elements $H_{ij}(\mathbf{R})$.
    pub hoppings: Vec<WannierHopping>,
}

impl WannierHamiltonian {
    /// Creates a new empty Wannier Hamiltonian with given number of orbitals and lattice vectors.
    pub fn new(num_orbitals: usize, lattice_vectors: [[f64; 3]; 3]) -> Self {
        Self {
            num_orbitals,
            lattice_vectors,
            hoppings: Vec::new(),
        }
    }

    /// Adds a real-space hopping matrix element $H_{ij}(\mathbf{R})$.
    pub fn add_hopping(
        &mut self,
        r_indices: [i32; 3],
        orb_i: usize,
        orb_j: usize,
        hopping: Complex,
    ) {
        assert!(orb_i < self.num_orbitals, "Orbital index i out of bounds");
        assert!(orb_j < self.num_orbitals, "Orbital index j out of bounds");
        self.hoppings.push(WannierHopping {
            r_indices,
            orb_i,
            orb_j,
            hopping_ev: hopping,
        });
    }

    /// Parses standard Wannier90 real-space Hamiltonian file (`prefix_hr.dat`).
    ///
    /// Standard Wannier90 `_hr.dat` format:
    /// - Header line (comment / timestamp)
    /// - `num_wann` (number of orbitals)
    /// - `nrpts` (number of Wigner-Seitz R vectors)
    /// - List of multiplicities (ignored for direct hopping summation)
    /// - Rows of `Rx Ry Rz i j Re(H) Im(H)`
    pub fn from_hr_text(content: &str, lattice_vectors: [[f64; 3]; 3]) -> Result<Self, String> {
        let mut lines = content.lines().map(|l| l.trim()).filter(|l| !l.is_empty());

        // Skip comment line
        lines
            .next()
            .ok_or_else(|| "Empty _hr.dat file".to_string())?;

        let num_wann_str = lines
            .next()
            .ok_or_else(|| "Missing num_wann in _hr.dat".to_string())?;
        let num_orbitals: usize = num_wann_str
            .parse()
            .map_err(|e| format!("Invalid num_wann '{num_wann_str}': {e}"))?;

        let nrpts_str = lines
            .next()
            .ok_or_else(|| "Missing nrpts in _hr.dat".to_string())?;
        let nrpts: usize = nrpts_str
            .parse()
            .map_err(|e| format!("Invalid nrpts '{nrpts_str}': {e}"))?;

        // Degeneracies list spans ceil(nrpts / 15) lines
        let mut deg_read = 0;
        while deg_read < nrpts {
            let line = lines
                .next()
                .ok_or_else(|| "Premature EOF while reading degeneracies".to_string())?;
            let tokens_count = line.split_whitespace().count();
            deg_read += tokens_count;
        }

        let mut hamiltonian = Self::new(num_orbitals, lattice_vectors);

        for line in lines {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens.len() < 7 {
                continue;
            }

            let rx: i32 = tokens[0]
                .parse()
                .map_err(|e| format!("Failed parsing rx in line '{line}': {e}"))?;
            let ry: i32 = tokens[1]
                .parse()
                .map_err(|e| format!("Failed parsing ry in line '{line}': {e}"))?;
            let rz: i32 = tokens[2]
                .parse()
                .map_err(|e| format!("Failed parsing rz in line '{line}': {e}"))?;
            let orb_i: usize = tokens[3]
                .parse::<usize>()
                .map_err(|e| format!("Failed parsing orb_i: {e}"))?
                - 1; // 1-indexed to 0-indexed
            let orb_j: usize = tokens[4]
                .parse::<usize>()
                .map_err(|e| format!("Failed parsing orb_j: {e}"))?
                - 1;
            let re: f64 = tokens[5]
                .parse()
                .map_err(|e| format!("Failed parsing re(H): {e}"))?;
            let im: f64 = tokens[6]
                .parse()
                .map_err(|e| format!("Failed parsing im(H): {e}"))?;

            hamiltonian.add_hopping([rx, ry, rz], orb_i, orb_j, Complex::new(re, im));
        }

        Ok(hamiltonian)
    }

    /// Evaluates the $N \times N$ complex Hermitian Hamiltonian $H(\mathbf{k})$ at wavevector $\mathbf{k}$ (in $\text{Å}^{-1}$):
    /// $$H_{ij}(\mathbf{k}) = \sum_{\mathbf{R}} H_{ij}(\mathbf{R}) \exp(i \mathbf{k} \cdot \mathbf{R})$$
    #[allow(clippy::needless_range_loop)]
    pub fn evaluate_k_space(&self, k_vec: &[f64; 3]) -> Vec<Vec<Complex>> {
        let n = self.num_orbitals;
        let mut h_k = vec![vec![Complex::ZERO; n]; n];

        for hop in &self.hoppings {
            // R = Rx * a1 + Ry * a2 + Rz * a3
            let rx = hop.r_indices[0] as f64 * self.lattice_vectors[0][0]
                + hop.r_indices[1] as f64 * self.lattice_vectors[1][0]
                + hop.r_indices[2] as f64 * self.lattice_vectors[2][0];
            let ry = hop.r_indices[0] as f64 * self.lattice_vectors[0][1]
                + hop.r_indices[1] as f64 * self.lattice_vectors[1][1]
                + hop.r_indices[2] as f64 * self.lattice_vectors[2][1];
            let rz = hop.r_indices[0] as f64 * self.lattice_vectors[0][2]
                + hop.r_indices[1] as f64 * self.lattice_vectors[1][2]
                + hop.r_indices[2] as f64 * self.lattice_vectors[2][2];

            let k_dot_r = k_vec[0] * rx + k_vec[1] * ry + k_vec[2] * rz;
            let phase = Complex::new(k_dot_r.cos(), k_dot_r.sin());
            let term = hop.hopping_ev.mul(phase);

            h_k[hop.orb_i][hop.orb_j] = h_k[hop.orb_i][hop.orb_j].add(term);
        }

        // Enforce strict Hermitian symmetry: H_ji = H_ij^*
        for i in 0..n {
            h_k[i][i].im = 0.0;
            for j in (i + 1)..n {
                let avg_re = 0.5 * (h_k[i][j].re + h_k[j][i].re);
                let avg_im = 0.5 * (h_k[i][j].im - h_k[j][i].im);
                h_k[i][j] = Complex::new(avg_re, avg_im);
                h_k[j][i] = Complex::new(avg_re, -avg_im);
            }
        }

        h_k
    }

    /// Computes sorted real eigenvalues $E_1 \le E_2 \le \dots \le E_N$ of $H(\mathbf{k})$
    /// using safe complex Hermitian Jacobi rotation.
    #[allow(clippy::needless_range_loop)]
    pub fn eigenvalues_at_k(&self, k_vec: &[f64; 3]) -> Vec<f64> {
        let n = self.num_orbitals;
        let mut a = self.evaluate_k_space(k_vec);

        // Cyclic Jacobi rotations
        let max_sweeps = 50;
        let tol = 1e-12;

        for _ in 0..max_sweeps {
            let mut max_off_diag = 0.0;

            for p in 0..(n - 1) {
                for q in (p + 1)..n {
                    let apq = a[p][q];
                    let r = apq.norm_sq().sqrt();
                    if r > max_off_diag {
                        max_off_diag = r;
                    }

                    if r < tol {
                        continue;
                    }

                    // Phase angle theta to make A_pq real
                    let theta = apq.im.atan2(apq.re);
                    let phase = Complex::new((-theta).cos(), (-theta).sin());

                    let d = a[q][q].re - a[p][p].re;
                    let tau = d / (2.0 * r);
                    let t = if tau >= 0.0 {
                        1.0 / (tau + (1.0 + tau * tau).sqrt())
                    } else {
                        -1.0 / (-tau + (1.0 + tau * tau).sqrt())
                    };

                    let c = 1.0 / (1.0 + t * t).sqrt();
                    let s = t * c;

                    // Update diagonals
                    a[p][p].re -= t * r;
                    a[q][q].re += t * r;
                    a[p][q] = Complex::ZERO;
                    a[q][p] = Complex::ZERO;

                    // Update off-diagonals
                    for k in 0..n {
                        if k != p && k != q {
                            let akp = a[k][p];
                            let akq = a[k][q];

                            // Unitary transformation components
                            let akq_rot = akq.mul(phase.conj());
                            let new_kp = akp
                                .mul(Complex::new(c, 0.0))
                                .sub(akq_rot.mul(Complex::new(s, 0.0)));
                            let new_kq = akp
                                .mul(phase)
                                .mul(Complex::new(s, 0.0))
                                .add(akq.mul(Complex::new(c, 0.0)));

                            a[k][p] = new_kp;
                            a[p][k] = new_kp.conj();
                            a[k][q] = new_kq;
                            a[q][k] = new_kq.conj();
                        }
                    }
                }
            }

            if max_off_diag < tol {
                break;
            }
        }

        let mut evals: Vec<f64> = (0..n).map(|i| a[i][i].re).collect();
        evals.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
        evals
    }

    /// Evaluates electronic band dispersion $E_n(\mathbf{k})$ along a list of k-vectors (e.g. high-symmetry path).
    pub fn band_dispersion(&self, k_path: &[[f64; 3]]) -> Vec<Vec<f64>> {
        k_path.iter().map(|k| self.eigenvalues_at_k(k)).collect()
    }

    /// Calculates Gaussian-broadened Density of States (DOS) $\text{DOS}(E)$ in $\text{states} / eV$:
    /// $$\text{DOS}(E) = \frac{1}{N_{\mathbf{k}}} \sum_{n, \mathbf{k}} \frac{1}{\sqrt{2\pi}\sigma} \exp\left( -\frac{(E - E_n(\mathbf{k}))^2}{2\sigma^2} \right)$$
    pub fn density_of_states(
        &self,
        k_grid: &[[f64; 3]],
        energy_mesh: &[f64],
        sigma_ev: f64,
    ) -> Vec<f64> {
        let n_k = k_grid.len().max(1) as f64;
        let sigma = sigma_ev.max(1e-4);
        let prefactor = 1.0 / (sigma * (2.0 * std::f64::consts::PI).sqrt() * n_k);
        let two_sigma_sq = 2.0 * sigma * sigma;

        // Compute all eigenvalues on the grid
        let all_evals: Vec<Vec<f64>> = k_grid.iter().map(|k| self.eigenvalues_at_k(k)).collect();

        energy_mesh
            .iter()
            .map(|&e| {
                let mut dos = 0.0;
                for evals in &all_evals {
                    for &e_n in evals {
                        let diff = e - e_n;
                        dos += (-diff * diff / two_sigma_sq).exp();
                    }
                }
                dos * prefactor
            })
            .collect()
    }

    /// Computes Fermi energy $E_F$ in $eV$ for a given target electron count per unit cell $N_e$
    /// at temperature $T$ using bisection on the integrated occupation:
    /// $$N_e = \frac{1}{N_{\mathbf{k}}} \sum_{n, \mathbf{k}} \frac{1}{1 + \exp\left( \frac{E_n(\mathbf{k}) - E_F}{k_B T} \right)}$$
    pub fn fermi_energy(&self, k_grid: &[[f64; 3]], num_electrons: f64, temp_k: f64) -> f64 {
        let n_k = k_grid.len().max(1) as f64;
        let kb_ev = BOLTZMANN_CONSTANT / phonon_core::ELEMENTARY_CHARGE;
        let kt = (kb_ev * temp_k).max(1e-5);

        let all_evals: Vec<Vec<f64>> = k_grid.iter().map(|k| self.eigenvalues_at_k(k)).collect();

        let mut e_min = -50.0;
        let mut e_max = 50.0;

        for _ in 0..50 {
            let e_mid = 0.5 * (e_min + e_max);
            let mut total_occ = 0.0;

            for evals in &all_evals {
                for &e_n in evals {
                    let arg = ((e_n - e_mid) / kt).clamp(-40.0, 40.0);
                    total_occ += 1.0 / (1.0 + arg.exp());
                }
            }

            let n_calc = total_occ / n_k;
            if n_calc < num_electrons {
                e_min = e_mid;
            } else {
                e_max = e_mid;
            }
        }

        0.5 * (e_min + e_max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1d_tight_binding_dispersion_and_dos() {
        // 1D atomic chain with lattice constant a = 3.0 A, onsite eps = 0.0, hopping t = -1.5 eV
        let a = 3.0;
        let lattice = [[a, 0.0, 0.0], [0.0, 10.0, 0.0], [0.0, 0.0, 10.0]];
        let mut model = WannierHamiltonian::new(1, lattice);

        // Onsite R = (0, 0, 0)
        model.add_hopping([0, 0, 0], 0, 0, Complex::new(0.0, 0.0));
        // Nearest-neighbor forward R = (1, 0, 0)
        model.add_hopping([1, 0, 0], 0, 0, Complex::new(-1.5, 0.0));
        // Nearest-neighbor backward R = (-1, 0, 0)
        model.add_hopping([-1, 0, 0], 0, 0, Complex::new(-1.5, 0.0));

        // Theoretical dispersion: E(k) = eps - 2*t*cos(k*a) = -3.0 * cos(k*a)
        // At Gamma (k = 0): E = -3.0 eV
        let evals_gamma = model.eigenvalues_at_k(&[0.0, 0.0, 0.0]);
        assert!((evals_gamma[0] - (-3.0)).abs() < 1e-6);

        // At Brillouin zone edge X (k = pi / a): E = +3.0 eV
        let k_x = std::f64::consts::PI / a;
        let evals_x = model.eigenvalues_at_k(&[k_x, 0.0, 0.0]);
        assert!((evals_x[0] - 3.0).abs() < 1e-6);

        // Half-filled band (1 electron per unit cell) -> Fermi level at E = 0.0 eV
        let k_grid: Vec<[f64; 3]> = (0..50)
            .map(|i| {
                [
                    (i as f64 / 50.0) * (2.0 * std::f64::consts::PI / a),
                    0.0,
                    0.0,
                ]
            })
            .collect();
        let ef = model.fermi_energy(&k_grid, 0.5, 300.0);
        assert!(ef.abs() < 0.1);
    }
}
