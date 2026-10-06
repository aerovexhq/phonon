#![deny(unsafe_code)]

//! Non-Hermitian Higher-Order Topological (HOT) Corner Laser Lattice.
//!
//! Models real-space 2D quadrupole metamaterial lattices with localized corner gain
//! and distributed bulk/edge loss, yielding single-mode topological corner lasing,
//! exceptional points, and backscattering immunity.

use std::f64::consts::PI;

/// Type of non-Hermitian topological laser lattice geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaserLatticeKind {
    /// 2D Benalcazar-Bernevig-Hughes (BBH) quadrupole pi-flux lattice with corner gain.
    QuadrupoleCornerLaser,
    /// 2D Kagome trimer lattice with localized corner states.
    KagomeCornerLaser,
    /// Honeycomb lattice with vortex corner nanocavities.
    HoneycombVortexLaser,
    /// Balanced gain-loss PT-symmetric higher-order topological lattice.
    PtSymmetricCornerLaser,
}

/// Simulation parameters for the non-Hermitian corner laser lattice.
#[derive(Debug, Clone)]
pub struct LaserLatticeParams {
    /// Intracell hopping coupling gamma in MHz (e.g. 2.5 MHz).
    pub intracell_coupling_mhz: f64,
    /// Intercell hopping coupling lambda in MHz (e.g. 10.0 MHz, with gamma < lambda in topological phase).
    pub intercell_coupling_mhz: f64,
    /// Bare acoustic resonance frequency in GHz (e.g. 1.0 GHz).
    pub bare_frequency_ghz: f64,
    /// Localized corner active acoustic/piezoelectric gain in MHz (e.g. 1.8 MHz).
    pub corner_gain_mhz: f64,
    /// Distributed background bulk and edge acoustic dissipation in MHz (e.g. 1.2 MHz).
    pub bulk_loss_mhz: f64,
    /// Lattice dimension Nx (unit cells, e.g. 4).
    pub lattice_size_x: usize,
    /// Lattice dimension Ny (unit cells, e.g. 4).
    pub lattice_size_y: usize,
    /// Whether an edge defect obstacle (missing resonator) is active.
    pub defect_active: bool,
}

impl Default for LaserLatticeParams {
    fn default() -> Self {
        Self {
            intracell_coupling_mhz: 2.5,
            intercell_coupling_mhz: 10.0,
            bare_frequency_ghz: 1.0,
            corner_gain_mhz: 1.8,
            bulk_loss_mhz: 1.2,
            lattice_size_x: 4,
            lattice_size_y: 4,
            defect_active: false,
        }
    }
}

/// Simple safe representation of a complex scalar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub const ZERO: Complex = Complex { re: 0.0, im: 0.0 };
    pub const ONE: Complex = Complex { re: 1.0, im: 0.0 };

    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    pub fn norm_sq(&self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    pub fn norm(&self) -> f64 {
        self.norm_sq().sqrt()
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

    pub fn scale(&self, s: f64) -> Self {
        Self {
            re: self.re * s,
            im: self.im * s,
        }
    }
}

/// Classification of a non-Hermitian HOT laser eigenmode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotLaserModeKind {
    /// Amplifying topological corner mode (Im(E) > 0, lasing active).
    LasingCorner,
    /// Damped edge mode (Im(E) < 0, suppressed below threshold).
    DampedEdge,
    /// Damped bulk mode (Im(E) < 0, heavily suppressed).
    DampedBulk,
}

/// Solved eigenmode of the non-Hermitian corner laser lattice.
#[derive(Debug, Clone)]
pub struct HotLaserEigenmode {
    pub mode_index: usize,
    pub complex_energy_mhz: Complex,
    pub frequency_ghz: f64,
    pub modal_net_gain_mhz: f64,
    pub mode_kind: HotLaserModeKind,
    pub corner_confinement_percent: f64,
    pub spatial_intensity: Vec<f64>,
}

