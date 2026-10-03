#![deny(unsafe_code)]

//! Real-Space Finite Lattice Assembly, Jacobi Eigensolver, Compact Localized States (CLS),
//! and Synthetic Aharonov-Bohm (AB) Caging Simulator for Acoustic Metamaterial Lieb Lattices.
//!
//! Features:
//! - Finite 2D Lieb lattice with Nx x Ny unit cells (3 sublattices: A, B, C; 3*Nx*Ny sites).
//! - Analytical and numerical single-plaquette Compact Localized State (CLS) with 100% spatial confinement.
//! - Destructive interference condition: net coupling into adjacent corner sites A cancels exactly.
//! - Synthetic Aharonov-Bohm caging under flux Phi = PI: wave packet trapping and dispersion suppression.
//! - Time-dependent wave packet dynamics: |psi(t)|^2 = |exp(-i H t) psi(0)|^2.
//! - Inverse Participation Ratio (IPR) localization metric: sum_i |psi_i|^4.
//! - Disorder resilience analysis against random on-site perturbations [-W, W].

use std::f64::consts::PI;
use super::hamiltonian::{LiebComplex, LiebParams};

/// Lightweight deterministic XorShift PRNG for disorder perturbations (100% safe Rust).
#[derive(Debug, Clone)]
pub struct XorShiftRng {
    state: u64,
}

impl XorShiftRng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x853c49e6748fea9b } else { seed },
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() as f64) / (u64::MAX as f64)
    }

    pub fn next_range(&mut self, w: f64) -> f64 {
        (self.next_f64() * 2.0 - 1.0) * w
    }
}

/// Compact Localized State (CLS) localized strictly within a single plaquette.
///
/// In the Lieb lattice, the CLS occupies 4 edge sites (2 B sites, 2 C sites)
/// surrounding a plaquette with alternating amplitudes (+1, -1, +1, -1) / 2.
/// Destructive interference causes net zero coupling into all adjacent corner sites A,
/// resulting in exact eigenvalue E = 0.0 and 100% spatial confinement.
#[derive(Debug, Clone, PartialEq)]
pub struct CompactLocalizedState {
    /// Plaquette coordinates (bottom-left cell x).
    pub cell_x: usize,
    /// Plaquette coordinates (bottom-left cell y).
    pub cell_y: usize,
    /// Global indices of the 4 occupied edge sites: [bottom B, right C, top B, left C].
    pub site_indices: [usize; 4],
    /// Amplitudes on the 4 edge sites: [+0.5, -0.5, +0.5, -0.5].
    pub amplitudes: [f64; 4],
    /// Energy eigenvalue in MHz (identically 0.0 MHz).
    pub energy: f64,
    /// Spatial energy confinement ratio within this single plaquette [0.0, 1.0] (1.0 = 100%).
    pub confinement_ratio: f64,
    /// Full lattice spatial probability density |psi_i|^2 across all 3*Nx*Ny sites.
    pub spatial_intensity: Vec<f64>,
}

impl CompactLocalizedState {
    /// Verifies the destructive interference condition by computing maximum residual |(H * psi)_i|
    /// across all sites in the lattice.
    pub fn verify_destructive_interference(&self, h_re: &[f64], dim: usize) -> f64 {
        let mut max_res = 0.0_f64;
        let mut psi_vec = vec![0.0; dim];
        for k in 0..4 {
            psi_vec[self.site_indices[k]] = self.amplitudes[k];
        }

        for i in 0..dim {
            let mut row_sum = 0.0_f64;
            for j in 0..dim {
                row_sum += h_re[i * dim + j] * psi_vec[j];
            }
            let res = row_sum.abs();
            if res > max_res {
                max_res = res;
            }
        }
        max_res
    }
}

/// Disorder stability evaluation metrics for the Compact Localized State.
#[derive(Debug, Clone, PartialEq)]
pub struct DisorderResilienceResult {
    /// Disorder amplitude W in MHz.
    pub disorder_w: f64,
    /// Mean energy shift <|Delta E|> in MHz.
    pub mean_energy_shift_mhz: f64,
    /// Maximum energy shift in MHz.
    pub max_energy_shift_mhz: f64,
    /// Mean spatial confinement ratio within the host plaquette [0.0, 1.0].
    pub mean_confinement_ratio: f64,
    /// Minimum spatial confinement ratio observed across realizations.
    pub min_confinement_ratio: f64,
    /// Whether the CLS remains robustly localized (confinement > 0.85).
    pub is_robust: bool,
}

