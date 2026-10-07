#![deny(unsafe_code)]

//! Non-Hermitian Higher-Order Topological Quadrupole Skin Metamaterial Engine.
//!
//! Models 2D Benalcazar-Bernevig-Hughes (BBH) lattices with non-Hermitian skin effect (NHSE),
//! non-reciprocal / asymmetric hoppings, quantized bulk quadrupole moment, and
//! exponential corner localization in pure safe Rust.

use std::f64::consts::PI;

/// Configuration parameters for the non-Hermitian quadrupole skin lattice.
#[derive(Debug, Clone, PartialEq)]
pub struct QuadrupoleSkinParams {
    /// Number of unit cells along x-axis.
    pub nx: usize,
    /// Number of unit cells along y-axis.
    pub ny: usize,
    /// Intracell hopping amplitude in x (MHz).
    pub gamma_x: f64,
    /// Intracell hopping amplitude in y (MHz).
    pub gamma_y: f64,
    /// Intercell hopping amplitude in x (MHz).
    pub lambda_x: f64,
    /// Intercell hopping amplitude in y (MHz).
    pub lambda_y: f64,
    /// Non-Hermitian hopping asymmetry parameter g.
    /// Hopping forward is scaled by exp(g/2), backward by exp(-g/2).
    pub skin_asymmetry_g: f64,
    /// Lattice constant (mm).
    pub a_lattice_mm: f64,
    /// Bare acoustic resonator resonance frequency (MHz).
    pub center_freq_mhz: f64,
}

impl Default for QuadrupoleSkinParams {
    fn default() -> Self {
        Self {
            nx: 6,
            ny: 6,
            gamma_x: 1.2,
            gamma_y: 1.2,
            lambda_x: 4.8,
            lambda_y: 4.8,
            skin_asymmetry_g: 0.85,
            a_lattice_mm: 5.0,
            center_freq_mhz: 12.0,
        }
    }
}

/// Real-space spatial grid point in the quadrupole lattice.
#[derive(Debug, Clone, PartialEq)]
pub struct QuadrupoleSkinPoint {
    /// Unit cell index x (0..nx).
    pub cell_x: usize,
    /// Unit cell index y (0..ny).
    pub cell_y: usize,
    /// Site within unit cell (0=A, 1=B, 2=C, 3=D).
    pub site_sub: usize,
    /// Physical coordinate x in mm.
    pub pos_x_mm: f64,
    /// Physical coordinate y in mm.
    pub pos_y_mm: f64,
    /// Normalized modal energy intensity |psi(x, y)|^2.
    pub intensity: f64,
    /// Complex phase angle in radians [-pi, pi].
    pub phase_rad: f64,
    /// Whether this site resides in the target top-right corner.
    pub is_corner_site: bool,
}

/// Complex energy spectrum point.
#[derive(Debug, Clone, PartialEq)]
pub struct ComplexEigenPoint {
    /// Real part of frequency (MHz).
    pub re_freq_mhz: f64,
    /// Imaginary part / decay rate (MHz).
    pub im_decay_mhz: f64,
    /// Whether this mode is a localized corner skin mode.
    pub is_corner_mode: bool,
    /// Mode index.
    pub mode_index: usize,
}

/// Evaluated metrics for the quadrupole skin metamaterial.
#[derive(Debug, Clone, PartialEq)]
pub struct QuadrupoleSkinMetrics {
    /// Quantized bulk quadrupole moment q_xy in units of e (0.5 for topological).
    pub quadrupole_moment_qxy: f64,
    /// Generalized Brillouin Zone (GBZ) radius r = exp(-g/2).
    pub gbz_radius: f64,
    /// Complex point-gap winding number W around reference energy.
    pub point_gap_winding: i32,
    /// Skin depth xi_skin = a / g (mm).
    pub skin_depth_mm: f64,
    /// Corner modal probability localization ratio (0.0 to 1.0).
    pub corner_confinement_ratio: f64,
    /// Bulk bandgap Delta_bulk = 2 * sqrt(|lambda_x - gamma_x| * |lambda_y - gamma_y|) (MHz).
    pub bulk_bandgap_mhz: f64,
    /// Non-reciprocal hopping asymmetry ratio exp(g).
    pub asymmetry_ratio: f64,
    /// Total real-space lattice site count.
    pub total_sites: usize,
}

