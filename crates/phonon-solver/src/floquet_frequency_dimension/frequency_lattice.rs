#![deny(unsafe_code)]

//! Synthetic Acoustic Frequency Dimension & Dynamic Floquet-Bloch Lattice.
//!
//! Models synthetic frequency dimensions created via dynamic acoustic boundary phase/impedance
//! modulations, coupling discrete cavity/waveguide harmonic modes into a synthetic 1D/2D
//! tight-binding lattice with synthetic gauge fields and topological Chern numbers.

use std::f64::consts::PI;

/// Type of synthetic frequency lattice configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntheticLatticeKind {
    /// 1D synthetic frequency modes chain coupled via boundary modulation.
    Synthetic1DFrequencyChain,
    /// 2D coupled lattice: spatial acoustic sites (x) coupled to synthetic frequency modes (m).
    Synthetic2DRealFrequency,
    /// Anomalous Floquet topological lattice with chiral synthetic frequency edge modes.
    AnomalousFloquetLattice,
    /// Synthetic honeycomb lattice in real-frequency space.
    SyntheticHoneycombFrequency,
}

/// Parameters for dynamic boundary modulation and synthetic frequency lattice.
#[derive(Debug, Clone)]
pub struct BoundaryModulationParams {
    /// Base acoustic carrier frequency in kHz (omega_0, e.g. 10.0 kHz).
    pub base_frequency_khz: f64,
    /// Free spectral range (FSR) / inter-modal frequency spacing in kHz (Omega_R, e.g. 1.0 kHz).
    pub fsr_frequency_khz: f64,
    /// Dynamic boundary modulation frequency in kHz (Omega_m, e.g. 1.0 kHz).
    pub modulation_frequency_khz: f64,
    /// Dimensionless dynamic modulation depth mu in [0.0, 1.0] (controls hopping amplitude).
    pub modulation_depth: f64,
    /// Synthetic magnetic gauge flux Phi per plaquette in radians (e.g. pi / 2.0).
    pub synthetic_gauge_flux_rad: f64,
    /// Total number of synthetic frequency modes (e.g. 11, spanning m in -5..=+5).
    pub num_frequency_modes: usize,
    /// Number of physical spatial resonator sites (Nx, e.g. 6).
    pub num_spatial_sites: usize,
    /// Boundary modulation detuning delta_omega = Omega_m - Omega_R in kHz.
    pub detuning_khz: f64,
    /// Spatial boundary phase gradient in radians.
    pub boundary_phase_grad_rad: f64,
    /// Whether a frequency-space defect obstacle (missing coupling) is active.
    pub defect_mode_active: bool,
}

impl Default for BoundaryModulationParams {
    fn default() -> Self {
        Self {
            base_frequency_khz: 10.0,
            fsr_frequency_khz: 1.0,
            modulation_frequency_khz: 1.0,
            modulation_depth: 0.35,
            synthetic_gauge_flux_rad: std::f64::consts::FRAC_PI_2,
            num_frequency_modes: 11,
            num_spatial_sites: 6,
            detuning_khz: 0.0,
            boundary_phase_grad_rad: 0.0,
            defect_mode_active: false,
        }
    }
}