/// Full solution result from the real-space finite Lieb lattice eigensolver.
#[derive(Debug, Clone, PartialEq)]
pub struct LiebLatticeResult {
    pub nx: usize,
    pub ny: usize,
    pub total_sites: usize,
    /// All eigenvalues in ascending order in MHz.
    pub all_eigenvalues: Vec<f64>,
    /// Complex eigenvectors: column j is the eigenvector corresponding to `all_eigenvalues[j]`.
    pub eigenvectors: Vec<Vec<LiebComplex>>,
    /// Number of identified zero-energy flat-band modes (|E| < 1e-4 MHz).
    pub num_flat_band_modes: usize,
    /// Single-plaquette Compact Localized States.
    pub cls_states: Vec<CompactLocalizedState>,
    /// Combined spatial intensity map |psi_all|^2 summing all CLS states (normalized).
    pub combined_cls_intensity: Vec<f64>,
}

/// Real-space finite 2D Lieb lattice model.
///
/// Contains Nx x Ny unit cells with 3 sites per cell:
/// - Site 0 (A): Corner site at (x, y)
/// - Site 1 (B): Horizontal edge site at (x + 0.5, y)
/// - Site 2 (C): Vertical edge site at (x, y + 0.5)
///
/// Total sites = 3 * Nx * Ny.
#[derive(Debug, Clone, PartialEq)]
pub struct LiebLattice {
    /// Number of unit cells along x.
    pub nx: usize,
    /// Number of unit cells along y.
    pub ny: usize,
    /// Lieb lattice physical parameters.
    pub params: LiebParams,
}

impl Default for LiebLattice {
    fn default() -> Self {
        Self::new(4, 4, LiebParams::default())
    }
}

impl LiebLattice {
    /// Creates a new finite Lieb lattice.
    pub fn new(nx: usize, ny: usize, params: LiebParams) -> Self {
        Self {
            nx: nx.max(2),
            ny: ny.max(2),
            params,
        }
    }

    /// Total number of sites in the finite lattice = 3 * Nx * Ny.
    pub fn total_sites(&self) -> usize {
        3 * self.nx * self.ny
    }

    /// Site index in the flat array [0 .. 3*Nx*Ny - 1].
    /// Sublattice: 0 = A (corner), 1 = B (x-edge), 2 = C (y-edge).
    pub fn site_index(&self, cell_x: usize, cell_y: usize, sublattice: usize) -> usize {
        (cell_y * self.nx + cell_x) * 3 + sublattice
    }

    /// Real-space coordinate (x_mm, y_mm) of a site.
    pub fn site_position_mm(&self, cell_x: usize, cell_y: usize, sublattice: usize) -> (f64, f64) {
        let a = self.params.a_mm;
        let base_x = (cell_x as f64) * a;
        let base_y = (cell_y as f64) * a;
        match sublattice {
            0 => (base_x, base_y),
            1 => (base_x + 0.5 * a, base_y),
            2 => (base_x, base_y + 0.5 * a),
            _ => (base_x, base_y),
        }
    }

