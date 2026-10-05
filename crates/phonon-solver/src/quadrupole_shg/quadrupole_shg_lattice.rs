#![deny(unsafe_code)]

//! Benalcazar-Bernevig-Hughes (BBH) higher-order quadrupole topological lattice
//! and corner-localized second-harmonic generation (SHG) modal engine.
//!
//! Models a 2D quadrupole topological insulator with pi-flux per plaquette and
//! quadratic acoustic non-linearity chi^(2) concentrating second-harmonic generation
//! in zero-dimensional corner nanocavities.

use std::f64::consts::PI;

/// Parameters defining the 2D quadrupole topological acoustic lattice for SHG.
#[derive(Debug, Clone)]
pub struct QuadrupoleShgParams {
    /// Intracell hopping amplitude gamma in kHz (default ~2.0 kHz).
    pub intracell_gamma_khz: f64,
    /// Intercell hopping amplitude lambda in kHz (default ~10.0 kHz, SOTI when gamma < lambda).
    pub intercell_lambda_khz: f64,
    /// Fundamental acoustic resonance frequency in Hz (default ~2500 Hz).
    pub fundamental_freq_hz: f64,
    /// Quadratic elastic / acoustic non-linearity coefficient chi^(2) in Pa^-1.
    pub non_linear_chi2: f64,
    /// Quality factor of the fundamental corner mode (Q1).
    pub quality_factor_q1: f64,
    /// Quality factor of the second-harmonic corner mode (Q2).
    pub quality_factor_q2: f64,
    /// Number of unit cells along x-axis (Nx >= 3).
    pub nx: usize,
    /// Number of unit cells along y-axis (Ny >= 3).
    pub ny: usize,
    /// Structural coupling disorder amplitude W in units of lambda (0.0 to 0.4).
    pub disorder_w: f64,
}

impl Default for QuadrupoleShgParams {
    fn default() -> Self {
        Self {
            intracell_gamma_khz: 2.0,
            intercell_lambda_khz: 10.0,
            fundamental_freq_hz: 2500.0,
            non_linear_chi2: 0.08,
            quality_factor_q1: 1200.0,
            quality_factor_q2: 1800.0,
            nx: 4,
            ny: 4,
            disorder_w: 0.0,
        }
    }
}

impl QuadrupoleShgParams {
    /// Hopping ratio r = gamma / lambda. SOTI topological phase when r < 1.0.
    #[inline]
    pub fn hopping_ratio(&self) -> f64 {
        if self.intercell_lambda_khz.abs() < 1e-9 {
            1e6
        } else {
            self.intracell_gamma_khz / self.intercell_lambda_khz
        }
    }

    /// Whether the lattice is in the topological second-order (SOTI) phase.
    #[inline]
    pub fn is_topological_soti(&self) -> bool {
        self.hopping_ratio() < 1.0
    }

    /// Quantized bulk quadrupole moment q_xy (0.5 for SOTI, 0.0 for trivial).
    #[inline]
    pub fn bulk_quadrupole_moment(&self) -> f64 {
        if self.is_topological_soti() {
            0.5
        } else {
            0.0
        }
    }

    /// Corner mode localization decay length xi in unit cells.
    #[inline]
    pub fn corner_decay_length(&self) -> f64 {
        let r = self.hopping_ratio();
        if r < 1.0 && r > 1e-6 {
            1.0 / (1.0 / r).ln()
        } else {
            10.0
        }
    }

    /// Bulk bandgap Delta_bulk in kHz: 2 * |lambda - gamma|.
    #[inline]
    pub fn bulk_bandgap_khz(&self) -> f64 {
        2.0 * (self.intercell_lambda_khz - self.intracell_gamma_khz).abs()
    }
}

/// Modal profile and metrics for the corner-localized fundamental and SHG modes.
#[derive(Debug, Clone)]
pub struct CornerShgModalMetrics {
    /// Corner energy confinement ratio for fundamental mode (0.0 to 1.0).
    pub fundamental_corner_confinement: f64,
    /// Corner energy confinement ratio for second-harmonic mode (0.0 to 1.0).
    pub shg_corner_confinement: f64,
    /// Non-linear modal overlap integral kappa_SHG between modes 1 and 2.
    pub non_linear_overlap_integral: f64,
    /// Phase mismatch Delta omega / omega_1.
    pub phase_mismatch_ratio: f64,
    /// Second-harmonic center frequency in Hz: f_2 = 2 * f_1.
    pub second_harmonic_freq_hz: f64,
}

/// Solver for the 2D quadrupole topological lattice and corner SHG modes.
#[derive(Debug, Clone)]
pub struct QuadrupoleShgSolver {
    pub params: QuadrupoleShgParams,
    pub metrics: CornerShgModalMetrics,
    /// 2D intensity field |P_1(x, y)|^2 across lattice sites for fundamental mode.
    pub fundamental_intensity_grid: Vec<Vec<f64>>,
    /// 2D intensity field |P_2(x, y)|^2 across lattice sites for second-harmonic mode.
    pub shg_intensity_grid: Vec<Vec<f64>>,
}

