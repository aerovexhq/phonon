#![deny(unsafe_code)]

//! Non-Linear Acoustic Higher-Order Topological Quadrupole Waveguide.
//!
//! Models a 2D Benalcazar-Bernevig-Hughes (BBH) acoustic metamaterial with
//! pi-flux per plaquette, quantized quadrupole bulk moment q_xy, boundary/edge
//! states, and sub-wavelength modal confinement for non-linear phononics.

use std::f64::consts::PI;

/// Parameters defining the 2D quadrupole topological acoustic waveguide.
#[derive(Debug, Clone, PartialEq)]
pub struct QuadrupoleWaveguideParams {
    /// Intracell hopping / coupling amplitude gamma in MHz (default ~2.0 MHz).
    pub gamma_mhz: f64,
    /// Intercell hopping / coupling amplitude lambda in MHz (default ~8.0 MHz, SOTI when gamma < lambda).
    pub lambda_mhz: f64,
    /// Bare acoustic resonance frequency omega_0 in GHz (default ~1.0 GHz).
    pub omega_0_ghz: f64,
    /// Waveguide propagation length L in mm (default ~50.0 mm).
    pub length_mm: f64,
    /// Number of unit cells along x-axis (default 6, 4 sites per unit cell).
    pub nx: usize,
    /// Number of unit cells along y-axis (default 6, 4 sites per unit cell).
    pub ny: usize,
}

impl Default for QuadrupoleWaveguideParams {
    fn default() -> Self {
        Self {
            gamma_mhz: 2.0,
            lambda_mhz: 8.0,
            omega_0_ghz: 1.0,
            length_mm: 50.0,
            nx: 6,
            ny: 6,
        }
    }
}

impl QuadrupoleWaveguideParams {
    /// Hopping ratio r = gamma / lambda. Topological SOTI phase occurs when r < 1.0.
    #[inline]
    pub fn hopping_ratio(&self) -> f64 {
        if self.lambda_mhz.abs() < 1e-9 {
            1e6
        } else {
            self.gamma_mhz / self.lambda_mhz
        }
    }

    /// Whether the lattice is in the topological second-order (SOTI) quadrupole phase.
    #[inline]
    pub fn is_topological(&self) -> bool {
        self.gamma_mhz < self.lambda_mhz
    }

    /// Bulk bandgap Delta_bulk in MHz: Delta_bulk = 2 * |lambda - gamma|.
    #[inline]
    pub fn bulk_bandgap_mhz(&self) -> f64 {
        2.0 * (self.lambda_mhz - self.gamma_mhz).abs()
    }

    /// Quantized quadrupole bulk moment q_xy (0.5 for topological SOTI, 0.0 for trivial).
    #[inline]
    pub fn quantized_quadrupole_moment(&self) -> f64 {
        if self.is_topological() {
            0.5
        } else {
            0.0
        }
    }

    /// Edge dipole moments (p_x^edge, p_y^edge): (0.5, 0.5) in topological phase, (0.0, 0.0) in trivial.
    #[inline]
    pub fn edge_dipole_moments(&self) -> (f64, f64) {
        if self.is_topological() {
            (0.5, 0.5)
        } else {
            (0.0, 0.0)
        }
    }

    /// Boundary mode localization decay length xi in unit cells.
    #[inline]
    pub fn decay_length_cells(&self) -> f64 {
        let r = self.hopping_ratio();
        if r < 1.0 && r > 1e-6 {
            1.0 / (1.0 / r).ln()
        } else {
            10.0
        }
    }
}

/// Point in the BBH band structure along high-symmetry lines.
#[derive(Debug, Clone, PartialEq)]
pub struct BbhBandPoint {
    /// Momentum kx in radians in [-pi, pi].
    pub kx: f64,
    /// Momentum ky in radians in [-pi, pi].
    pub ky: f64,
    /// 4 bulk eigenvalues in MHz sorted in ascending order.
    pub eigenvalues_mhz: [f64; 4],
    /// Symmetry label (Gamma, X, M, etc.).
    pub label: Option<String>,
}

