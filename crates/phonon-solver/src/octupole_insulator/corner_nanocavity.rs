#![deny(unsafe_code)]

//! Real-space 3D cubic tight-binding lattice assembly, eigensolver,
//! and topological corner nanocavity state analyzer for 3D acoustic octupole metamaterials.
//!
//! Models Nx x Ny x Nz unit cells with 8 acoustic resonator sites per cell (8*Nx*Ny*Nz sites),
//! bulk pi-flux cubic plaquettes, and extracts the 8 localized zero-dimensional corner states at mid-gap.

use super::octupole_hamiltonian::OctupoleParams;

/// Corner identifier for the 8 localized 3D cubic corner states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CubicCornerId {
    /// Bottom-South-West: (0, 0, 0)
    Corner000,
    /// Bottom-South-East: (Nx-1, 0, 0)
    Corner100,
    /// Bottom-North-West: (0, Ny-1, 0)
    Corner010,
    /// Bottom-North-East: (Nx-1, Ny-1, 0)
    Corner110,
    /// Top-South-West: (0, 0, Nz-1)
    Corner001,
    /// Top-South-East: (Nx-1, 0, Nz-1)
    Corner101,
    /// Top-North-West: (0, Ny-1, Nz-1)
    Corner011,
    /// Top-North-East: (Nx-1, Ny-1, Nz-1)
    Corner111,
}

impl CubicCornerId {
    /// Human-readable label for UI and telemetry.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Corner000 => "Corner (0,0,0) [BSW]",
            Self::Corner100 => "Corner (1,0,0) [BSE]",
            Self::Corner010 => "Corner (0,1,0) [BNW]",
            Self::Corner110 => "Corner (1,1,0) [BNE]",
            Self::Corner001 => "Corner (0,0,1) [TSW]",
            Self::Corner101 => "Corner (1,0,1) [TSE]",
            Self::Corner011 => "Corner (0,1,1) [TNW]",
            Self::Corner111 => "Corner (1,1,1) [TNE]",
        }
    }

    /// Primary sublattice site index (0..7) within the unit cell hosting this corner mode.
    pub fn primary_site(&self) -> usize {
        match self {
            Self::Corner000 => 0,
            Self::Corner100 => 1,
            Self::Corner010 => 2,
            Self::Corner110 => 3,
            Self::Corner001 => 4,
            Self::Corner101 => 5,
            Self::Corner011 => 6,
            Self::Corner111 => 7,
        }
    }
}

/// Represents an identified localized 3D corner eigenmode.
#[derive(Debug, Clone, PartialEq)]
pub struct OctupoleCornerState {
    /// Global mode index in the eigenspectrum.
    pub mode_index: usize,
    /// Energy eigenvalue E in MHz (detuning from bare frequency omega_0).
    pub energy: f64,
    /// Physical corner hosting this zero mode.
    pub corner_id: CubicCornerId,
    /// Spatial energy localization ratio in its designated corner unit cell [0.0, 1.0].
    pub localization_ratio: f64,
    /// Fraction of energy probability confined across all 8 corner unit cells combined [0.0, 1.0].
    pub all_corners_confinement: f64,
    /// Spatial intensity map across all lattice sites.
    pub spatial_intensity: Vec<f64>,
}

/// Full solution result from the real-space 3D finite lattice eigensolver.
#[derive(Debug, Clone, PartialEq)]
pub struct OctupoleLatticeResult {
    /// Grid dimensions (Nx, Ny, Nz).
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    /// Total number of sites in the finite lattice = 8 * Nx * Ny * Nz.
    pub total_sites: usize,
    /// All eigenvalues in ascending order in MHz.
    pub all_eigenvalues: Vec<f64>,
    /// Bulk bandgap Delta_bulk in MHz.
    pub bulk_bandgap: f64,
    /// Identified mid-gap corner states (|E| < 0.1 * Delta_bulk).
    pub corner_states: Vec<OctupoleCornerState>,
    /// Average corner energy confinement ratio across identified corner modes.
    pub energy_confinement_ratio: f64,
    /// Localization decay length xi in mm: xi = a / ln(lambda / gamma).
    pub localization_length_mm: f64,
    /// Resonator Quality Factor Q estimation based on topological isolation.
    pub quality_factor: f64,
    /// Whether the lattice configuration is in the topological octupole phase.
    pub is_topological: bool,
    /// Spatial intensity combined across all 8 corner modes.
    pub combined_corner_intensity: Vec<f64>,
}