impl QuadrupoleShgSolver {
    /// Construct a new quadrupole SHG solver and solve initial fields.
    pub fn new(params: QuadrupoleShgParams) -> Self {
        let mut solver = Self {
            params,
            metrics: CornerShgModalMetrics {
                fundamental_corner_confinement: 0.0,
                shg_corner_confinement: 0.0,
                non_linear_overlap_integral: 0.0,
                phase_mismatch_ratio: 0.0,
                second_harmonic_freq_hz: 5000.0,
            },
            fundamental_intensity_grid: Vec::new(),
            shg_intensity_grid: Vec::new(),
        };
        solver.recompute();
        solver
    }

    /// Recompute corner modal fields, non-linear overlap, and confinement metrics.
    pub fn recompute(&mut self) {
        let nx_cells = self.params.nx.max(3);
        let ny_cells = self.params.ny.max(3);

        // 2 sites per unit cell per axis -> total sites = (2*Nx) x (2*Ny)
        let sx = nx_cells * 2;
        let sy = ny_cells * 2;

        self.fundamental_intensity_grid = vec![vec![0.0; sx]; sy];
        self.shg_intensity_grid = vec![vec![0.0; sx]; sy];

        let is_topo = self.params.is_topological_soti();
        let xi = self.params.corner_decay_length().max(0.2);
        let disorder = self.params.disorder_w.clamp(0.0, 0.4);

        // Calculate analytical tight-binding wavefunctions:
        // In topological phase, corner states decay exponentially away from the 4 corners:
        // Corner 1: (0, 0), Corner 2: (sx-1, 0), Corner 3: (0, sy-1), Corner 4: (sx-1, sy-1).
        let corners = [
            (0.0, 0.0),
            ((sx - 1) as f64, 0.0),
            (0.0, (sy - 1) as f64),
            ((sx - 1) as f64, (sy - 1) as f64),
        ];

        let mut sum_fund = 0.0;
        let mut sum_shg = 0.0;
        let mut corner_fund_sum = 0.0;
        let mut corner_shg_sum = 0.0;

        for iy in 0..sy {
            let y = iy as f64;
            for ix in 0..sx {
                let x = ix as f64;

                // Distance to nearest corner in cell units (each cell has 2 sites)
                let mut min_dist_cells = 1e9;
                for &(cx, cy) in &corners {
                    let dx = (x - cx).abs() * 0.5;
                    let dy = (y - cy).abs() * 0.5;
                    let d = dx + dy; // Taxicab distance in square Manhattan lattice
                    if d < min_dist_cells {
                        min_dist_cells = d;
                    }
                }

                let (p1, p2) = if is_topo {
                    // Exponential corner localization with small pseudorandom disorder perturbation
                    let phase_mod = ((ix * 7 + iy * 13) as f64 * PI / 4.0).sin();
                    let disorder_pert = 1.0 + disorder * 0.5 * phase_mod;

                    let env1 = (-min_dist_cells / xi).exp() * disorder_pert;
                    // Second harmonic has tighter sub-wavelength overlap due to quadratic dependence
                    let env2 = (-min_dist_cells / (xi * 0.75)).exp() * disorder_pert;

                    (env1 * env1, env2 * env2)
                } else {
                    // Trivial phase: extended bulk Bloch waves with zero corner peaking
                    let kx = (ix as f64 + 1.0) * PI / (sx as f64 + 1.0);
                    let ky = (iy as f64 + 1.0) * PI / (sy as f64 + 1.0);
                    let wave = (kx.sin() * ky.sin()).abs();
                    (wave * wave, wave * wave)
                };

                self.fundamental_intensity_grid[iy][ix] = p1;
                self.shg_intensity_grid[iy][ix] = p2;

                sum_fund += p1;
                sum_shg += p2;

                // Corner area definition: 4 corner unit cells (2x2 sites each)
                let is_in_corner_cell = (ix <= 1 || ix >= sx - 2) && (iy <= 1 || iy >= sy - 2);
                if is_in_corner_cell {
                    corner_fund_sum += p1;
                    corner_shg_sum += p2;
                }
            }
        }

        // Normalize grids
        if sum_fund > 1e-12 {
            for row in &mut self.fundamental_intensity_grid {
                for val in row {
                    *val /= sum_fund;
                }
            }
        }
        if sum_shg > 1e-12 {
            for row in &mut self.shg_intensity_grid {
                for val in row {
                    *val /= sum_shg;
                }
            }
        }

        let fundamental_confinement = if sum_fund > 1e-12 {
            (corner_fund_sum / sum_fund).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let shg_confinement = if sum_shg > 1e-12 {
            (corner_shg_sum / sum_shg).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Spatial overlap integral kappa_SHG = sum_{x,y} chi^(2) * P1(x,y) * sqrt(P2(x,y))
        let mut overlap_integral = 0.0;
        for iy in 0..sy {
            for ix in 0..sx {
                let p1 = self.fundamental_intensity_grid[iy][ix];
                let p2 = self.shg_intensity_grid[iy][ix];
                overlap_integral += self.params.non_linear_chi2 * p1 * p2.sqrt();
            }
        }

        let f1 = self.params.fundamental_freq_hz;
        let f2 = 2.0 * f1;

        // Phase mismatch is minimal in topological corner mode
        let phase_mismatch = if is_topo {
            0.002 * (1.0 + disorder * 0.5)
        } else {
            0.15
        };

        self.metrics = CornerShgModalMetrics {
            fundamental_corner_confinement: fundamental_confinement,
            shg_corner_confinement: shg_confinement,
            non_linear_overlap_integral: overlap_integral,
            phase_mismatch_ratio: phase_mismatch,
            second_harmonic_freq_hz: f2,
        };
    }
}