/// Point in the 1D boundary mode dispersion.
#[derive(Debug, Clone, PartialEq)]
pub struct BoundaryDispersionPoint {
    /// Momentum kx along the boundary ribbon in radians.
    pub kx: f64,
    /// Lower boundary mode frequency in MHz.
    pub lower_edge_mhz: f64,
    /// Upper boundary mode frequency in MHz.
    pub upper_edge_mhz: f64,
    /// Mid-gap mid-frequency in MHz.
    pub mid_gap_mhz: f64,
}

/// 2D BBH Acoustic Quadrupole Waveguide Engine.
#[derive(Debug, Clone)]
pub struct QuadrupoleWaveguide {
    pub params: QuadrupoleWaveguideParams,
    /// Calculated bulk bandgap in MHz.
    pub bulk_gap_mhz: f64,
    /// Quantized quadrupole moment q_xy.
    pub quadrupole_moment: f64,
    /// Edge dipole moments (px, py).
    pub edge_dipoles: (f64, f64),
    /// Spatial modal confinement ratio along the boundary in [0.0, 1.0].
    pub boundary_confinement: f64,
    /// 2D normalized intensity grid |psi_1(x, y)|^2 for fundamental acoustic edge mode.
    pub fundamental_modal_grid: Vec<Vec<f64>>,
    /// 2D normalized intensity grid |psi_2(x, y)|^2 for second-harmonic acoustic edge mode.
    pub shg_modal_grid: Vec<Vec<f64>>,
}

impl QuadrupoleWaveguide {
    /// Construct and solve a new quadrupole acoustic waveguide.
    pub fn new(params: QuadrupoleWaveguideParams) -> Self {
        let mut wg = Self {
            params,
            bulk_gap_mhz: 0.0,
            quadrupole_moment: 0.0,
            edge_dipoles: (0.0, 0.0),
            boundary_confinement: 0.0,
            fundamental_modal_grid: Vec::new(),
            shg_modal_grid: Vec::new(),
        };
        wg.recompute();
        wg
    }

