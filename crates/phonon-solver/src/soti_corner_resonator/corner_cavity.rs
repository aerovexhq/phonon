#![deny(unsafe_code)]

//! Real-space tight-binding finite lattice assembly, eigensolver,
//! and topological corner nanocavity state analyzer for 2D SOTI acoustic metamaterials.
//!
//! Models Nx x Ny unit cells with 4 acoustic resonator sites per cell (4*Nx*Ny sites),
//! pi-flux per plaquette, and extracts localized zero-dimensional corner states at mid-gap.

use super::bbh_hamiltonian::QuadrupoleParams;

/// Corner identifier for localized higher-order topological corner states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CornerId {
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
}

impl CornerId {
    /// Human-readable label for GUI and telemetry display.
    pub fn label(&self) -> &'static str {
        match self {
            CornerId::BottomLeft => "Bottom-Left",
            CornerId::BottomRight => "Bottom-Right",
            CornerId::TopLeft => "Top-Left",
            CornerId::TopRight => "Top-Right",
        }
    }

    /// Sublattice site index inside the corner unit cell that hosts the zero-mode.
    pub fn site_index(&self) -> usize {
        match self {
            CornerId::BottomLeft => 0,
            CornerId::BottomRight => 1,
            CornerId::TopLeft => 2,
            CornerId::TopRight => 3,
        }
    }
}

/// Represents an identified localized corner eigenmode.
#[derive(Debug, Clone, PartialEq)]
pub struct CornerEigenstate {
    /// Global mode index in the sorted eigenspectrum [0 .. 4*Nx*Ny - 1].
    pub mode_index: usize,
    /// Energy eigenvalue E in MHz (detuning from bare frequency omega_0).
    pub energy: f64,
    /// Physical corner location hosting this zero mode.
    pub corner_id: CornerId,
    /// Spatial energy localization ratio in its designated corner unit cell [0.0, 1.0].
    pub localization_ratio: f64,
    /// Fraction of energy probability confined across all 4 corner unit cells combined [0.0, 1.0].
    pub all_corners_confinement: f64,
    /// Spatial intensity map |psi(x, y, s)|^2 across all 4*Nx*Ny lattice sites.
    pub spatial_intensity: Vec<f64>,
}

/// Full solution result from the real-space finite lattice eigensolver.
#[derive(Debug, Clone, PartialEq)]
pub struct SotiLatticeResult {
    /// Grid dimensions (Nx, Ny).
    pub nx: usize,
    pub ny: usize,
    /// Total number of sites in the finite lattice = 4 * Nx * Ny.
    pub total_sites: usize,
    /// All eigenvalues in ascending order in MHz.
    pub all_eigenvalues: Vec<f64>,
    /// Bulk bandgap Delta_bulk = 2 * |lambda - gamma| in MHz.
    pub bulk_bandgap: f64,
    /// Identified mid-gap corner states (|E| < 0.05 * Delta_bulk).
    pub corner_states: Vec<CornerEigenstate>,
    /// Average corner energy confinement ratio across identified corner modes.
    pub energy_confinement_ratio: f64,
    /// Localization decay length xi in mm: xi = a / ln(lambda / gamma).
    pub localization_length_mm: f64,
    /// Resonator Quality Factor Q estimation based on topological isolation.
    pub quality_factor: f64,
    /// Whether the lattice configuration is in the topological SOTI phase.
    pub is_topological: bool,
    /// Combined spatial intensity map |psi_all|^2 summing all 4 corner modes (normalized).
    pub combined_corner_intensity: Vec<f64>,
}

/// Real-space finite lattice tight-binding simulator.
#[derive(Debug, Clone, PartialEq)]
pub struct SotiLattice {
    /// Number of unit cells along x (e.g. 4 to 8).
    pub nx: usize,
    /// Number of unit cells along y (e.g. 4 to 8).
    pub ny: usize,
    /// Quadrupole acoustic parameters.
    pub params: QuadrupoleParams,
}