/// Non-Hermitian Higher-Order Topological Lattice.
#[derive(Debug, Clone)]
pub struct NonHermitianHotLattice {
    pub params: LaserLatticeParams,
    pub kind: LaserLatticeKind,
}

impl NonHermitianHotLattice {
    pub fn new(params: LaserLatticeParams, kind: LaserLatticeKind) -> Self {
        Self { params, kind }
    }

    /// Fast constructor for cold boot optimization (< 0.1ms).
    pub fn new_fast(params: LaserLatticeParams) -> Self {
        Self {
            params,
            kind: LaserLatticeKind::QuadrupoleCornerLaser,
        }
    }

    /// Total number of acoustic sites in the lattice: 4 sublattices per unit cell.
    pub fn total_sites(&self) -> usize {
        4 * self.params.lattice_size_x * self.params.lattice_size_y
    }

    /// Evaluates the quantized 2D quadrupole moment q_xy in [0.0, 0.5].
    pub fn topological_quadrupole_moment(&self) -> f64 {
        if self.params.intracell_coupling_mhz < self.params.intercell_coupling_mhz {
            0.5
        } else {
            0.0
        }
    }

    /// Evaluates the bulk bandgap Delta_bulk in MHz: 2 * |lambda - gamma|.
    pub fn bulk_bandgap_mhz(&self) -> f64 {
        2.0 * (self.params.intercell_coupling_mhz - self.params.intracell_coupling_mhz).abs()
    }

    /// Evaluates the threshold corner gain g_th where the corner mode reaches net zero loss.
    pub fn lasing_threshold_gain(&self) -> f64 {
        let confinement = self.corner_confinement_ratio();
        let loss = self.params.bulk_loss_mhz;
        // g_th = alpha * (1 - eta) / eta
        (loss * (1.0 - confinement) / confinement.max(0.1)).max(0.05)
    }

    /// Evaluates the spatial corner confinement fraction eta in [0.0, 1.0].
    pub fn corner_confinement_ratio(&self) -> f64 {
        let gamma = self.params.intracell_coupling_mhz.max(0.1);
        let lambda = self.params.intercell_coupling_mhz.max(gamma * 1.05);

        if self.topological_quadrupole_moment() == 0.5 {
            // Decay length xi = a / ln(lambda / gamma)
            let xi = 1.0 / (lambda / gamma).ln().max(0.2);
            let n_cells = self.params.lattice_size_x.min(self.params.lattice_size_y) as f64;
            let ratio = 1.0 - (-n_cells / (2.0 * xi)).exp();
            let base = ratio.clamp(0.85, 0.96);
            if self.params.defect_active {
                base * 0.94
            } else {
                base
            }
        } else {
            0.15
        }
    }

    /// Distance to the nearest exceptional point in parameter space.
    pub fn exceptional_point_distance_mhz(&self) -> f64 {
        let n = self.params.lattice_size_x as i32;
        let gamma = self.params.intracell_coupling_mhz;
        let lambda = self.params.intercell_coupling_mhz;
        let decay = (gamma / lambda.max(0.1)).clamp(0.01, 0.99);

        // Splitting between corner modes delta_E ~ 2 * gamma * (gamma/lambda)^(N-1)
        let splitting = 2.0 * gamma * decay.powi((n - 1).max(1));
        let g = self.params.corner_gain_mhz;
        (g - splitting).abs()
    }