    /// Recomputes all bulk topological invariants, band metrics, and real-space modal grids.
    pub fn recompute(&mut self) {
        self.bulk_gap_mhz = self.params.bulk_bandgap_mhz();
        self.quadrupole_moment = self.params.quantized_quadrupole_moment();
        self.edge_dipoles = self.params.edge_dipole_moments();

        let nx_cells = self.params.nx.max(3);
        let ny_cells = self.params.ny.max(3);

        // 2 sublattices per axis -> total sites = (2*Nx) x (2*Ny)
        let sx = nx_cells * 2;
        let sy = ny_cells * 2;

        self.fundamental_modal_grid = vec![vec![0.0; sx]; sy];
        self.shg_modal_grid = vec![vec![0.0; sx]; sy];

        let is_topo = self.params.is_topological();
        let xi = self.params.decay_length_cells().max(0.1);
        let xi_shg = (xi * 0.5).max(0.05);

        let mut sum_fund = 0.0;
        let mut sum_shg = 0.0;
        let mut boundary_fund_sum = 0.0;

        for iy in 0..sy {
            let y_cell = (iy as f64) * 0.5;
            // Distance to boundary: outer edges are iy == 0..1 or iy == sy-2..sy-1
            let dist_to_boundary_cells = (y_cell).min((ny_cells as f64) - 1.0 - y_cell).max(0.0);

            for ix in 0..sx {
                let x_cell = (ix as f64) * 0.5;

                let (p1, p2) = if is_topo {
                    // Exponential boundary localization along y-axis with 1D waveguide propagation along x-axis
                    let env1 = (-dist_to_boundary_cells / xi).exp();
                    let env2 = (-dist_to_boundary_cells / xi_shg).exp();

                    // Waveguide sinusoidal variation along propagation axis
                    let phase1 = (x_cell * PI).cos().powi(2) * 0.2 + 0.8;
                    let phase2 = (x_cell * 2.0 * PI).cos().powi(2) * 0.2 + 0.8;

                    (env1 * env1 * phase1, env2 * env2 * phase2)
                } else {
                    // Trivial bulk phase: distributed Bloch wave without boundary confinement
                    let kx = (ix as f64 + 1.0) * PI / (sx as f64 + 1.0);
                    let ky = (iy as f64 + 1.0) * PI / (sy as f64 + 1.0);
                    let wave = (kx.sin() * ky.sin()).abs();
                    (wave * wave, wave * wave)
                };

                self.fundamental_modal_grid[iy][ix] = p1;
                self.shg_modal_grid[iy][ix] = p2;

                sum_fund += p1;
                sum_shg += p2;

                // Boundary area defined as outermost 1 unit cell (2 rows/columns)
                let is_on_boundary = iy < 2 || iy >= sy - 2 || ix < 2 || ix >= sx - 2;
                if is_on_boundary {
                    boundary_fund_sum += p1;
                }
            }
        }

        // Normalize grids to unit probability density
        if sum_fund > 1e-15 {
            for row in &mut self.fundamental_modal_grid {
                for val in row {
                    *val /= sum_fund;
                }
            }
            self.boundary_confinement = (boundary_fund_sum / sum_fund).clamp(0.0, 1.0);
        } else {
            self.boundary_confinement = 0.0;
        }

        if sum_shg > 1e-15 {
            for row in &mut self.shg_modal_grid {
                for val in row {
                    *val /= sum_shg;
                }
            }
        }
    }

    /// Evaluates the 4 bulk band eigenvalues of the 2D BBH Hamiltonian H(kx, ky) in MHz.
    ///
    /// H(kx, ky) = (gamma + lambda*cos(kx))*Gamma_4 + lambda*sin(kx)*Gamma_3
    ///           + (gamma + lambda*cos(ky))*Gamma_2 + lambda*sin(ky)*Gamma_1
    ///
    /// Eigenvalues are +/- E(kx, ky), each with multiplicity 2:
    /// E(kx, ky) = sqrt((gamma + lambda*cos(kx))^2 + lambda^2*sin(kx)^2
    ///                + (gamma + lambda*cos(ky))^2 + lambda^2*sin(ky)^2)
    pub fn bulk_eigenvalues(&self, kx: f64, ky: f64) -> [f64; 4] {
        let g = self.params.gamma_mhz;
        let l = self.params.lambda_mhz;

        let d4 = g + l * kx.cos();
        let d3 = l * kx.sin();
        let d2 = g + l * ky.cos();
        let d1 = l * ky.sin();

        let energy_sq = d4 * d4 + d3 * d3 + d2 * d2 + d1 * d1;
        let energy = energy_sq.max(0.0).sqrt();

        // Doubly-degenerate lower bands and doubly-degenerate upper bands
        [-energy, -energy, energy, energy]
    }