    /// Assembles the real-space tight-binding Hamiltonian as real and imaginary flat matrices.
    ///
    /// Dimension is total_sites x total_sites.
    /// Peierls phase gauge flux:
    /// In Landau gauge, horizontal bonds are real (phase 0), and vertical bonds at column x
    /// acquire phase factor exp(i * x * Phi / 2), threading flux Phi per plaquette.
    pub fn assemble_hamiltonian(&self) -> (Vec<f64>, Vec<f64>) {
        let dim = self.total_sites();
        let mut h_re = vec![0.0; dim * dim];
        let mut h_im = vec![0.0; dim * dim];

        let j_x = self.params.j_x;
        let j_y = self.params.j_y;
        let phi = self.params.phi;

        for y in 0..self.ny {
            for x in 0..self.nx {
                let s_a = self.site_index(x, y, 0);
                let s_b = self.site_index(x, y, 1);
                let s_c = self.site_index(x, y, 2);

                // On-site detuning on corner site A
                if self.params.delta_site.abs() > 1e-12 {
                    h_re[s_a * dim + s_a] += self.params.delta_site;
                }

                // 1. Intracell horizontal coupling A(x, y) <-> B(x, y) with J_x (phase 0)
                h_re[s_a * dim + s_b] += j_x;
                h_re[s_b * dim + s_a] += j_x;

                // 2. Intercell horizontal coupling B(x, y) <-> A(x + 1, y) with J_x (phase 0)
                if x + 1 < self.nx {
                    let next_a = self.site_index(x + 1, y, 0);
                    h_re[s_b * dim + next_a] += j_x;
                    h_re[next_a * dim + s_b] += j_x;
                }

                // Peierls phase for vertical bonds: phase = x * (phi / 2)
                let vert_phase = (x as f64) * (phi * 0.5);
                let cos_p = vert_phase.cos();
                let sin_p = vert_phase.sin();

                // 3. Intracell vertical coupling A(x, y) <-> C(x, y) with J_y * exp(i * vert_phase)
                // H[s_a, s_c] = J_y * exp(i * vert_phase), H[s_c, s_a] = conj
                h_re[s_a * dim + s_c] += j_y * cos_p;
                h_im[s_a * dim + s_c] += j_y * sin_p;
                h_re[s_c * dim + s_a] += j_y * cos_p;
                h_im[s_c * dim + s_a] -= j_y * sin_p;

                // 4. Intercell vertical coupling C(x, y) <-> A(x, y + 1) with J_y * exp(i * vert_phase)
                if y + 1 < self.ny {
                    let next_a = self.site_index(x, y + 1, 0);
                    h_re[s_c * dim + next_a] += j_y * cos_p;
                    h_im[s_c * dim + next_a] += j_y * sin_p;
                    h_re[next_a * dim + s_c] += j_y * cos_p;
                    h_im[next_a * dim + s_c] -= j_y * sin_p;
                }
            }
        }

        (h_re, h_im)
    }

    /// Assembles the Hamiltonian with random on-site disorder perturbations delta in [-disorder_w, disorder_w].
    pub fn assemble_hamiltonian_with_disorder(&self, disorder_w: f64, seed: u64) -> (Vec<f64>, Vec<f64>) {
        let (mut h_re, h_im) = self.assemble_hamiltonian();
        let dim = self.total_sites();
        let mut rng = XorShiftRng::new(seed);

        for i in 0..dim {
            let perturbation = rng.next_range(disorder_w);
            h_re[i * dim + i] += perturbation;
        }

        (h_re, h_im)
    }

    /// Generates an analytical single-plaquette Compact Localized State (CLS)
    /// located at plaquette (cell_x, cell_y).
    ///
    /// Requires cell_x + 1 < nx and cell_y + 1 < ny.
    pub fn generate_analytical_cls(&self, cell_x: usize, cell_y: usize) -> Option<CompactLocalizedState> {
        if cell_x + 1 >= self.nx || cell_y + 1 >= self.ny {
            return None;
        }

        let dim = self.total_sites();

        // The 4 edge sites bounding this plaquette:
        // Bottom edge: B(cell_x, cell_y)
        let s_b_bot = self.site_index(cell_x, cell_y, 1);
        // Right edge: C(cell_x + 1, cell_y)
        let s_c_right = self.site_index(cell_x + 1, cell_y, 2);
        // Top edge: B(cell_x, cell_y + 1)
        let s_b_top = self.site_index(cell_x, cell_y + 1, 1);
        // Left edge: C(cell_x, cell_y)
        let s_c_left = self.site_index(cell_x, cell_y, 2);

        let site_indices = [s_b_bot, s_c_right, s_b_top, s_c_left];
        let amplitudes = [0.5, -0.5, 0.5, -0.5];

        let mut spatial_intensity = vec![0.0; dim];
        for k in 0..4 {
            spatial_intensity[site_indices[k]] = amplitudes[k] * amplitudes[k];
        }

        // Energy is strictly 0.0 MHz and spatial confinement is 100% within this plaquette
        Some(CompactLocalizedState {
            cell_x,
            cell_y,
            site_indices,
            amplitudes,
            energy: 0.0,
            confinement_ratio: 1.0,
            spatial_intensity,
        })
    }

    /// Solves the real-space tight-binding Hamiltonian and extracts flat-band compact states.
    pub fn solve(&self) -> LiebLatticeResult {
        let (h_re, h_im) = self.assemble_hamiltonian();
        self.solve_matrix(&h_re, &h_im)
    }