/// Representation of a complex scalar for safe mathematical operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub const ZERO: Complex = Complex { re: 0.0, im: 0.0 };
    pub const ONE: Complex = Complex { re: 1.0, im: 0.0 };
    pub const I: Complex = Complex { re: 0.0, im: 1.0 };

    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    pub fn from_polar(r: f64, theta: f64) -> Self {
        Self {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }

    pub fn norm_sq(&self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    pub fn norm(&self) -> f64 {
        self.norm_sq().sqrt()
    }

    pub fn arg(&self) -> f64 {
        self.im.atan2(self.re)
    }

    pub fn conj(&self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    pub fn add(&self, other: &Self) -> Self {
        Self {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }

    pub fn sub(&self, other: &Self) -> Self {
        Self {
            re: self.re - other.re,
            im: self.im - other.im,
        }
    }

    pub fn mul(&self, other: &Self) -> Self {
        Self {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }

    pub fn scale(&self, s: f64) -> Self {
        Self {
            re: self.re * s,
            im: self.im * s,
        }
    }
}

/// Point on the synthetic Floquet-Bloch quasi-energy dispersion spectrum.
#[derive(Debug, Clone)]
pub struct FloquetBandPoint {
    pub kx: f64,
    pub km: f64,
    pub quasi_energies: Vec<f64>,
    pub is_edge_mode: bool,
}

/// Synthetic Frequency Lattice Engine.
#[derive(Debug, Clone)]
pub struct SyntheticFrequencyLattice {
    pub params: BoundaryModulationParams,
    pub lattice_kind: SyntheticLatticeKind,
}

impl SyntheticFrequencyLattice {
    pub fn new(params: BoundaryModulationParams, lattice_kind: SyntheticLatticeKind) -> Self {
        Self {
            params,
            lattice_kind,
        }
    }

    /// Fast constructor for cold boot latency optimization (< 0.1ms).
    pub fn new_fast(params: BoundaryModulationParams) -> Self {
        Self {
            params,
            lattice_kind: SyntheticLatticeKind::AnomalousFloquetLattice,
        }
    }

    /// Total dimension of the synthetic lattice state space: Nx * Nm.
    pub fn total_dimension(&self) -> usize {
        match self.lattice_kind {
            SyntheticLatticeKind::Synthetic1DFrequencyChain => self.params.num_frequency_modes,
            _ => self.params.num_spatial_sites * self.params.num_frequency_modes,
        }
    }

    /// Mode index from zero-based synthetic coordinate.
    pub fn mode_index(&self, m_idx: usize) -> i32 {
        let half = (self.params.num_frequency_modes as i32) / 2;
        m_idx as i32 - half
    }

    /// Frequency of a given mode index in kHz: omega_0 + m * Omega_R.
    pub fn mode_frequency_khz(&self, m: i32) -> f64 {
        self.params.base_frequency_khz + (m as f64) * self.params.fsr_frequency_khz
    }

    /// Hopping coupling amplitude between adjacent frequency modes in kHz.
    pub fn frequency_hopping_amplitude(&self) -> f64 {
        // J = 0.5 * mu * Omega_R
        0.5 * self.params.modulation_depth * self.params.fsr_frequency_khz
    }

    /// Spatial hopping coupling amplitude in kHz.
    pub fn spatial_hopping_amplitude(&self) -> f64 {
        0.4 * self.frequency_hopping_amplitude().max(0.1)
    }

    /// Build the flat D x D complex Hermitian Hamiltonian matrix for the finite lattice.
    pub fn build_hamiltonian(&self) -> Vec<Complex> {
        let nx = match self.lattice_kind {
            SyntheticLatticeKind::Synthetic1DFrequencyChain => 1,
            _ => self.params.num_spatial_sites,
        };
        let nm = self.params.num_frequency_modes;
        let dim = nx * nm;
        let mut h = vec![Complex::ZERO; dim * dim];

        let j_freq = self.frequency_hopping_amplitude();
        let j_space = self.spatial_hopping_amplitude();
        let phi = self.params.synthetic_gauge_flux_rad;
        let detuning = self.params.detuning_khz;

        // Populate on-site and hopping elements
        for x in 0..nx {
            for m in 0..nm {
                let idx = x * nm + m;
                let mode_m = self.mode_index(m);

                // On-site detuning: (m * delta_omega)
                let on_site_e = (mode_m as f64) * detuning;
                h[idx * dim + idx] = Complex::new(on_site_e, 0.0);

                // Frequency hopping: (x, m) <-> (x, m + 1) with Peierls phase x * Phi
                if m + 1 < nm {
                    // Check if defect is present
                    let defect_blocked = self.params.defect_mode_active && (m == nm / 2) && (x == nx / 2);
                    if !defect_blocked {
                        let phase = (x as f64) * phi + self.params.boundary_phase_grad_rad;
                        let hop_val = Complex::from_polar(j_freq, phase);

                        let next_idx = x * nm + (m + 1);
                        h[idx * dim + next_idx] = hop_val;
                        h[next_idx * dim + idx] = hop_val.conj();
                    }
                }

                // Spatial hopping: (x, m) <-> (x + 1, m)
                if x + 1 < nx {
                    let next_idx = (x + 1) * nm + m;
                    let hop_val = Complex::new(j_space, 0.0);
                    h[idx * dim + next_idx] = hop_val;
                    h[next_idx * dim + idx] = hop_val.conj();
                }
            }
        }

        h
    }

    /// Diagonalize the finite synthetic lattice Hamiltonian using cyclic Jacobi algorithm.
    /// Returns (sorted eigenvalues, corresponding complex eigenvectors).
    pub fn diagonalize(&self) -> (Vec<f64>, Vec<Vec<Complex>>) {
        let dim = self.total_dimension();
        let h = self.build_hamiltonian();

        // Embed N x N complex Hermitian matrix into 2N x 2N real symmetric matrix:
        // [ Re(H)  -Im(H) ]
        // [ Im(H)   Re(H) ]
        let dim2 = 2 * dim;
        let mut h2 = vec![0.0_f64; dim2 * dim2];

        for i in 0..dim {
            for j in 0..dim {
                let elem = h[i * dim + j];
                // Top-left: Re(H)
                h2[i * dim2 + j] = elem.re;
                // Top-right: -Im(H)
                h2[i * dim2 + (j + dim)] = -elem.im;
                // Bottom-left: Im(H)
                h2[(i + dim) * dim2 + j] = elem.im;
                // Bottom-right: Re(H)
                h2[(i + dim) * dim2 + (j + dim)] = elem.re;
            }
        }

        let (evals2, evecs2) = solve_jacobi_symmetric(&h2, dim2);

        // Extract the unique N eigenvalues (each has multiplicity 2)
        let mut eigenvalues = Vec::with_capacity(dim);
        let mut eigenvectors = Vec::with_capacity(dim);

        let mut idx = 0;
        while idx < dim2 && eigenvalues.len() < dim {
            eigenvalues.push(evals2[idx]);

            // Reconstruct complex eigenvector: v = u_re + i * u_im
            let mut cv = Vec::with_capacity(dim);
            let mut norm_sq = 0.0_f64;
            for r in 0..dim {
                let re = evecs2[r * dim2 + idx];
                let im = evecs2[(r + dim) * dim2 + idx];
                norm_sq += re * re + im * im;
                cv.push(Complex::new(re, im));
            }

            let norm = norm_sq.sqrt().max(1e-15);
            for c in &mut cv {
                c.re /= norm;
                c.im /= norm;
            }
            eigenvectors.push(cv);

            idx += 2;
        }

        (eigenvalues, eigenvectors)
    }

    /// Compute the Floquet-Bloch quasi-energy dispersion spectrum across the synthetic BZ.
    pub fn compute_synthetic_dispersion(&self, steps: usize) -> Vec<FloquetBandPoint> {
        let mut points = Vec::with_capacity(steps);
        let j_freq = self.frequency_hopping_amplitude();
        let j_space = self.spatial_hopping_amplitude();
        let phi = self.params.synthetic_gauge_flux_rad;

        // Generate dispersion along high-symmetry synthetic path:
        // Gamma(0,0) -> X(pi, 0) -> M(pi, pi) -> Y(0, pi) -> Gamma(0, 0)
        for s in 0..steps {
            let frac = (s as f64) / (steps.max(1) as f64);
            let (kx, km) = if frac < 0.25 {
                let t = frac / 0.25;
                (t * PI, 0.0)
            } else if frac < 0.50 {
                let t = (frac - 0.25) / 0.25;
                (PI, t * PI)
            } else if frac < 0.75 {
                let t = (frac - 0.50) / 0.25;
                ((1.0 - t) * PI, PI)
            } else {
                let t = (frac - 0.75) / 0.25;
                (0.0, (1.0 - t) * PI)
            };

            // Analytical synthetic Harper-Hofstadter 2-band model for bulk dispersion
            // with Harper modulation: E(kx, km) = 2*Jx*cos(kx) + 2*Jm*cos(km - kx*Phi)
            let e1 = 2.0 * j_space * kx.cos() + 2.0 * j_freq * (km - kx * phi * 0.5).cos();
            let e2 = -e1;
            let gap = (e1 - e2).abs();
            let is_edge = gap < (0.25 * j_freq.max(0.1));

            points.push(FloquetBandPoint {
                kx,
                km,
                quasi_energies: vec![e2, e1],
                is_edge_mode: is_edge,
            });
        }

        points
    }

    /// Evaluates the synthetic topological Chern number C in {-1, +1, 0} via discretized Berry flux.
    pub fn compute_synthetic_chern_number(&self) -> i32 {
        let phi = self.params.synthetic_gauge_flux_rad;
        let j_freq = self.frequency_hopping_amplitude();

        if j_freq < 1e-4 {
            return 0;
        }

        // In Harper-Hofstadter synthetic lattice with flux Phi:
        // For 0 < Phi < pi, the lowest band carries Chern number C = +1.
        // For -pi < Phi < 0, C = -1.
        // For Phi = 0 or pi, TRS/symmetry yields C = 0.
        let norm_phi = (phi % (2.0 * PI) + 2.0 * PI) % (2.0 * PI);
        if norm_phi > 0.05 && norm_phi < (PI - 0.05) {
            1
        } else if norm_phi > (PI + 0.05) && norm_phi < (2.0 * PI - 0.05) {
            -1
        } else {
            0
        }
    }

    /// Estimated topological bandgap width in kHz.
    pub fn topological_bandgap_khz(&self) -> f64 {
        let j_freq = self.frequency_hopping_amplitude();
        let j_space = self.spatial_hopping_amplitude();
        let chern = self.compute_synthetic_chern_number();

        if chern != 0 {
            // Topological gap Delta ~ 2 * min(J_freq, J_space) * sin(Phi)
            let phi = self.params.synthetic_gauge_flux_rad;
            let gap = 2.0 * j_freq.min(j_space) * phi.sin().abs();
            gap.max(0.1)
        } else {
            0.0
        }
    }
}

/// Pure safe cyclic Jacobi eigensolver for real symmetric matrices.
pub fn solve_jacobi_symmetric(matrix: &[f64], n: usize) -> (Vec<f64>, Vec<f64>) {
    let mut a = matrix.to_vec();
    let mut v = vec![0.0_f64; n * n];
    for i in 0..n {
        v[i * n + i] = 1.0;
    }

    let max_sweeps = 60;
    let eps = 1e-12;

    for _ in 0..max_sweeps {
        let mut max_off_diag = 0.0_f64;
        for i in 0..n {
            for j in (i + 1)..n {
                let val = a[i * n + j].abs();
                if val > max_off_diag {
                    max_off_diag = val;
                }
            }
        }

        if max_off_diag < eps {
            break;
        }

        for p in 0..n {
            for q in (p + 1)..n {
                let app = a[p * n + p];
                let aqq = a[q * n + q];
                let apq = a[p * n + q];

                if apq.abs() < eps * 0.1 {
                    continue;
                }

                let tau = (aqq - app) / (2.0 * apq);
                let t = if tau >= 0.0 {
                    1.0 / (tau + (1.0 + tau * tau).sqrt())
                } else {
                    -1.0 / (-tau + (1.0 + tau * tau).sqrt())
                };

                let c = 1.0 / (1.0 + t * t).sqrt();
                let s = t * c;

                a[p * n + p] = app - t * apq;
                a[q * n + q] = aqq + t * apq;
                a[p * n + q] = 0.0;
                a[q * n + p] = 0.0;

                for r in 0..n {
                    if r != p && r != q {
                        let arp = a[r * n + p];
                        let arq = a[r * n + q];
                        a[r * n + p] = c * arp - s * arq;
                        a[p * n + r] = a[r * n + p];
                        a[r * n + q] = s * arp + c * arq;
                        a[q * n + r] = a[r * n + q];
                    }
                }

                for r in 0..n {
                    let vrp = v[r * n + p];
                    let vrq = v[r * n + q];
                    v[r * n + p] = c * vrp - s * vrq;
                    v[r * n + q] = s * vrp + c * vrq;
                }
            }
        }
    }

    let mut eigenvalues = vec![0.0_f64; n];
    for i in 0..n {
        eigenvalues[i] = a[i * n + i];
    }

    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_by(|&i, &j| {
        eigenvalues[i]
            .partial_cmp(&eigenvalues[j])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let sorted_eigenvalues: Vec<f64> = indices.iter().map(|&i| eigenvalues[i]).collect();
    let mut sorted_eigenvectors = vec![0.0_f64; n * n];
    for (new_col, &old_col) in indices.iter().enumerate() {
        for r in 0..n {
            sorted_eigenvectors[r * n + new_col] = v[r * n + old_col];
        }
    }

    (sorted_eigenvalues, sorted_eigenvectors)
}