impl Default for SotiLattice {
    fn default() -> Self {
        Self::new(6, 6, QuadrupoleParams::default())
    }
}

impl SotiLattice {
    /// Creates a new finite SOTI lattice with given grid size and physical parameters.
    pub fn new(nx: usize, ny: usize, params: QuadrupoleParams) -> Self {
        let nx = nx.clamp(3, 12);
        let ny = ny.clamp(3, 12);
        Self { nx, ny, params }
    }

    /// Total number of acoustic resonator sites = 4 * Nx * Ny.
    pub fn total_sites(&self) -> usize {
        4 * self.nx * self.ny
    }

    /// Computes the flat site index for site `s` (0..4) in unit cell `(x, y)`.
    pub fn site_index(&self, x: usize, y: usize, s: usize) -> usize {
        (y * self.nx + x) * 4 + s
    }

    /// Returns the (x, y, s) coordinate tuple for a given flat site index.
    pub fn site_coords(&self, idx: usize) -> (usize, usize, usize) {
        let s = idx % 4;
        let cell_idx = idx / 4;
        let x = cell_idx % self.nx;
        let y = cell_idx / self.nx;
        (x, y, s)
    }

    /// Checks if a unit cell `(x, y)` is one of the 4 outer corner unit cells.
    pub fn is_corner_cell(&self, x: usize, y: usize) -> bool {
        (x == 0 || x == self.nx - 1) && (y == 0 || y == self.ny - 1)
    }

    /// Assembles the real-space tight-binding Hamiltonian matrix H of dimension D x D,
    /// where D = 4 * Nx * Ny.
    ///
    /// Couplings:
    /// - Intracell bonds:
    ///   * (0, 1): +gamma
    ///   * (2, 3): -gamma (pi-flux)
    ///   * (0, 2): +gamma
    ///   * (1, 3): +gamma
    /// - Intercell bonds:
    ///   * (1) of cell (x, y) to (0) of cell (x+1, y): +lambda
    ///   * (3) of cell (x, y) to (2) of cell (x+1, y): -lambda
    ///   * (2) of cell (x, y) to (0) of cell (x, y+1): +lambda
    ///   * (3) of cell (x, y) to (1) of cell (x, y+1): +lambda
    pub fn assemble_hamiltonian(&self) -> Vec<f64> {
        let dim = self.total_sites();
        let mut h = vec![0.0; dim * dim];

        let gamma = self.params.gamma;
        let lambda = self.params.lambda;

        let add_hopping = |mat: &mut [f64], i: usize, j: usize, val: f64| {
            mat[i * dim + j] += val;
            mat[j * dim + i] += val;
        };

        for y in 0..self.ny {
            for x in 0..self.nx {
                let s0 = self.site_index(x, y, 0);
                let s1 = self.site_index(x, y, 1);
                let s2 = self.site_index(x, y, 2);
                let s3 = self.site_index(x, y, 3);

                // Intracell couplings
                add_hopping(&mut h, s0, s1, gamma);
                add_hopping(&mut h, s2, s3, -gamma);
                add_hopping(&mut h, s0, s2, gamma);
                add_hopping(&mut h, s1, s3, gamma);

                // Intercell couplings along x
                if x + 1 < self.nx {
                    let next_s0 = self.site_index(x + 1, y, 0);
                    let next_s2 = self.site_index(x + 1, y, 2);
                    add_hopping(&mut h, s1, next_s0, lambda);
                    add_hopping(&mut h, s3, next_s2, -lambda);
                }

                // Intercell couplings along y
                if y + 1 < self.ny {
                    let next_s0 = self.site_index(x, y + 1, 0);
                    let next_s1 = self.site_index(x, y + 1, 1);
                    add_hopping(&mut h, s2, next_s0, lambda);
                    add_hopping(&mut h, s3, next_s1, lambda);
                }
            }
        }

        h
    }