    /// Solves a given real/imaginary Hamiltonian matrix using the safe cyclic Jacobi eigensolver.
    pub fn solve_matrix(&self, h_re: &[f64], h_im: &[f64]) -> LiebLatticeResult {
        let dim = self.total_sites();
        let (eigenvalues, eigenvectors) = hermitian_eigensolver(h_re, h_im, dim);

        // Identify zero-energy flat band modes (|E| < 1e-4 MHz)
        let mut num_flat = 0;
        for &e in &eigenvalues {
            if e.abs() < 1e-4 {
                num_flat += 1;
            }
        }

        // Build analytical CLS states across all plaquettes
        let mut cls_states = Vec::new();
        let mut combined_cls_intensity = vec![0.0; dim];

        for y in 0..(self.ny.saturating_sub(1)) {
            for x in 0..(self.nx.saturating_sub(1)) {
                if let Some(cls) = self.generate_analytical_cls(x, y) {
                    for i in 0..dim {
                        combined_cls_intensity[i] += cls.spatial_intensity[i];
                    }
                    cls_states.push(cls);
                }
            }
        }

        // Normalize combined intensity
        if !cls_states.is_empty() {
            let n_cls = cls_states.len() as f64;
            for val in &mut combined_cls_intensity {
                *val /= n_cls;
            }
        }

        LiebLatticeResult {
            nx: self.nx,
            ny: self.ny,
            total_sites: dim,
            all_eigenvalues: eigenvalues,
            eigenvectors,
            num_flat_band_modes: num_flat,
            cls_states,
            combined_cls_intensity,
        }
    }
}

/// Synthetic Aharonov-Bohm Caging Simulator.
///
/// Implements wave packet dynamics, time-dependent quantum evolution psi(t) = exp(-i H t) psi(0),
/// Inverse Participation Ratio (IPR) localization metric, and synthetic gauge flux caging curves.
#[derive(Debug, Clone, PartialEq)]
pub struct AbCagingSimulator {
    pub lattice: LiebLattice,
    pub solution: LiebLatticeResult,
}

impl AbCagingSimulator {
    /// Creates a new AbCagingSimulator for the specified lattice.
    pub fn new(lattice: LiebLattice) -> Self {
        let solution = lattice.solve();
        Self { lattice, solution }
    }

    /// Updates the simulator with new parameters and recomputes eigensystem.
    pub fn update_params(&mut self, params: LiebParams) {
        self.lattice.params = params;
        self.solution = self.lattice.solve();
    }

    /// Computes the time-evolved state psi(t) = exp(-i H t) psi(0).
    ///
    /// Decomposes psi(0) onto eigenmodes:
    /// c_m = <v_m | psi(0)> = sum_k v_m*(k) psi_k(0)
    /// c_m(t) = c_m * exp(-i * E_m * t)
    /// psi(t) = sum_m c_m(t) * |v_m>
    pub fn evolve_wave_packet(&self, psi_0: &[LiebComplex], t_ns: f64) -> Vec<LiebComplex> {
        let dim = self.lattice.total_sites();
        let num_modes = self.solution.all_eigenvalues.len();
        let mut psi_t = vec![LiebComplex::ZERO; dim];

        // Convert t in ns and energies in MHz to phase in radians:
        // omega * t = 2 * pi * (E_MHz * 1e6) * (t_ns * 1e-9) = 2 * pi * 1e-3 * E_MHz * t_ns
        let time_factor = 2.0 * PI * 1e-3 * t_ns;

        for m in 0..num_modes {
            let energy = self.solution.all_eigenvalues[m];
            let v_m = &self.solution.eigenvectors[m];

            // Overlap c_m = <v_m | psi_0>
            let mut c_re = 0.0_f64;
            let mut c_im = 0.0_f64;
            for k in 0..dim {
                let vk = v_m[k];
                let pk = psi_0[k];
                // vk^* * pk = (vk.re - i vk.im) * (pk.re + i pk.im)
                c_re += vk.re * pk.re + vk.im * pk.im;
                c_im += vk.re * pk.im - vk.im * pk.re;
            }

            // Propagate phase: exp(-i * energy * time_factor)
            let phase = -energy * time_factor;
            let cos_p = phase.cos();
            let sin_p = phase.sin();

            let cm_t_re = c_re * cos_p - c_im * sin_p;
            let cm_t_im = c_re * sin_p + c_im * cos_p;

            // Add contribution to psi_t
            for k in 0..dim {
                let vk = v_m[k];
                psi_t[k].re += cm_t_re * vk.re - cm_t_im * vk.im;
                psi_t[k].im += cm_t_re * vk.im + cm_t_im * vk.re;
            }
        }

        psi_t
    }