/// Defect robustness telemetry point.
#[derive(Debug, Clone, PartialEq)]
pub struct DefectRobustnessPoint {
    /// Disorder amplitude W in MHz.
    pub disorder_w: f64,
    /// Mean absolute corner mode energy in MHz.
    pub mean_corner_energy_mhz: f64,
    /// Corner energy confinement ratio as a percentage.
    pub confinement_pct: f64,
    /// Quality factor Q.
    pub quality_factor: f64,
}

/// Fast pseudo-random number generator for disorder perturbations (XorShift64).
#[derive(Debug, Clone)]
struct XorShiftRng {
    state: u64,
}

impl XorShiftRng {
    fn new(seed: u64) -> Self {
        let state = if seed == 0 {
            0x8a5cd789635d2dff
        } else {
            seed
        };
        Self { state }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn next_range(&mut self, amplitude: f64) -> f64 {
        let u = (self.next_u64() as f64) / (u64::MAX as f64);
        (2.0 * u - 1.0) * amplitude
    }
}

/// Real-space 3D cubic tight-binding metamaterial lattice.
#[derive(Debug, Clone, PartialEq)]
pub struct OctupoleCubicLattice {
    pub params: OctupoleParams,
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
}

impl OctupoleCubicLattice {
    /// Creates a new finite cubic lattice with specified dimensions.
    pub fn new(params: OctupoleParams, nx: usize, ny: usize, nz: usize) -> Self {
        Self {
            params,
            nx: nx.max(2),
            ny: ny.max(2),
            nz: nz.max(2),
        }
    }

    /// Creates a standard 3x3x3 cubic lattice.
    pub fn default_3x3x3(params: OctupoleParams) -> Self {
        Self::new(params, 3, 3, 3)
    }

    /// Total number of sites in the finite lattice = 8 * Nx * Ny * Nz.
    pub fn total_sites(&self) -> usize {
        8 * self.nx * self.ny * self.nz
    }

    /// Global linear index of site (sx, sy, sz) in unit cell (x, y, z).
    ///
    /// Sublattice index s = sz * 4 + sy * 2 + sx in [0..7].
    #[inline]
    pub fn site_index(&self, x: usize, y: usize, z: usize, sx: usize, sy: usize, sz: usize) -> usize {
        let cell_idx = (z * self.ny + y) * self.nx + x;
        let s = sz * 4 + sy * 2 + sx;
        cell_idx * 8 + s
    }

    /// Assembles the unperturbed real symmetric Hamiltonian matrix (flat row-major).
    pub fn assemble_hamiltonian(&self) -> Vec<f64> {
        let dim = self.total_sites();
        let mut h = vec![0.0; dim * dim];

        let gamma = self.params.gamma;
        let lambda = self.params.lambda;

        let add_hopping = |mat: &mut [f64], i: usize, j: usize, val: f64| {
            mat[i * dim + j] += val;
            mat[j * dim + i] += val;
        };

        for z in 0..self.nz {
            for y in 0..self.ny {
                for x in 0..self.nx {
                    // Intracell couplings along x: connect sx=0 and sx=1
                    // Sign = (-1)^sy
                    for sz in 0..2 {
                        for sy in 0..2 {
                            let sign = if sy == 0 { 1.0 } else { -1.0 };
                            let i = self.site_index(x, y, z, 0, sy, sz);
                            let j = self.site_index(x, y, z, 1, sy, sz);
                            add_hopping(&mut h, i, j, gamma * sign);
                        }
                    }

                    // Intracell couplings along y: connect sy=0 and sy=1
                    // Sign = (-1)^sz
                    for sz in 0..2 {
                        let sign = if sz == 0 { 1.0 } else { -1.0 };
                        for sx in 0..2 {
                            let i = self.site_index(x, y, z, sx, 0, sz);
                            let j = self.site_index(x, y, z, sx, 1, sz);
                            add_hopping(&mut h, i, j, gamma * sign);
                        }
                    }

                    // Intracell couplings along z: connect sz=0 and sz=1
                    // Sign = (-1)^sx
                    for sy in 0..2 {
                        for sx in 0..2 {
                            let sign = if sx == 0 { 1.0 } else { -1.0 };
                            let i = self.site_index(x, y, z, sx, sy, 0);
                            let j = self.site_index(x, y, z, sx, sy, 1);
                            add_hopping(&mut h, i, j, gamma * sign);
                        }
                    }

                    // Intercell couplings along x: connect (x, sx=1) to (x+1, sx=0)
                    if x + 1 < self.nx {
                        for sz in 0..2 {
                            for sy in 0..2 {
                                let sign = if sy == 0 { 1.0 } else { -1.0 };
                                let i = self.site_index(x, y, z, 1, sy, sz);
                                let j = self.site_index(x + 1, y, z, 0, sy, sz);
                                add_hopping(&mut h, i, j, lambda * sign);
                            }
                        }
                    }

                    // Intercell couplings along y: connect (y, sy=1) to (y+1, sy=0)
                    if y + 1 < self.ny {
                        for sz in 0..2 {
                            let sign = if sz == 0 { 1.0 } else { -1.0 };
                            for sx in 0..2 {
                                let i = self.site_index(x, y, z, sx, 1, sz);
                                let j = self.site_index(x, y + 1, z, sx, 0, sz);
                                add_hopping(&mut h, i, j, lambda * sign);
                            }
                        }
                    }

                    // Intercell couplings along z: connect (z, sz=1) to (z+1, sz=0)
                    if z + 1 < self.nz {
                        for sy in 0..2 {
                            for sx in 0..2 {
                                let sign = if sx == 0 { 1.0 } else { -1.0 };
                                let i = self.site_index(x, y, z, sx, sy, 1);
                                let j = self.site_index(x, y, z + 1, sx, sy, 0);
                                add_hopping(&mut h, i, j, lambda * sign);
                            }
                        }
                    }
                }
            }
        }

        h
    }