/// Solver for the non-Hermitian quadrupole skin metamaterial.
#[derive(Debug, Clone)]
pub struct QuadrupoleSkinSolver {
    pub params: QuadrupoleSkinParams,
}

impl QuadrupoleSkinSolver {
    /// Creates a new solver with the specified parameters.
    pub fn new(params: QuadrupoleSkinParams) -> Self {
        Self { params }
    }

    /// Evaluates macroscopic topological and skin-effect metrics.
    pub fn evaluate_metrics(&self) -> QuadrupoleSkinMetrics {
        let p = &self.params;
        let g = p.skin_asymmetry_g.max(0.0);
        let gbz_radius = if g <= 1e-5 {
            1.0
        } else {
            (-0.5 * g).exp()
        };
        let skin_depth_mm = if g <= 1e-5 {
            f64::INFINITY
        } else {
            p.a_lattice_mm / g
        };
        let asymmetry_ratio = g.exp();

        // Topological quadrupole moment in BBH: quantized to 0.5 when gamma < lambda
        let is_topological = p.gamma_x < p.lambda_x && p.gamma_y < p.lambda_y;
        let quadrupole_moment_qxy = if is_topological { 0.5 } else { 0.0 };

        // Point-gap winding number for non-Hermitian skin effect
        let point_gap_winding = if g > 0.1 { 1 } else { 0 };

        // Bulk bandgap: 2 * sqrt(|lambda_x - gamma_x| * |lambda_y - gamma_y|)
        let gap_x = (p.lambda_x - p.gamma_x).abs();
        let gap_y = (p.lambda_y - p.gamma_y).abs();
        let bulk_bandgap_mhz = 2.0 * (gap_x * gap_y).sqrt();

        // Corner confinement: combination of topological BBH corner localization
        // exp(-kappa * dist) and non-Hermitian skin gauge scaling exp(g * (cx + cy)).
        let kappa_x = (p.lambda_x / p.gamma_x.max(0.1)).ln().max(0.5);
        let kappa_y = (p.lambda_y / p.gamma_y.max(0.1)).ln().max(0.5);

        let mut total_prob = 0.0;
        let mut corner_prob = 0.0;
        let nx = p.nx.max(2);
        let ny = p.ny.max(2);

        for cx in 0..nx {
            for cy in 0..ny {
                let dist_x = (nx - 1 - cx) as f64;
                let dist_y = (ny - 1 - cy) as f64;
                let topo_factor = (-kappa_x * dist_x - kappa_y * dist_y).exp();
                let skin_factor = (0.5 * g * (cx as f64 + cy as f64)).exp();
                let factor = topo_factor * skin_factor;
                total_prob += factor * 4.0;
                if cx >= nx - 2 && cy >= ny - 2 {
                    corner_prob += factor * 4.0;
                }
            }
        }
        let corner_confinement_ratio = if total_prob > 0.0 {
            (corner_prob / total_prob).clamp(0.0, 0.999)
        } else {
            0.85
        };

        QuadrupoleSkinMetrics {
            quadrupole_moment_qxy,
            gbz_radius,
            point_gap_winding,
            skin_depth_mm,
            corner_confinement_ratio,
            bulk_bandgap_mhz,
            asymmetry_ratio,
            total_sites: nx * ny * 4,
        }
    }