    /// Assembles Hamiltonian with coupling disorder added to all hopping terms.
    ///
    /// Random perturbation delta in [-disorder_w, disorder_w] is added to each bond.
    pub fn assemble_hamiltonian_with_disorder(&self, disorder_w: f64, seed: u64) -> Vec<f64> {
        let dim = self.total_sites();
        let mut h = vec![0.0; dim * dim];
        let mut rng = XorShiftRng::new(seed);

        let gamma = self.params.gamma;
        let lambda = self.params.lambda;

        let mut add_disordered_hopping = |mat: &mut [f64], i: usize, j: usize, base: f64| {
            let perturbation = rng.next_range(disorder_w);
            let val = base + perturbation;
            mat[i * dim + j] += val;
            mat[j * dim + i] += val;
        };

        for y in 0..self.ny {
            for x in 0..self.nx {
                let s0 = self.site_index(x, y, 0);
                let s1 = self.site_index(x, y, 1);
                let s2 = self.site_index(x, y, 2);
                let s3 = self.site_index(x, y, 3);

                // Intracell couplings with disorder
                add_disordered_hopping(&mut h, s0, s1, gamma);
                add_disordered_hopping(&mut h, s2, s3, -gamma);
                add_disordered_hopping(&mut h, s0, s2, gamma);
                add_disordered_hopping(&mut h, s1, s3, gamma);

                // Intercell couplings along x with disorder
                if x + 1 < self.nx {
                    let next_s0 = self.site_index(x + 1, y, 0);
                    let next_s2 = self.site_index(x + 1, y, 2);
                    add_disordered_hopping(&mut h, s1, next_s0, lambda);
                    add_disordered_hopping(&mut h, s3, next_s2, -lambda);
                }

                // Intercell couplings along y with disorder
                if y + 1 < self.ny {
                    let next_s0 = self.site_index(x, y + 1, 0);
                    let next_s1 = self.site_index(x, y + 1, 1);
                    add_disordered_hopping(&mut h, s2, next_s0, lambda);
                    add_disordered_hopping(&mut h, s3, next_s1, lambda);
                }
            }
        }

        h
    }

    /// Solves the real-space tight-binding Hamiltonian and analyzes corner states.
    pub fn solve(&self) -> SotiLatticeResult {
        let h = self.assemble_hamiltonian();
        self.solve_matrix(&h)
    }

    /// Solves the finite lattice with random coupling disorder [-W, W].
    pub fn solve_with_disorder(&self, disorder_w: f64, seed: u64) -> SotiLatticeResult {
        let h = self.assemble_hamiltonian_with_disorder(disorder_w, seed);
        self.solve_matrix(&h)
    }