    /// Assembles Hamiltonian with random coupling disorder in [-W, W].
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

        for z in 0..self.nz {
            for y in 0..self.ny {
                for x in 0..self.nx {
                    // Intracell x
                    for sz in 0..2 {
                        for sy in 0..2 {
                            let sign = if sy == 0 { 1.0 } else { -1.0 };
                            let i = self.site_index(x, y, z, 0, sy, sz);
                            let j = self.site_index(x, y, z, 1, sy, sz);
                            add_disordered_hopping(&mut h, i, j, gamma * sign);
                        }
                    }

                    // Intracell y
                    for sz in 0..2 {
                        let sign = if sz == 0 { 1.0 } else { -1.0 };
                        for sx in 0..2 {
                            let i = self.site_index(x, y, z, sx, 0, sz);
                            let j = self.site_index(x, y, z, sx, 1, sz);
                            add_disordered_hopping(&mut h, i, j, gamma * sign);
                        }
                    }

                    // Intracell z
                    for sy in 0..2 {
                        for sx in 0..2 {
                            let sign = if sx == 0 { 1.0 } else { -1.0 };
                            let i = self.site_index(x, y, z, sx, sy, 0);
                            let j = self.site_index(x, y, z, sx, sy, 1);
                            add_disordered_hopping(&mut h, i, j, gamma * sign);
                        }
                    }

                    // Intercell x
                    if x + 1 < self.nx {
                        for sz in 0..2 {
                            for sy in 0..2 {
                                let sign = if sy == 0 { 1.0 } else { -1.0 };
                                let i = self.site_index(x, y, z, 1, sy, sz);
                                let j = self.site_index(x + 1, y, z, 0, sy, sz);
                                add_disordered_hopping(&mut h, i, j, lambda * sign);
                            }
                        }
                    }

                    // Intercell y
                    if y + 1 < self.ny {
                        for sz in 0..2 {
                            let sign = if sz == 0 { 1.0 } else { -1.0 };
                            for sx in 0..2 {
                                let i = self.site_index(x, y, z, sx, 1, sz);
                                let j = self.site_index(x, y + 1, z, sx, 0, sz);
                                add_disordered_hopping(&mut h, i, j, lambda * sign);
                            }
                        }
                    }

                    // Intercell z
                    if z + 1 < self.nz {
                        for sy in 0..2 {
                            for sx in 0..2 {
                                let sign = if sx == 0 { 1.0 } else { -1.0 };
                                let i = self.site_index(x, y, z, sx, sy, 1);
                                let j = self.site_index(x, y, z + 1, sx, sy, 0);
                                add_disordered_hopping(&mut h, i, j, lambda * sign);
                            }
                        }
                    }
                }
            }
        }