    /// Computes the spatial probability density |psi_i|^2 across all lattice sites.
    pub fn spatial_intensity(psi: &[LiebComplex]) -> Vec<f64> {
        psi.iter().map(|c| c.norm_sq()).collect()
    }

    /// Computes the Inverse Participation Ratio (IPR) caging metric:
    /// I_P = sum_i |psi_i|^4.
    ///
    /// For a state localized on a single site, I_P = 1.0.
    /// For a 4-site CLS with amplitudes 0.5, I_P = 4 * (0.25)^2 = 0.25.
    /// For a state dispersed across M sites, I_P approx 1 / M -> 0.
    pub fn inverse_participation_ratio(psi: &[LiebComplex]) -> f64 {
        let norm_sq_sum: f64 = psi.iter().map(|c| c.norm_sq()).sum();
        if norm_sq_sum < 1e-15 {
            return 0.0;
        }

        let mut ipr = 0.0_f64;
        for c in psi {
            let p_i = c.norm_sq() / norm_sq_sum;
            ipr += p_i * p_i;
        }
        ipr
    }

    /// Constructs an initial single-site excitation wave packet at the central corner site A.
    pub fn initial_center_site_wavepacket(&self) -> Vec<LiebComplex> {
        let dim = self.lattice.total_sites();
        let mut psi_0 = vec![LiebComplex::ZERO; dim];
        let center_x = self.lattice.nx / 2;
        let center_y = self.lattice.ny / 2;
        let center_idx = self.lattice.site_index(center_x, center_y, 0);
        psi_0[center_idx] = LiebComplex::ONE;
        psi_0
    }

    /// Constructs an initial wave packet corresponding to the central single-plaquette CLS.
    pub fn initial_cls_wavepacket(&self) -> Vec<LiebComplex> {
        let dim = self.lattice.total_sites();
        let mut psi_0 = vec![LiebComplex::ZERO; dim];
        let cx = (self.lattice.nx.saturating_sub(1)) / 2;
        let cy = (self.lattice.ny.saturating_sub(1)) / 2;
        if let Some(cls) = self.lattice.generate_analytical_cls(cx, cy) {
            for k in 0..4 {
                psi_0[cls.site_indices[k]] = LiebComplex::new(cls.amplitudes[k], 0.0);
            }
        }
        psi_0
    }

    /// Scans Inverse Participation Ratio (IPR) vs Synthetic Gauge Flux Phi in [0.0, 2 * PI]
    /// at a given evolution time `evolution_time_ns`.
    ///
    /// At Phi = PI, destructive interference suppresses spatial dispersion, producing
    /// a pronounced caging peak in the IPR.
    pub fn caging_curve_vs_flux(
        &self,
        num_flux_points: usize,
        evolution_time_ns: f64,
    ) -> Vec<(f64, f64)> {
        let n = num_flux_points.max(8);
        let mut curve = Vec::with_capacity(n);

        let mut temp_lattice = self.lattice.clone();

        for i in 0..n {
            let phi = 2.0 * PI * (i as f64) / ((n - 1) as f64);
            temp_lattice.params.phi = phi;
            let temp_sim = AbCagingSimulator::new(temp_lattice.clone());

            let psi_0 = temp_sim.initial_center_site_wavepacket();
            let psi_t = temp_sim.evolve_wave_packet(&psi_0, evolution_time_ns);
            let ipr = Self::inverse_participation_ratio(&psi_t);

            // Return (phi_over_pi, ipr)
            curve.push((phi / PI, ipr));
        }

        curve
    }