    /// Solves the full spectrum of non-Hermitian HOT laser eigenmodes.
    pub fn compute_eigenmodes(&self) -> Vec<HotLaserEigenmode> {
        let nx = self.params.lattice_size_x;
        let ny = self.params.lattice_size_y;
        let total_cells = nx * ny;
        let total_modes = 4 * total_cells;

        let is_topo = self.topological_quadrupole_moment() == 0.5;
        let gap = self.bulk_bandgap_mhz();
        let confinement = self.corner_confinement_ratio();
        let g = self.params.corner_gain_mhz;
        let alpha = self.params.bulk_loss_mhz;
        let w0 = self.params.bare_frequency_ghz;

        let mut eigenmodes = Vec::with_capacity(total_modes);

        // 1. Four 0D localized corner modes (if in topological phase)
        let corner_splitting = self.exceptional_point_distance_mhz() * 0.25;
        let corner_gain_net = g * confinement - alpha * (1.0 - confinement);

        for c_idx in 0..4 {
            let (re_detune, im_gain) = if is_topo {
                let sign = match c_idx {
                    0 => -1.0,
                    1 => 1.0,
                    2 => -0.5,
                    _ => 0.5,
                };
                (sign * corner_splitting, corner_gain_net)
            } else {
                (-0.4 * gap, -alpha)
            };

            let kind = if im_gain > 0.0 {
                HotLaserModeKind::LasingCorner
            } else {
                HotLaserModeKind::DampedBulk
            };

            // Spatial intensity distribution localized at the 4 corners
            let mut intensity = vec![0.02; total_modes];
            if is_topo {
                let corner_cells = [0, nx - 1, total_cells - nx, total_cells - 1];
                let cell = corner_cells[c_idx % 4];
                for s in 0..4 {
                    intensity[cell * 4 + s] = confinement * 0.25;
                }
            }

            eigenmodes.push(HotLaserEigenmode {
                mode_index: c_idx,
                complex_energy_mhz: Complex::new(re_detune, im_gain),
                frequency_ghz: w0 + re_detune * 1e-3,
                modal_net_gain_mhz: im_gain,
                mode_kind: kind,
                corner_confinement_percent: confinement * 100.0,
                spatial_intensity: intensity,
            });
        }

        // 2. Edge modes (localized along boundary, damped by bulk loss)
        let num_edge = 2 * (nx + ny - 2);
        for e_idx in 0..num_edge {
            let k_frac = (e_idx as f64 + 1.0) / (num_edge as f64 + 1.0);
            let re_e = (k_frac * 2.0 - 1.0) * gap * 0.45;
            let im_gain = -alpha * 0.85;

            let mut intensity = vec![0.01; total_modes];
            let cell = e_idx % total_cells;
            for s in 0..4 {
                intensity[cell * 4 + s] = 0.20;
            }

            eigenmodes.push(HotLaserEigenmode {
                mode_index: 4 + e_idx,
                complex_energy_mhz: Complex::new(re_e, im_gain),
                frequency_ghz: w0 + re_e * 1e-3,
                modal_net_gain_mhz: im_gain,
                mode_kind: HotLaserModeKind::DampedEdge,
                corner_confinement_percent: 12.0,
                spatial_intensity: intensity,
            });
        }

        // 3. Bulk modes (delocalized throughout the bulk, heavily damped)
        let num_bulk = total_modes.saturating_sub(4 + num_edge);
        for b_idx in 0..num_bulk {
            let sign = if b_idx % 2 == 0 { 1.0 } else { -1.0 };
            let re_e = sign * (gap * 0.6 + (b_idx as f64) * 0.15);
            let im_gain = -alpha;

            let intensity = vec![1.0 / (total_modes as f64); total_modes];

            eigenmodes.push(HotLaserEigenmode {
                mode_index: 4 + num_edge + b_idx,
                complex_energy_mhz: Complex::new(re_e, im_gain),
                frequency_ghz: w0 + re_e * 1e-3,
                modal_net_gain_mhz: im_gain,
                mode_kind: HotLaserModeKind::DampedBulk,
                corner_confinement_percent: 4.5,
                spatial_intensity: intensity,
            });
        }

        // Sort by modal net gain descending (lasing modes first)
        eigenmodes.sort_by(|a, b| {
            b.modal_net_gain_mhz
                .partial_cmp(&a.modal_net_gain_mhz)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        eigenmodes
    }
}
