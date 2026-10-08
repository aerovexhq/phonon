#![deny(unsafe_code)]

//! Phase 435: Acoustic Corner Mode & Piezoelectric Transduction Engine.
//!
//! Models a 2D higher-order topological acoustic metamaterial (HOTI)
//! with localized 0D corner states at mid-gap, high spatial confinement,
//! and piezoelectric electromechanical coupling to superconducting transmon circuits.

/// Parameters for the topological acoustic corner mode metamaterial.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticCornerParams {
    /// Resonator bare frequency f0 in GHz (default: 5.0 GHz).
    pub f0_ghz: f64,
    /// Intracell hopping amplitude gamma in MHz (default: 2.5 MHz).
    pub gamma_mhz: f64,
    /// Intercell hopping amplitude lambda in MHz (default: 12.5 MHz).
    pub lambda_mhz: f64,
    /// Number of unit cells along each spatial dimension (default: 6).
    pub grid_size: usize,
    /// Acoustic mechanical quality factor Q (default: 2.5e5).
    pub q_factor: f64,
    /// Piezoelectric electromechanical transduction coupling rate g_trans/2pi in MHz (default: 15.0 MHz).
    pub coupling_trans_mhz: f64,
}

impl Default for AcousticCornerParams {
    fn default() -> Self {
        Self {
            f0_ghz: 5.0,
            gamma_mhz: 2.5,
            lambda_mhz: 12.5,
            grid_size: 6,
            q_factor: 2.5e5,
            coupling_trans_mhz: 15.0,
        }
    }
}

/// Point on the 2D spatial acoustic grid.
#[derive(Debug, Clone, PartialEq)]
pub struct CornerSpatialPoint {
    /// Normalized X coordinate in [0.0, 1.0].
    pub x: f64,
    /// Normalized Y coordinate in [0.0, 1.0].
    pub y: f64,
    /// Local acoustic modal intensity |psi(x, y)|^2.
    pub intensity: f64,
    /// Phase in radians [-pi, pi].
    pub phase: f64,
    /// Whether this site belongs to a topological corner unit cell.
    pub is_corner: bool,
}

/// Key physical metrics for the topological corner mode.
#[derive(Debug, Clone, PartialEq)]
pub struct CornerModeMetrics {
    /// Resonant frequency of the 0D corner mode in GHz.
    pub corner_frequency_ghz: f64,
    /// Topological bulk bandgap Delta_bulk in MHz (2 * |lambda - gamma|).
    pub bulk_gap_mhz: f64,
    /// Spatial modal confinement ratio in corner sites (fraction in [0.0, 1.0]).
    pub confinement_ratio: f64,
    /// Spatial decay length in unit cell units xi = 1.0 / ln(lambda / gamma).
    pub decay_length_cells: f64,
    /// Electromechanical transduction coupling rate g_trans/2pi in MHz.
    pub transduction_rate_mhz: f64,
    /// Acoustic decay rate kappa_ph/2pi in kHz (f0 / Q).
    pub acoustic_damping_khz: f64,
    /// Whether the metamaterial is strictly in the topological SOTI phase (gamma < lambda).
    pub is_topological: bool,
}

/// Solver for the 2D higher-order topological acoustic metamaterial.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticCornerSolver {
    pub params: AcousticCornerParams,
}

impl Default for AcousticCornerSolver {
    fn default() -> Self {
        Self {
            params: AcousticCornerParams::default(),
        }
    }
}

impl AcousticCornerSolver {
    /// Creates a new solver with the given parameters.
    pub fn new(params: AcousticCornerParams) -> Self {
        Self { params }
    }

    /// Solves the acoustic corner mode spatial profile and evaluates physical metrics.
    pub fn solve_corner_mode(&self) -> (CornerModeMetrics, Vec<CornerSpatialPoint>) {
        let p = &self.params;
        let is_topological = p.gamma_mhz < p.lambda_mhz;

        // Bulk bandgap Delta_bulk = 2 * |lambda - gamma|
        let bulk_gap_mhz = 2.0 * (p.lambda_mhz - p.gamma_mhz).abs();

        // Corner state frequency is at midgap (f0)
        let corner_frequency_ghz = p.f0_ghz;

        // Decay length xi = 1 / ln(lambda / gamma)
        let ratio = if p.gamma_mhz > 1e-6 {
            p.lambda_mhz / p.gamma_mhz
        } else {
            100.0
        };
        let decay_length_cells = if ratio > 1.0 {
            1.0 / ratio.ln()
        } else {
            10.0 // Delocalized / trivial
        };

        // Acoustic dissipation rate kappa_ph / 2pi = f0 / Q
        // f0 is in GHz (1e9 Hz), Q is unitless. Damping in kHz:
        let acoustic_damping_khz = (p.f0_ghz * 1e6) / p.q_factor;

        let n = p.grid_size.max(4);
        let mut points = Vec::with_capacity(n * n);
        let mut corner_energy = 0.0;
        let mut total_energy = 0.0;

        // Four physical corners in cell index coordinates (0..n-1)
        let corners = [(0, 0), (n - 1, 0), (0, n - 1), (n - 1, n - 1)];

        for iy in 0..n {
            let y_norm = iy as f64 / (n - 1) as f64;
            for ix in 0..n {
                let x_norm = ix as f64 / (n - 1) as f64;

                let is_corner_cell = (ix == 0 || ix == n - 1) && (iy == 0 || iy == n - 1);

                // Calculate minimum distance to nearest corner in cell units
                let mut min_dist_cells = f64::MAX;
                for &(cx, cy) in &corners {
                    let dx = (ix as f64 - cx as f64).abs();
                    let dy = (iy as f64 - cy as f64).abs();
                    let d = (dx * dx + dy * dy).sqrt();
                    if d < min_dist_cells {
                        min_dist_cells = d;
                    }
                }

                // In topological phase, corner modes decay exponentially away from corners:
                // psi ~ exp(-d / xi)
                let amplitude = if is_topological {
                    (-min_dist_cells / decay_length_cells).exp()
                } else {
                    // In trivial phase, modes are extended throughout the bulk
                    1.0 / (n as f64)
                };

                let intensity = amplitude * amplitude;
                total_energy += intensity;
                if is_corner_cell {
                    corner_energy += intensity;
                }

                // Quadrupole pi-flux phase pattern alternating in signs
                let phase = if (ix + iy) % 2 == 0 { 0.0 } else { std::f64::consts::PI };

                points.push(CornerSpatialPoint {
                    x: x_norm,
                    y: y_norm,
                    intensity,
                    phase,
                    is_corner: is_corner_cell,
                });
            }
        }

        // Normalize intensities so sum(intensity) = 1.0
        let norm_factor = if total_energy > 1e-12 {
            1.0 / total_energy
        } else {
            1.0
        };

        for pt in &mut points {
            pt.intensity *= norm_factor;
        }

        let confinement_ratio = if total_energy > 1e-12 {
            corner_energy / total_energy
        } else {
            0.0
        };

        let metrics = CornerModeMetrics {
            corner_frequency_ghz,
            bulk_gap_mhz,
            confinement_ratio,
            decay_length_cells,
            transduction_rate_mhz: p.coupling_trans_mhz,
            acoustic_damping_khz,
            is_topological,
        };

        (metrics, points)
    }
}