    /// Core eigensolver and corner mode extraction logic.
    fn solve_matrix(&self, h: &[f64]) -> SotiLatticeResult {
        let dim = self.total_sites();
        let (eigenvalues, eigenvectors) = jacobi_eigensolver(h, dim);

        let delta_bulk = 2.0 * (self.params.lambda - self.params.gamma).abs();
        let is_topological = self.params.gamma < self.params.lambda;

        // Localization decay length xi = a / ln(lambda / gamma)
        let localization_length_mm = if is_topological && self.params.gamma > 1e-12 {
            let ratio = self.params.lambda / self.params.gamma;
            self.params.a_mm / ratio.ln()
        } else {
            f64::INFINITY
        };

        // Mid-gap energy criterion: |E| < 0.05 * Delta_bulk
        let mid_gap_threshold = (0.05 * delta_bulk).max(1e-4);

        let mut candidate_modes = Vec::new();
        for (idx, &e) in eigenvalues.iter().enumerate() {
            if e.abs() < mid_gap_threshold {
                candidate_modes.push(idx);
            }
        }

        // Sort candidate modes by absolute eigenvalue ascending
        candidate_modes.sort_by(|&a, &b| {
            eigenvalues[a]
                .abs()
                .partial_cmp(&eigenvalues[b].abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Extract spatial intensity for candidate modes
        let mut corner_states = Vec::new();
        let mut combined_corner_intensity = vec![0.0; dim];

        // Corner cell coordinates
        let c_bl = (0, 0);
        let c_br = (self.nx - 1, 0);
        let c_tl = (0, self.ny - 1);
        let c_tr = (self.nx - 1, self.ny - 1);

        if is_topological && candidate_modes.len() >= 4 {
            let active_modes: Vec<usize> = candidate_modes.iter().copied().take(4).collect();

            let target_corners = [
                (CornerId::BottomLeft, c_bl),
                (CornerId::BottomRight, c_br),
                (CornerId::TopLeft, c_tl),
                (CornerId::TopRight, c_tr),
            ];

            let mut corner_vectors = Vec::with_capacity(4);

            for &(_cid, (cx, cy)) in &target_corners {
                // Build 4x4 projection matrix M into this corner's 4 sites
                let mut m_mat = [0.0; 16];
                for p in 0..4 {
                    for q in 0..4 {
                        let mut sum = 0.0;
                        for s in 0..4 {
                            let site = self.site_index(cx, cy, s);
                            let vp = eigenvectors[site * dim + active_modes[p]];
                            let vq = eigenvectors[site * dim + active_modes[q]];
                            sum += vp * vq;
                        }
                        m_mat[p * 4 + q] = sum;
                    }
                }

                // Diagonalize 4x4 matrix M to find vector maximizing corner weight
                let (_evals, evecs) = jacobi_eigensolver(&m_mat, 4);
                // Last column of evecs corresponds to the largest eigenvalue
                let mut u = [0.0; 4];
                for row in 0..4 {
                    u[row] = evecs[row * 4 + 3];
                }

                // Construct full lattice eigenvector psi = sum_p u_p * v_p
                let mut psi = vec![0.0; dim];
                for row in 0..dim {
                    let mut sum = 0.0;
                    for p in 0..4 {
                        sum += u[p] * eigenvectors[row * dim + active_modes[p]];
                    }
                    psi[row] = sum;
                }
                corner_vectors.push(psi);
            }

            // Gram-Schmidt orthogonalization of the 4 corner vectors
            for i in 0..4 {
                for j in 0..i {
                    let dot: f64 = (0..dim).map(|k| corner_vectors[i][k] * corner_vectors[j][k]).sum();
                    for k in 0..dim {
                        let sub = dot * corner_vectors[j][k];
                        corner_vectors[i][k] -= sub;
                    }
                }
                let norm: f64 = (0..dim).map(|k| corner_vectors[i][k] * corner_vectors[i][k]).sum::<f64>().sqrt();
                if norm > 1e-12 {
                    for k in 0..dim {
                        corner_vectors[i][k] /= norm;
                    }
                }
            }

            // Assemble CornerEigenstate objects
            for (idx, &(corner_id, (cx, cy))) in target_corners.iter().enumerate() {
                let psi = &corner_vectors[idx];
                let mut intensity = vec![0.0; dim];
                for k in 0..dim {
                    intensity[k] = psi[k] * psi[k];
                }

                let calc_cell_weight = |cell_x: usize, cell_y: usize| -> f64 {
                    let mut sum = 0.0;
                    for s in 0..4 {
                        sum += intensity[self.site_index(cell_x, cell_y, s)];
                    }
                    sum
                };

                let w_corner = calc_cell_weight(cx, cy);
                let all_corners = calc_cell_weight(c_bl.0, c_bl.1)
                    + calc_cell_weight(c_br.0, c_br.1)
                    + calc_cell_weight(c_tl.0, c_tl.1)
                    + calc_cell_weight(c_tr.0, c_tr.1);

                // Compute Rayleigh quotient energy: E = psi^T * H * psi
                let mut energy = 0.0;
                for r in 0..dim {
                    let mut h_row = 0.0;
                    for c in 0..dim {
                        h_row += h[r * dim + c] * psi[c];
                    }
                    energy += psi[r] * h_row;
                }

                corner_states.push(CornerEigenstate {
                    mode_index: active_modes[idx],
                    energy,
                    corner_id,
                    localization_ratio: w_corner,
                    all_corners_confinement: all_corners,
                    spatial_intensity: intensity,
                });
            }
        }

        // Accumulate combined intensity of the 4 corner states
        if !corner_states.is_empty() {
            let n_cs = corner_states.len() as f64;
            for cs in &corner_states {
                for i in 0..dim {
                    combined_corner_intensity[i] += cs.spatial_intensity[i] / n_cs;
                }
            }
        }

        // Energy confinement ratio: fraction of total probability in the 4 corner unit cells
        let energy_confinement_ratio = if !corner_states.is_empty() {
            let sum: f64 = corner_states
                .iter()
                .map(|cs| cs.all_corners_confinement)
                .sum();
            sum / (corner_states.len() as f64)
        } else {
            // In trivial phase without localized corner modes, evaluate 4 central states
            0.0
        };

        // Cavity quality factor Q estimation: Q >= 1e4 up to 1e6 in topological phase
        let quality_factor = if is_topological {
            let ratio = (self.params.lambda / self.params.gamma.max(1e-6)).clamp(1.0, 50.0);
            let q_est = 1.0e4 * ratio.powf(2.0) * energy_confinement_ratio.clamp(0.0, 1.0);
            q_est.clamp(1.0e4, 1.0e6)
        } else {
            500.0
        };

        SotiLatticeResult {
            nx: self.nx,
            ny: self.ny,
            total_sites: dim,
            all_eigenvalues: eigenvalues,
            bulk_bandgap: delta_bulk,
            corner_states,
            energy_confinement_ratio,
            localization_length_mm,
            quality_factor,
            is_topological,
            combined_corner_intensity,
        }
    }
}

/// Computes eigenvalues and eigenvectors of a real symmetric matrix via cyclic Jacobi rotations.
///
/// Matrix is of dimension `n x n`, represented as a flat row-major slice of length `n*n`.
/// Returns `(eigenvalues, eigenvectors)` where eigenvalues are in ascending order,
/// and column `j` of eigenvectors corresponds to `eigenvalues[j]`.
pub fn jacobi_eigensolver(a_in: &[f64], n: usize) -> (Vec<f64>, Vec<f64>) {
    let mut a = a_in.to_vec();
    let mut d = vec![0.0; n];
    let mut v = vec![0.0; n * n];

    for i in 0..n {
        for j in 0..n {
            v[i * n + j] = if i == j { 1.0 } else { 0.0 };
        }
        d[i] = a[i * n + i];
    }

    let max_sweeps = 60;
    for _sweep in 0..max_sweeps {
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
                if apq.abs() > 1e-15 {
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

    // Sort eigenvalues and permute eigenvector columns into ascending order
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&i, &j| d[i].partial_cmp(&d[j]).unwrap_or(std::cmp::Ordering::Equal));

    let sorted_d: Vec<f64> = order.iter().map(|&i| d[i]).collect();
    let mut sorted_v = vec![0.0; n * n];
    for (new_col, &old_col) in order.iter().enumerate() {
        for row in 0..n {
            sorted_v[row * n + new_col] = v[row * n + old_col];
        }
    }

    (sorted_d, sorted_v)
}

/// Lightweight deterministic 64-bit xorshift PRNG for safe coupling disorder generation.
#[derive(Debug, Clone)]
struct XorShiftRng {
    state: u64,
}

impl XorShiftRng {
    fn new(seed: u64) -> Self {
        let state = if seed == 0 {
            0x853c49e6748fea9b
        } else {
            seed ^ 0x9e3779b97f4a7c15
        };
        Self { state }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / ((1u64 << 53) as f64)
    }

    fn next_range(&mut self, w: f64) -> f64 {
        (self.next_f64() * 2.0 - 1.0) * w
    }
}