        h
    }

    /// Solves the real-space tight-binding Hamiltonian and analyzes corner states.
    pub fn solve(&self) -> OctupoleLatticeResult {
        let h = self.assemble_hamiltonian();
        self.solve_matrix(&h)
    }

    /// Solves the finite lattice with random coupling disorder [-W, W].
    pub fn solve_with_disorder(&self, disorder_w: f64, seed: u64) -> OctupoleLatticeResult {
        let h = self.assemble_hamiltonian_with_disorder(disorder_w, seed);
        self.solve_matrix(&h)
    }

    /// Core eigensolver and 8 corner mode extraction logic.
    fn solve_matrix(&self, h: &[f64]) -> OctupoleLatticeResult {
        let dim = self.total_sites();
        let (eigenvalues, eigenvectors) = jacobi_eigensolver(h, dim);

        let delta_bulk = 2.0 * 3.0_f64.sqrt() * (self.params.lambda - self.params.gamma).abs();
        let is_topological = self.params.gamma.abs() < self.params.lambda.abs();

        // Localization decay length xi = a / ln(lambda / gamma)
        let localization_length_mm = if is_topological && self.params.gamma > 1e-12 {
            let ratio = self.params.lambda / self.params.gamma;
            self.params.a_mm / ratio.ln()
        } else {
            f64::INFINITY
        };

        // Mid-gap energy criterion: |E| < 0.1 * Delta_bulk
        let mid_gap_threshold = (0.1 * delta_bulk).max(1e-4);

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

        // 8 corner unit cell coordinates
        let corners = [
            (CubicCornerId::Corner000, (0, 0, 0)),
            (CubicCornerId::Corner100, (self.nx - 1, 0, 0)),
            (CubicCornerId::Corner010, (0, self.ny - 1, 0)),
            (CubicCornerId::Corner110, (self.nx - 1, self.ny - 1, 0)),
            (CubicCornerId::Corner001, (0, 0, self.nz - 1)),
            (CubicCornerId::Corner101, (self.nx - 1, 0, self.nz - 1)),
            (CubicCornerId::Corner011, (0, self.ny - 1, self.nz - 1)),
            (CubicCornerId::Corner111, (self.nx - 1, self.ny - 1, self.nz - 1)),
        ];

        let mut corner_states = Vec::new();
        let mut combined_corner_intensity = vec![0.0; dim];

        let calc_cell_weight = |intensity: &[f64], cx: usize, cy: usize, cz: usize| -> f64 {
            let mut sum = 0.0;
            for sz in 0..2 {
                for sy in 0..2 {
                    for sx in 0..2 {
                        let idx = self.site_index(cx, cy, cz, sx, sy, sz);
                        sum += intensity[idx];
                    }
                }
            }
            sum
        };

        let calc_all_8_corners = |intensity: &[f64]| -> f64 {
            let mut sum = 0.0;
            for &(_, (cx, cy, cz)) in &corners {
                sum += calc_cell_weight(intensity, cx, cy, cz);
            }
            sum
        };

        if is_topological && candidate_modes.len() >= 8 {
            let active_modes: Vec<usize> = candidate_modes.iter().copied().take(8).collect();

            for &(corner_id, (cx, cy, cz)) in &corners {
                // Construct 8x8 projection matrix M into this corner's 8 sites
                let mut m_mat = vec![0.0; 64];
                for p in 0..8 {
                    for q in 0..8 {
                        let mut sum = 0.0;
                        for sz in 0..2 {
                            for sy in 0..2 {
                                for sx in 0..2 {
                                    let site = self.site_index(cx, cy, cz, sx, sy, sz);
                                    let vp = eigenvectors[site * dim + active_modes[p]];
                                    let vq = eigenvectors[site * dim + active_modes[q]];
                                    sum += vp * vq;
                                }
                            }
                        }
                        m_mat[p * 8 + q] = sum;
                    }
                }

                // Diagonalize 8x8 projection matrix to find optimal linear combination
                let (_proj_evals, proj_evecs) = jacobi_eigensolver(&m_mat, 8);
                // Last eigenvector corresponds to maximum eigenvalue
                let best_vec_col = 7;

                // Synthesize optimal wave function psi in full lattice space
                let mut psi = vec![0.0; dim];
                for k in 0..dim {
                    let mut sum = 0.0;
                    for p in 0..8 {
                        let coeff = proj_evecs[p * 8 + best_vec_col];
                        sum += coeff * eigenvectors[k * dim + active_modes[p]];
                    }
                    psi[k] = sum;
                }

                // Normalize psi
                let mut norm_sq = 0.0;
                for k in 0..dim {
                    norm_sq += psi[k] * psi[k];
                }
                if norm_sq > 1e-12 {
                    let inv_norm = 1.0 / norm_sq.sqrt();
                    for k in 0..dim {
                        psi[k] *= inv_norm;
                    }
                }

                let mut intensity = vec![0.0; dim];
                for k in 0..dim {
                    intensity[k] = psi[k] * psi[k];
                }

                let w_corner = calc_cell_weight(&intensity, cx, cy, cz);
                let all_corners = calc_all_8_corners(&intensity);

                // Compute Rayleigh quotient energy: E = psi^T * H * psi
                let mut energy = 0.0;
                for r in 0..dim {
                    let mut h_row = 0.0;
                    for c in 0..dim {
                        h_row += h[r * dim + c] * psi[c];
                    }
                    energy += psi[r] * h_row;
                }

                corner_states.push(OctupoleCornerState {
                    mode_index: corner_states.len(),
                    energy,
                    corner_id,
                    localization_ratio: w_corner,
                    all_corners_confinement: all_corners,
                    spatial_intensity: intensity,
                });
            }
        }

        // Accumulate combined intensity across all corner states
        if !corner_states.is_empty() {
            let n_cs = corner_states.len() as f64;
            for cs in &corner_states {
                for i in 0..dim {
                    combined_corner_intensity[i] += cs.spatial_intensity[i] / n_cs;
                }
            }
        }

        let energy_confinement_ratio = if !corner_states.is_empty() {
            let sum: f64 = corner_states
                .iter()
                .map(|cs| cs.all_corners_confinement)
                .sum();
            sum / (corner_states.len() as f64)
        } else {
            0.0
        };

        // Quality factor Q estimation: Q >= 1e5 up to 1e7 in topological phase
        let quality_factor = if is_topological {
            let ratio = (self.params.lambda / self.params.gamma.max(1e-6)).clamp(1.0, 50.0);
            let q_est = 2.0e4 * ratio.powf(2.5) * energy_confinement_ratio.clamp(0.0, 1.0);
            q_est.clamp(1.0e5, 1.0e7)
        } else {
            1000.0
        };

        OctupoleLatticeResult {
            nx: self.nx,
            ny: self.ny,
            nz: self.nz,
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

    /// Evaluates defect robustness across multiple disorder strengths W.
    pub fn evaluate_defect_robustness(
        &self,
        disorder_levels: &[f64],
        seed: u64,
    ) -> Vec<DefectRobustnessPoint> {
        let mut results = Vec::with_capacity(disorder_levels.len());
        for (i, &w) in disorder_levels.iter().enumerate() {
            let res = self.solve_with_disorder(w, seed + i as u64);
            let mean_energy = if !res.corner_states.is_empty() {
                let sum: f64 = res.corner_states.iter().map(|cs| cs.energy.abs()).sum();
                sum / res.corner_states.len() as f64
            } else {
                0.0
            };
            results.push(DefectRobustnessPoint {
                disorder_w: w,
                mean_corner_energy_mhz: mean_energy,
                confinement_pct: res.energy_confinement_ratio * 100.0,
                quality_factor: res.quality_factor,
            });
        }
        results
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

    let max_sweeps = 35;
    for _sweep in 0..max_sweeps {
        let mut sm = 0.0;
        for p in 0..n {
            for q in (p + 1)..n {
                sm += a[p * n + q].abs();
            }
        }
        if sm <= 1e-11 {
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

    // Sort eigenvalues and corresponding eigenvector columns ascending
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&i, &j| d[i].partial_cmp(&d[j]).unwrap_or(std::cmp::Ordering::Equal));

    let sorted_d: Vec<f64> = order.iter().map(|&idx| d[idx]).collect();
    let mut sorted_v = vec![0.0; n * n];
    for row in 0..n {
        for (new_col, &old_col) in order.iter().enumerate() {
            sorted_v[row * n + new_col] = v[row * n + old_col];
        }
    }

    (sorted_d, sorted_v)
}