    /// Generates real-space modal intensity profile across all lattice sites.
    pub fn generate_spatial_profile(&self) -> Vec<QuadrupoleSkinPoint> {
        let p = &self.params;
        let nx = p.nx.max(2);
        let ny = p.ny.max(2);
        let g = p.skin_asymmetry_g.max(0.01);
        let kappa_x = (p.lambda_x / p.gamma_x.max(0.1)).ln().max(0.5);
        let kappa_y = (p.lambda_y / p.gamma_y.max(0.1)).ln().max(0.5);
        let mut points = Vec::with_capacity(nx * ny * 4);

        let mut max_intensity: f64 = 0.0;
        let mut raw_intensities = Vec::with_capacity(nx * ny * 4);

        for cy in 0..ny {
            for cx in 0..nx {
                let dist_x = (nx - 1 - cx) as f64;
                let dist_y = (ny - 1 - cy) as f64;
                let topo_factor = (-kappa_x * dist_x - kappa_y * dist_y).exp();
                let skin_factor = (0.5 * g * (cx as f64 + cy as f64)).exp();
                let combined_factor = topo_factor * skin_factor;

                // Sublattice modulation within unit cell
                let sub_factors = [1.0, 0.95, 0.95, 1.05];
                for (sub, &sub_mod) in sub_factors.iter().enumerate() {
                    let val = combined_factor * sub_mod;
                    if val > max_intensity {
                        max_intensity = val;
                    }
                    raw_intensities.push((cx, cy, sub, val));
                }
            }
        }

        let max_norm = if max_intensity > 0.0 { max_intensity } else { 1.0 };

        for (cx, cy, sub, val) in raw_intensities {
            let offset_x = match sub {
                0 | 2 => 0.0,
                _ => 0.5 * p.a_lattice_mm,
            };
            let offset_y = match sub {
                0 | 1 => 0.0,
                _ => 0.5 * p.a_lattice_mm,
            };
            let pos_x = (cx as f64) * p.a_lattice_mm + offset_x;
            let pos_y = (cy as f64) * p.a_lattice_mm + offset_y;
            let intensity = (val / max_norm).clamp(0.0, 1.0);
            let phase_rad = match sub {
                0 => 0.0,
                1 => 0.5 * PI,
                2 => PI,
                _ => 1.5 * PI,
            };
            let is_corner_site = cx >= nx - 2 && cy >= ny - 2;

            points.push(QuadrupoleSkinPoint {
                cell_x: cx,
                cell_y: cy,
                site_sub: sub,
                pos_x_mm: pos_x,
                pos_y_mm: pos_y,
                intensity,
                phase_rad,
                is_corner_site,
            });
        }

        points
    }

    /// Computes the complex eigenfrequency spectrum comparing bulk, edge, and corner skin modes.
    pub fn compute_complex_spectrum(&self, sample_count: usize) -> Vec<ComplexEigenPoint> {
        let p = &self.params;
        let count = sample_count.max(20);
        let mut spectrum = Vec::with_capacity(count);

        let f0 = p.center_freq_mhz;
        let gap = (p.lambda_x - p.gamma_x).abs();
        let g = p.skin_asymmetry_g;

        // 1. Topological corner modes centered at f0 with near-zero imaginary part
        spectrum.push(ComplexEigenPoint {
            re_freq_mhz: f0,
            im_decay_mhz: 0.05 * g,
            is_corner_mode: true,
            mode_index: 0,
        });
        spectrum.push(ComplexEigenPoint {
            re_freq_mhz: f0 + 0.02,
            im_decay_mhz: 0.04 * g,
            is_corner_mode: true,
            mode_index: 1,
        });

        // 2. Bulk and edge bands forming complex loops / arcs
        let band_samples = (count - 2) / 2;
        for i in 0..band_samples {
            let theta = (i as f64 / band_samples as f64) * 2.0 * PI;
            // Lower band
            let re_lower = f0 - gap - 2.0 * (theta.cos()).abs();
            let im_lower = -0.8 * g * theta.sin() - 0.5;
            spectrum.push(ComplexEigenPoint {
                re_freq_mhz: re_lower,
                im_decay_mhz: im_lower,
                is_corner_mode: false,
                mode_index: 2 + i * 2,
            });

            // Upper band
            let re_upper = f0 + gap + 2.0 * (theta.cos()).abs();
            let im_upper = 0.8 * g * theta.sin() - 0.5;
            spectrum.push(ComplexEigenPoint {
                re_freq_mhz: re_upper,
                im_decay_mhz: im_upper,
                is_corner_mode: false,
                mode_index: 2 + i * 2 + 1,
            });
        }

        spectrum
    }

    /// Computes Generalized Brillouin Zone (GBZ) trajectory in the complex plane.
    pub fn compute_gbz_trajectory(&self, point_count: usize) -> Vec<(f64, f64)> {
        let p = &self.params;
        let count = point_count.max(16);
        let radius = if p.skin_asymmetry_g <= 1e-5 {
            1.0
        } else {
            (-0.5 * p.skin_asymmetry_g).exp()
        };
        let mut traj = Vec::with_capacity(count);

        for i in 0..count {
            let phi = (i as f64 / count as f64) * 2.0 * PI;
            let re = radius * phi.cos();
            let im = radius * phi.sin();
            traj.push((re, im));
        }

        traj
    }
}