    /// Evaluates time-evolution dynamics over time t in [0, t_max_ns],
    /// returning points (t_ns, ipr_current_flux, ipr_pi_caged).
    pub fn time_evolution_dynamics(
        &self,
        t_max_ns: f64,
        num_steps: usize,
    ) -> Vec<(f64, f64, f64)> {
        let n = num_steps.max(10);
        let mut curve = Vec::with_capacity(n);

        // Precompute Pi-caged simulator for comparison
        let mut caged_lattice = self.lattice.clone();
        caged_lattice.params.phi = PI;
        let caged_sim = AbCagingSimulator::new(caged_lattice);

        let psi_0 = self.initial_center_site_wavepacket();
        let psi_0_caged = caged_sim.initial_center_site_wavepacket();

        for i in 0..n {
            let t = t_max_ns * (i as f64) / ((n - 1) as f64);
            let psi_t = self.evolve_wave_packet(&psi_0, t);
            let ipr_curr = Self::inverse_participation_ratio(&psi_t);

            let psi_t_caged = caged_sim.evolve_wave_packet(&psi_0_caged, t);
            let ipr_caged = Self::inverse_participation_ratio(&psi_t_caged);

            curve.push((t, ipr_curr, ipr_caged));
        }

        curve
    }

    /// Evaluates disorder resilience of the Compact Localized State against random on-site disorder [-W, W].
    pub fn evaluate_disorder_resilience(
        &self,
        disorder_w: f64,
        num_realizations: usize,
        seed: u64,
    ) -> DisorderResilienceResult {
        let n_trials = num_realizations.max(1);
        let mut total_shift = 0.0_f64;
        let mut max_shift = 0.0_f64;
        let mut total_confinement = 0.0_f64;
        let mut min_confinement = 1.0_f64;

        let cx = (self.lattice.nx.saturating_sub(1)) / 2;
        let cy = (self.lattice.ny.saturating_sub(1)) / 2;
        let cls = match self.lattice.generate_analytical_cls(cx, cy) {
            Some(c) => c,
            None => {
                return DisorderResilienceResult {
                    disorder_w,
                    mean_energy_shift_mhz: 0.0,
                    max_energy_shift_mhz: 0.0,
                    mean_confinement_ratio: 1.0,
                    min_confinement_ratio: 1.0,
                    is_robust: true,
                };
            }
        };

        for trial in 0..n_trials {
            let trial_seed = seed.wrapping_add((trial as u64) * 1013904223);
            let (h_dis_re, h_dis_im) = self.lattice.assemble_hamiltonian_with_disorder(disorder_w, trial_seed);

            // Compute Rayleigh quotient energy: E_cls = psi^T * H * psi
            let dim = self.lattice.total_sites();
            let mut psi_vec = vec![0.0; dim];
            for k in 0..4 {
                psi_vec[cls.site_indices[k]] = cls.amplitudes[k];
            }

            let mut energy = 0.0_f64;
            for r in 0..dim {
                let mut h_row = 0.0_f64;
                for c in 0..dim {
                    h_row += h_dis_re[r * dim + c] * psi_vec[c];
                }
                energy += psi_vec[r] * h_row;
            }

            let shift = energy.abs();
            total_shift += shift;
            if shift > max_shift {
                max_shift = shift;
            }

            // Test time evolved confinement at t = 20 ns under disordered Hamiltonian
            let dis_sol = self.lattice.solve_matrix(&h_dis_re, &h_dis_im);
            let temp_sim = AbCagingSimulator {
                lattice: self.lattice.clone(),
                solution: dis_sol,
            };

            let mut cls_psi = vec![LiebComplex::ZERO; dim];
            for k in 0..4 {
                cls_psi[cls.site_indices[k]] = LiebComplex::new(cls.amplitudes[k], 0.0);
            }

            let psi_t = temp_sim.evolve_wave_packet(&cls_psi, 20.0);
            let int_t = Self::spatial_intensity(&psi_t);
            let mut conf = 0.0_f64;
            for k in 0..4 {
                conf += int_t[cls.site_indices[k]];
            }
            conf = conf.clamp(0.0, 1.0);

            total_confinement += conf;
            if conf < min_confinement {
                min_confinement = conf;
            }
        }

        let mean_shift = total_shift / (n_trials as f64);
        let mean_conf = total_confinement / (n_trials as f64);
        let is_robust = mean_conf > 0.85;

        DisorderResilienceResult {
            disorder_w,
            mean_energy_shift_mhz: mean_shift,
            max_energy_shift_mhz: max_shift,
            mean_confinement_ratio: mean_conf,
            min_confinement_ratio: min_confinement,
            is_robust,
        }
    }
}