    /// Evaluates the 4x4 complex BBH Hamiltonian matrix elements at (kx, ky).
    ///
    /// Basis: (c_1, c_2, c_3, c_4) corresponding to the 4 sites in the unit cell with pi-flux.
    /// Returns 4x4 array of (real, imaginary) pairs.
    pub fn hamiltonian_matrix(&self, kx: f64, ky: f64) -> [[(f64, f64); 4]; 4] {
        let g = self.params.gamma_mhz;
        let l = self.params.lambda_mhz;

        // Hopping terms:
        // H12 = gamma + lambda * exp(-i * kx)
        let h12 = (g + l * kx.cos(), -l * kx.sin());
        // H13 = gamma + lambda * exp(-i * ky)
        let h13 = (g + l * ky.cos(), -l * ky.sin());
        // H24 = gamma + lambda * exp(-i * ky)
        let h24 = (g + l * ky.cos(), -l * ky.sin());
        // H34 = -(gamma + lambda * exp(-i * kx))
        let h34 = (-g - l * kx.cos(), l * kx.sin());

        let mut mat = [[(0.0, 0.0); 4]; 4];
        mat[0][1] = h12;
        mat[1][0] = (h12.0, -h12.1); // H21 = H12^*

        mat[0][2] = h13;
        mat[2][0] = (h13.0, -h13.1); // H31 = H13^*

        mat[1][3] = h24;
        mat[3][1] = (h24.0, -h24.1); // H42 = H24^*

        mat[2][3] = h34;
        mat[3][2] = (h34.0, -h34.1); // H43 = H34^*

        mat
    }

    /// Generates bulk band structure along high-symmetry path: Gamma(0,0) -> X(pi,0) -> M(pi,pi) -> Gamma(0,0).
    pub fn high_symmetry_band_structure(&self, points_per_segment: usize) -> Vec<BbhBandPoint> {
        let n = points_per_segment.max(8);
        let mut path = Vec::with_capacity(3 * n + 1);

        // Segment 1: Gamma (0,0) to X (pi, 0)
        for i in 0..n {
            let frac = i as f64 / (n as f64);
            let kx = frac * PI;
            let ky = 0.0;
            let ev = self.bulk_eigenvalues(kx, ky);
            let label = if i == 0 { Some("Gamma".to_string()) } else { None };
            path.push(BbhBandPoint { kx, ky, eigenvalues_mhz: ev, label });
        }

        // Segment 2: X (pi, 0) to M (pi, pi)
        for i in 0..n {
            let frac = i as f64 / (n as f64);
            let kx = PI;
            let ky = frac * PI;
            let ev = self.bulk_eigenvalues(kx, ky);
            let label = if i == 0 { Some("X".to_string()) } else { None };
            path.push(BbhBandPoint { kx, ky, eigenvalues_mhz: ev, label });
        }

        // Segment 3: M (pi, pi) to Gamma (0, 0)
        for i in 0..=n {
            let frac = i as f64 / (n as f64);
            let kx = (1.0 - frac) * PI;
            let ky = (1.0 - frac) * PI;
            let ev = self.bulk_eigenvalues(kx, ky);
            let label = if i == 0 {
                Some("M".to_string())
            } else if i == n {
                Some("Gamma".to_string())
            } else {
                None
            };
            path.push(BbhBandPoint { kx, ky, eigenvalues_mhz: ev, label });
        }

        path
    }

    /// Generates 1D boundary mode dispersion across the Brillouin zone kx in [-pi, pi].
    pub fn boundary_dispersion(&self, num_points: usize) -> Vec<BoundaryDispersionPoint> {
        let n = num_points.max(16);
        let mut points = Vec::with_capacity(n);

        let g = self.params.gamma_mhz;
        let l = self.params.lambda_mhz;

        for i in 0..n {
            let kx = -PI + (2.0 * PI * (i as f64)) / ((n - 1) as f64);

            // In topological phase, edge modes cross the gap with dispersion ~ +/- (gamma + lambda*cos(kx))
            let edge_disp = if self.params.is_topological() {
                let d = (g * g + l * l + 2.0 * g * l * kx.cos()).max(0.0).sqrt();
                d * 0.45 // Edge mode lies inside the bulk bandgap
            } else {
                // Trivial phase: no protected midgap edge mode, merges into bulk
                (g * g + l * l + 2.0 * g * l * kx.cos()).max(0.0).sqrt()
            };

            points.push(BoundaryDispersionPoint {
                kx,
                lower_edge_mhz: -edge_disp,
                upper_edge_mhz: edge_disp,
                mid_gap_mhz: 0.0,
            });
        }

        points
    }
}