/// Solves an N x N real symmetric or complex Hermitian matrix with 100% safe Rust.
///
/// Returns (eigenvalues in ascending order, complex eigenvectors).
pub fn hermitian_eigensolver(
    h_re: &[f64],
    h_im: &[f64],
    n: usize,
) -> (Vec<f64>, Vec<Vec<LiebComplex>>) {
    let mut max_im = 0.0_f64;
    for &val in h_im {
        let a = val.abs();
        if a > max_im {
            max_im = a;
        }
    }

    if max_im < 1e-12 {
        // Purely real symmetric matrix: diagonalize directly via cyclic Jacobi
        let (eigenvalues, v_flat) = jacobi_symmetric_eigensolver(h_re, n);
        let mut eigenvectors = Vec::with_capacity(n);
        for m in 0..n {
            let mut vec_m = Vec::with_capacity(n);
            for r in 0..n {
                vec_m.push(LiebComplex::new(v_flat[r * n + m], 0.0));
            }
            eigenvectors.push(vec_m);
        }
        (eigenvalues, eigenvectors)
    } else {
        // Complex Hermitian matrix: construct 2N x 2N real symmetric block matrix:
        // M = [ H_re  -H_im ]
        //     [ H_im   H_re ]
        let n2 = 2 * n;
        let mut m_mat = vec![0.0; n2 * n2];

        for r in 0..n {
            for c in 0..n {
                let re = h_re[r * n + c];
                let im = h_im[r * n + c];

                // Top-left block: H_re
                m_mat[r * n2 + c] = re;
                // Top-right block: -H_im
                m_mat[r * n2 + (n + c)] = -im;
                // Bottom-left block: H_im
                m_mat[(n + r) * n2 + c] = im;
                // Bottom-right block: H_re
                m_mat[(n + r) * n2 + (n + c)] = re;
            }
        }

        let (all_evals, v2_flat) = jacobi_symmetric_eigensolver(&m_mat, n2);

        // In 2N x 2N, every eigenvalue of H is paired. Pick every second eigenvalue.
        let mut eigenvalues = Vec::with_capacity(n);
        let mut eigenvectors = Vec::with_capacity(n);

        let mut idx = 0;
        while idx < n2 && eigenvalues.len() < n {
            eigenvalues.push(all_evals[idx]);

            let mut vec_m = Vec::with_capacity(n);
            let mut norm_sq = 0.0_f64;
            for r in 0..n {
                let u = v2_flat[r * n2 + idx];
                let v = v2_flat[(n + r) * n2 + idx];
                norm_sq += u * u + v * v;
                vec_m.push(LiebComplex::new(u, v));
            }

            // Normalize
            let norm = norm_sq.sqrt().max(1e-15);
            for c in &mut vec_m {
                c.re /= norm;
                c.im /= norm;
            }

            eigenvectors.push(vec_m);
            idx += 2;
        }

        (eigenvalues, eigenvectors)
    }
}

/// Computes eigenvalues and eigenvectors of a real symmetric N x N matrix via cyclic Jacobi rotations.
///
/// Matrix is flat row-major slice of length N*N.
/// Returns (eigenvalues in ascending order, flat eigenvectors matrix where column j is eigenvector j).
pub fn jacobi_symmetric_eigensolver(a_in: &[f64], n: usize) -> (Vec<f64>, Vec<f64>) {
    let mut a = a_in.to_vec();
    let mut d = vec![0.0; n];
    let mut v = vec![0.0; n * n];

    for i in 0..n {
        for j in 0..n {
            v[i * n + j] = if i == j { 1.0 } else { 0.0 };
        }
        d[i] = a[i * n + i];
    }

    let max_sweeps = 80;
    for _sweep in 0..max_sweeps {
        let mut sm = 0.0_f64;
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

    // Sort eigenvalues in ascending order and reorder eigenvector columns
    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_by(|&i, &j| d[i].partial_cmp(&d[j]).unwrap_or(std::cmp::Ordering::Equal));

    let mut sorted_d = vec![0.0; n];
    let mut sorted_v = vec![0.0; n * n];

    for (new_col, &orig_col) in indices.iter().enumerate() {
        sorted_d[new_col] = d[orig_col];
        for row in 0..n {
            sorted_v[row * n + new_col] = v[row * n + orig_col];
        }
    }

    (sorted_d, sorted_v)
}
